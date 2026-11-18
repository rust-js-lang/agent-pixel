mod artboard_styles;
mod browser;
mod canvas_styles;
mod dashboard_styles;
mod editor_styles;
mod journey_styles;
mod shell_styles;

mod components;
mod dashboard;
mod journey;
mod projects;
mod transport;
mod view;
mod webmcp;
mod zoom;

pub use transport::serve_worker;

use js::JsError;
use pixel_geometry::scene::{ComponentInsert, ComponentSwitch, Design, Document, Patch, Scene};
use projects::Project;
use react::dom::client::{Root, create_root};
use std::cell::{Cell, RefCell};
use webapi::{EventTargetExt, StorageEvent, events, window_global::window};

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
    screen: Option<&'static journey::Screen>,
    preview: bool,
    zoom: f64,
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

    fn storage_key(&self) -> String {
        let base = self.project.storage_key(self.server);
        match self.screen {
            Some(screen) if screen.id != journey::SCREENS[0].id => format!("{base}:{}", screen.id),
            _ => base,
        }
    }

    fn initial_scene(&self) -> Scene {
        match self.screen {
            Some(screen) => screen.initial_scene(),
            None => self.project.initial_scene(),
        }
    }

    fn save_local(&self) -> Result<(), String> {
        let body = serde_json::to_string(&self.document).map_err(|e| e.to_string())?;
        browser::save_draft(&self.storage_key(), Some(&body)).map_err(message)
    }
}

pub fn mount() {
    let node = window()
        .document()
        .get_element_by_id("root")
        .expect("Editor root");
    let root = create_root(node, None);
    ROOT.set(Some(root));
    let Some((id, server)) = projects::route() else {
        render_dashboard();
        js::spawn(Box::new(async {
            if let Err(error) = webmcp::register_dashboard_tools().await {
                webapi::console::error((error.message(),));
            }
        }));
        return;
    };
    let Some(project) = projects::find(&id) else {
        root.render(dashboard::missing());
        return;
    };
    EDITOR.with_borrow_mut(|editor| {
        if project.id == journey::PROJECT {
            editor.screen = Some(
                browser::query_value("screen")
                    .as_deref()
                    .and_then(journey::find)
                    .unwrap_or(&journey::SCREENS[0]),
            );
            editor.preview = browser::workspace_query("preview");
        }
        editor.project = project;
        editor.zoom = zoom::FIT;
        editor.server = server;
    });
    if !EDITOR.with_borrow(|editor| editor.preview) {
        window().add_event_listener(events::Keydown, zoom::keyboard, None);
    }
    status("Opening document…".into());
    js::spawn(Box::new(async {
        if let Err(error) = start().await {
            status(error);
        }
    }));
}

fn render_dashboard() {
    if let Some(root) = ROOT.get() {
        root.render(dashboard::home());
    }
}

fn render() {
    if let Some(root) = ROOT.get() {
        let element = EDITOR.with_borrow(|editor| {
            if editor.preview {
                view::prototype(editor)
            } else {
                view::editor(editor)
            }
        });
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
    let (key, server) = EDITOR.with_borrow(|editor| (editor.storage_key(), editor.server));
    let local = projects::load_document(&key);
    let (saved, mut warning) = match local {
        Ok(Some(saved)) => (saved, None),
        Ok(None) if server => (fetch_document(None).await?, None),
        Ok(None) => (Document::default(), None),
        Err(_) => (
            Document::default(),
            Some("Browser draft unavailable".to_string()),
        ),
    };
    let scene = saved
        .scene
        .unwrap_or_else(|| EDITOR.with_borrow(Editor::initial_scene));
    let design = match restore(&scene).await {
        Ok(restored) => restored,
        Err(error) => {
            warning = Some(error);
            restore(&EDITOR.with_borrow(Editor::initial_scene)).await?
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
    refresh_linked_components().await;
    window().add_event_listener(
        events::Storage,
        |event: &StorageEvent| {
            if event
                .key()
                .is_none_or(|key| key == components::library_key())
            {
                js::spawn(Box::new(refresh_linked_components()));
            }
        },
        None,
    );
    render();
    if EDITOR.with_borrow(|editor| editor.preview) {
        return Ok(());
    }
    let result = webmcp::register_editor_tools().await;
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
    change_design("edit", &json).await
}

async fn place_component(id: &str, x: i32, y: i32) -> Result<String, String> {
    let placement = ComponentInsert {
        variant: components::component(id)?,
        x,
        y,
    };
    let json = serde_json::to_string(&placement).map_err(|error| error.to_string())?;
    change_design("insert", &json).await
}

async fn change_variant(id: String, variant: &str) -> Result<String, String> {
    let switch = ComponentSwitch {
        id,
        variant: components::component(variant)?,
    };
    let json = serde_json::to_string(&switch).map_err(|error| error.to_string())?;
    change_design("switch", &json).await
}

async fn refresh_linked_components() {
    let has_instances = EDITOR.with_borrow(|editor| {
        !editor.busy
            && editor
                .document
                .scene
                .as_ref()
                .is_some_and(|scene| !scene.instances.is_empty())
    });
    if !has_instances {
        return;
    }
    let result = async {
        let body =
            serde_json::to_string(&components::library()?).map_err(|error| error.to_string())?;
        request("refresh_components", &body).await
    }
    .await;
    match result {
        Ok(design) => {
            let changed = EDITOR.with_borrow_mut(|editor| {
                if editor.document.scene.as_ref() == Some(&design.scene) {
                    return false;
                }
                editor.document.scene = Some(design.scene);
                if editor.save_local().is_ok() {
                    editor.status = "Linked components updated from Canopy UI".into();
                } else {
                    editor.status = "Linked components updated in memory".into();
                }
                true
            });
            if changed {
                render();
            }
        }
        Err(error) => status(error),
    }
}

async fn change_design(command: &str, json: &str) -> Result<String, String> {
    if EDITOR.with_borrow(|editor| editor.busy) {
        return Err("Wait for the document server operation.".into());
    }
    let design = request(command, json).await?;
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
        refresh_linked_components().await;
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
