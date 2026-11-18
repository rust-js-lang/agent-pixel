mod browser;
mod view;

use js::JsError;
use pixel_geometry::scene::{Design, Document, Patch, Scene};
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
        browser::save_draft(self.storage_key(), Some(&body)).map_err(message)
    }

    fn storage_key(&self) -> &'static str {
        if self.server {
            "agent-pixel:poster:v1:server"
        } else {
            "agent-pixel:poster:v1"
        }
    }
}

const TOOLS: &str = r##"[
  {
    "name": "read_design",
    "description": "Read the artboard and its ordered layers.",
    "inputSchema": {"type":"object","properties":{},"additionalProperties":false},
    "annotations": {"readOnlyHint":true}
  },
  {
    "name": "edit_layer",
    "description": "Patch a layer by ID or omit ID to create a shape. Coordinates are artboard pixels.",
    "inputSchema": {
      "type":"object","minProperties":1,"additionalProperties":false,
      "properties":{
        "id":{"type":"string"},"kind":{"type":"string","enum":["rectangle","ellipse"]},
        "x":{"type":"integer"},"y":{"type":"integer"},"width":{"type":"integer"},"height":{"type":"integer"},
        "fill":{"type":"string","pattern":"^#[0-9A-Fa-f]{6}$"}
      }
    }
  }
]"##;

pub fn mount() {
    let node = window()
        .document()
        .get_element_by_id("root")
        .expect("Editor root");
    ROOT.set(Some(create_root(node, None)));
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
    let body = browser::request_worker(command, body)
        .await
        .map_err(message)?;
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
    let server = browser::server_mode();
    EDITOR.with_borrow_mut(|editor| editor.server = server);
    let key = EDITOR.with_borrow(Editor::storage_key);
    let local = browser::load_draft(key).map_err(message).and_then(|value| {
        value
            .map(|json| serde_json::from_str::<Document>(&json))
            .transpose()
            .map_err(|e| e.to_string())
    });
    let (saved, mut warning) = match local {
        Ok(Some(saved)) => (saved, None),
        Ok(None) if server => (fetch_document(None).await?, None),
        Ok(None) => (Document::default(), None),
        Err(_) => (
            Document::default(),
            Some("Browser draft unavailable".to_string()),
        ),
    };
    let mut design = request("read", "null").await?;
    if let Some(scene) = saved.scene {
        match restore(&scene).await {
            Ok(restored) => design = restored,
            Err(error) => warning = Some(error),
        }
    }
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
    let result = browser::register_tools(
        TOOLS,
        || js::promise(read_design()),
        |input| js::promise(edit_design(input)),
    )
    .await;
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
            browser::save_draft(editor.storage_key(), None).map_err(message)?;
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
        browser::save_draft(editor.storage_key(), None).map_err(message)?;
        Ok(format!("Loaded server revision {}", saved.revision))
    })
}
