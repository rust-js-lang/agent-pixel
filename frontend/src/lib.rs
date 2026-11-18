mod browser;
mod dashboard;
mod projects;
mod transport;
mod view;
mod webmcp;

pub use transport::serve_worker;

use js::JsError;
use pixel_geometry::scene::{Design, Document, Patch, Scene};
use projects::Project;
use react::dom::client::{Root, create_root};
use std::cell::{Cell, RefCell};
use webapi::window_global::window;

thread_local! {
    static ROOT: Cell<Option<&'static Root>> = const { Cell::new(None) };
    static EDITOR: RefCell<Editor> = RefCell::new(Editor::default());
}

#[derive(Default)]
struct Editor {
    document: Document,
    selected: Option<String>,
    server: bool,
    project: Project,
    busy: bool,
    fields_version: u32,
    status: String,
    agent: String,
}

impl Editor {
    fn select(&mut self, id: Option<String>) {
        self.selected = id;
    }

    fn replace_scene(&mut self, scene: Scene) {
        self.select(scene.layers.last().map(|layer| layer.id.clone()));
        self.document.scene = Some(scene);
    }

    fn save_local(&self) -> Result<(), String> {
        let body = serde_json::to_string(&self.document).map_err(|e| e.to_string())?;
        browser::save_draft(&self.project.storage_key(self.server), Some(&body)).map_err(message)
    }
}

pub fn mount() {
    let node = window()
        .document()
        .get_element_by_id("root")
        .expect("Editor root");
    let root = create_root(node, None);
    let Some((id, server)) = projects::route() else {
        root.render(dashboard::home());
        return;
    };
    let (catalog, _) = projects::list();
    let Some(project) = catalog.into_iter().find(|project| project.id == id) else {
        root.render(dashboard::missing());
        return;
    };
    EDITOR.with_borrow_mut(|editor| {
        editor.project = project;
        editor.server = server;
    });
    ROOT.set(Some(root));
    status("Opening document…".into());
    js::spawn(Box::new(async {
        if let Err(error) = start().await {
            status(error);
        }
    }));
}

fn render() {
    if let Some(root) = ROOT.get() {
        let element = EDITOR.with_borrow(view::editor);
        root.render(element);
    }
}

fn status(message: String) {
    EDITOR.with_borrow_mut(|editor| editor.status = message);
    render();
}

fn message(error: &JsError) -> String {
    error.message()
}

async fn request(command: &str, body: &str) -> Result<Design, String> {
    let body = transport::request(command, body).await?;
    serde_json::from_str(&body).map_err(|error| error.to_string())
}

async fn fetch_document(body: Option<&str>) -> Result<Document, String> {
    let response = browser::document_request(body).await.map_err(message)?;
    match response.status {
        200 => serde_json::from_str(&response.body).map_err(|error| error.to_string()),
        409 => Err("Server changed · Local draft kept".into()),
        _ => Err("Document server unavailable".into()),
    }
}

async fn start() -> Result<(), String> {
    let (project, server) = EDITOR.with_borrow(|editor| (editor.project.clone(), editor.server));
    let local = project.load(server);
    let (saved, mut warning) = match local {
        Ok(Some(saved)) => (saved, None),
        Ok(None) if server => (fetch_document(None).await?, None),
        Ok(None) => (Document::default(), None),
        Err(_) => (
            Document::default(),
            Some("Browser draft unavailable".to_string()),
        ),
    };
    let scene = saved.scene.unwrap_or_else(|| project.initial_scene());
    let design = match restore(&scene).await {
        Ok(restored) => restored,
        Err(error) => {
            warning = Some(error);
            restore(&project.initial_scene()).await?
        }
    };
    EDITOR.with_borrow_mut(|editor| {
        editor.replace_scene(design.scene);
        editor.document.revision = saved.revision;
        editor.status = warning.unwrap_or_else(|| {
            if server {
                format!("Server workspace · Revision {}", saved.revision)
            } else {
                "Browser workspace · Ready".into()
            }
        });
    });
    render();
    let result = webmcp::register_tools().await;
    EDITOR.with_borrow_mut(|editor| {
        editor.agent = match result {
            Ok(true) => "WebMCP tools ready".into(),
            Ok(false) => "WebMCP unavailable in this browser".into(),
            Err(error) => format!("WebMCP registration failed: {}", error.message()),
        }
    });
    render();
    Ok(())
}

async fn restore(scene: &Scene) -> Result<Design, String> {
    let json = serde_json::to_string(scene).map_err(|error| error.to_string())?;
    request("restore", &json).await
}

async fn read_design() -> Result<String, String> {
    let design = request("read", "null").await?;
    serde_json::to_string(&design).map_err(|error| error.to_string())
}

async fn edit_design(json: String) -> Result<String, String> {
    if EDITOR.with_borrow(|editor| editor.busy) {
        return Err("Wait for the document server operation.".into());
    }
    let design = request("edit", &json).await?;
    let body = serde_json::to_string(&design).map_err(|error| error.to_string())?;
    EDITOR.with_borrow_mut(|editor| {
        editor.select(design.edited);
        editor.document.scene = Some(design.scene);
        editor.status = if editor.save_local().is_err() {
            "Browser storage unavailable · Edits are in memory"
        } else if editor.server {
            "Local draft saved · Save to server to publish this revision"
        } else {
            "Saved in this browser"
        }
        .into();
    });
    render();
    Ok(body)
}

fn edit(patch: Patch) {
    js::spawn(Box::new(async move {
        let json = serde_json::to_string(&patch).expect("Serialize patch");
        if let Err(error) = edit_design(json).await {
            EDITOR.with_borrow_mut(|editor| editor.fields_version += 1);
            status(error);
        }
    }));
}

fn select(id: String) {
    EDITOR.with_borrow_mut(|editor| {
        if editor.selected.as_ref() != Some(&id) {
            editor.select(Some(id));
        }
    });
    render();
}

fn sync_to_server(load: bool) {
    if load
        && !window().confirm(Some(
            "Replace the current draft with the saved server copy?",
        ))
    {
        return;
    }
    if EDITOR.with_borrow_mut(|editor| std::mem::replace(&mut editor.busy, true)) {
        return;
    }
    render();
    js::spawn(Box::new(async move {
        let result = if load {
            load_server().await
        } else {
            save_server().await
        };
        EDITOR.with_borrow_mut(|editor| editor.busy = false);
        status(result.unwrap_or_else(|error| error));
    }));
}

async fn save_server() -> Result<String, String> {
    let json = EDITOR
        .with_borrow(|editor| serde_json::to_string(&editor.document))
        .map_err(|error| error.to_string())?;
    let saved = fetch_document(Some(&json)).await?;
    EDITOR.with_borrow_mut(|editor| {
        editor.document.revision = saved.revision;
        if editor.document.scene == saved.scene {
            browser::save_draft(&editor.project.storage_key(editor.server), None)
                .map_err(message)?;
            Ok(format!("Saved server revision {}", saved.revision))
        } else {
            editor.save_local()?;
            Ok("Newer edits remain in your local draft".into())
        }
    })
}

async fn load_server() -> Result<String, String> {
    let saved = fetch_document(None).await?;
    let scene = saved.scene.ok_or("No saved server copy yet")?;
    let design = restore(&scene).await?;
    EDITOR.with_borrow_mut(|editor| {
        editor.replace_scene(design.scene);
        editor.document.revision = saved.revision;
        browser::save_draft(&editor.project.storage_key(editor.server), None).map_err(message)?;
        Ok(format!("Loaded server revision {}", saved.revision))
    })
}
