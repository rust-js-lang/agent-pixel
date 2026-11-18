//! Browser bindings for the experimental WebMCP model context.
use pixel_geometry::scene::{Author, Batch};

use crate::projects;
use js::{JsError, JsObject, Promise, Unknown, json, promise};
use serde::Deserialize;
use std::marker::PhantomData;

struct ModelContext(PhantomData<JsObject>);

#[cfg_attr(rust_js, rust_js::link_name = "get document.modelContext")]
fn document_context() -> Option<&'static ModelContext> {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "get navigator.modelContext")]
fn navigator_context() -> Option<&'static ModelContext> {
    unreachable!()
}

impl ModelContext {
    #[cfg_attr(rust_js, rust_js::link_name = "registerTool")]
    fn register(&self, tool: Tool) -> Result<Option<Promise<()>>, &'static JsError> {
        unreachable!()
    }
}

type Execute = Box<dyn Fn(Option<&'static Unknown>) -> Promise<&'static Unknown>>;

struct Tool {
    name: &'static str,
    description: &'static str,
    input_schema: &'static Unknown,
    annotations: Option<Annotations>,
    execute: Execute,
}

struct Annotations {
    read_only_hint: bool,
}

const SCREEN_EDIT_SCHEMA: &str = r#"{
    "type":"object","required":["revision","edit"],"additionalProperties":false,"properties":{
        "revision":{"type":"string","description":"Current project screen revision from list_screens"},
        "edit":{"oneOf":[
            {"type":"object","required":["op","title"],"additionalProperties":false,"properties":{"op":{"const":"create"},"title":{"type":"string"}}},
            {"type":"object","required":["op","id","title"],"additionalProperties":false,"properties":{"op":{"const":"rename"},"id":{"type":"string"},"title":{"type":"string"}}},
            {"type":"object","required":["op","id","title"],"additionalProperties":false,"properties":{"op":{"const":"duplicate"},"id":{"type":"string"},"title":{"type":"string"}}},
            {"type":"object","required":["op","id","index"],"additionalProperties":false,"properties":{"op":{"const":"reorder"},"id":{"type":"string"},"index":{"type":"integer","minimum":0,"description":"Zero-based position in the screen list"}}},
            {"type":"object","required":["op","id"],"additionalProperties":false,"properties":{"op":{"const":"delete"},"id":{"type":"string"}}}
        ]}
    }
}"#;
const SCREEN_OPEN_SCHEMA: &str = r#"{"type":"object","required":["id"],"additionalProperties":false,"properties":{"id":{"type":"string","description":"Stable screen ID from list_screens"}}}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScreenMutation {
    revision: String,
    edit: pixel_geometry::screens::ScreenEdit,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenScreen {
    id: String,
}

fn open_screen(input: Option<&Unknown>) -> Result<String, String> {
    let OpenScreen { id } = decode(input, "Provide a screen ID.")?;
    let href = crate::screen_href(&id)?;
    let json = serde_json::json!({ "id": id, "href": href }).to_string();
    webapi::window_global::location().assign(&href);
    Ok(json)
}

const PANEL_SCHEMA: &str = r#"{"type":"object","required":["panel"],"additionalProperties":false,"properties":{"panel":{"enum":["file","assets"],"type":"string"}}}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspacePanel {
    panel: Panel,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Panel {
    File,
    Assets,
}

const READ_SCHEMA: &str = r#"{"type":"object","properties":{},"additionalProperties":false}"#;
const EDIT_SCHEMA: &str = r##"{
    "type":"object","minProperties":1,"additionalProperties":false,
    "properties":{
        "id":{"type":"string"},
        "kind":{"type":"string","enum":["rectangle","ellipse","text"]},
        "x":{"type":"integer"},"y":{"type":"integer"},"width":{"type":"integer"},"height":{"type":"integer"},
        "fill":{"type":"string","description":"Hex fill or text color","pattern":"^#[0-9A-Fa-f]{6}$"},
        "text":{"type":"string","minLength":1,"maxLength":512,"description":"Content for a text layer"},
        "fontSize":{"type":"integer","minimum":8,"maximum":96}
    }
}"##;

const INSERT_SCHEMA: &str = r#"{
    "type":"object","required":["id","x","y"],"additionalProperties":false,
    "properties":{
        "id":{"type":"string","description":"Variant ID from list_components"},
        "x":{"type":"integer","minimum":0,"description":"Left of the component in artboard pixels"},
        "y":{"type":"integer","minimum":0,"description":"Top of the component in artboard pixels"}
    }
}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InsertComponent {
    id: String,
    x: i32,
    y: i32,
}

const SWITCH_SCHEMA: &str = r#"{
    "type":"object","required":["instanceId","variantId"],"additionalProperties":false,
    "properties":{
        "instanceId":{"type":"string","description":"Instance ID from read_design's scene.instances"},
        "variantId":{"type":"string","description":"Variant ID from the same family in list_components"}
    }
}"#;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SwitchVariant {
    instance_id: String,
    variant_id: String,
}

const REVISION_SCHEMA: &str = r#"{"type":"object","required":["revision"],"additionalProperties":false,"properties":{"revision":{"type":"string"}}}"#;
const BATCH_SCHEMA: &str = r#"{
    "type":"object","required":["revision","edits"],"additionalProperties":false,
    "properties":{
        "revision":{"type":"string","description":"Current revision from read_design"},
        "edits":{"type":"array","minItems":1,"maxItems":100,"items":{"oneOf":[
            {"type":"object","required":["op","layer"],"additionalProperties":false,"properties":{
                "op":{"const":"patch"},"layer":{"type":"object","description":"Same fields as edit_layer. Omit ID to create a layer."}
            }},
            {"type":"object","required":["op","placement"],"additionalProperties":false,"properties":{"op":{"const":"place_image"},"placement":{"type":"object","required":["image","x","y","width","height"],"additionalProperties":false,"properties":{"image":{"type":"object","required":["name","dataUrl"],"additionalProperties":false,"properties":{"name":{"type":"string"},"dataUrl":{"type":"string","description":"Base64 PNG, JPEG or WebP data URL, at most 512 KiB decoded"}}},"x":{"type":"integer"},"y":{"type":"integer"},"width":{"type":"integer"},"height":{"type":"integer"}}}}},
            {"type":"object","required":["op","id"],"additionalProperties":false,"properties":{
                "op":{"const":"delete"},"id":{"type":"string"}
            }},
            {"type":"object","required":["op","width","height"],"additionalProperties":false,"properties":{
                "op":{"const":"resize_artboard"},"width":{"type":"integer","minimum":64,"maximum":8192},"height":{"type":"integer","minimum":64,"maximum":8192}
            }},
            {"type":"object","required":["op","name"],"additionalProperties":false,"properties":{
                "op":{"const":"create_group"},"name":{"type":"string","minLength":1,"maxLength":80},"parentId":{"type":["string","null"]}
            }},
            {"type":"object","required":["op","id","parentId"],"additionalProperties":false,"properties":{
                "op":{"const":"set_layer_parent"},"id":{"type":"string"},"parentId":{"type":["string","null"],"description":"Group ID, or null to move to the top level"}
            }},
            {"type":"object","required":["op","id","name"],"additionalProperties":false,"properties":{
                "op":{"const":"rename_group"},"id":{"type":"string"},"name":{"type":"string","minLength":1,"maxLength":80}
            }},
            {"type":"object","required":["op","id"],"additionalProperties":false,"properties":{
                "op":{"const":"ungroup"},"id":{"type":"string"}
            }}
        ]}}
    }
}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    revision: String,
}

fn history_tool(name: &'static str, description: &'static str, command: &'static str) -> Tool {
    Tool {
        name,
        description,
        annotations: None,
        input_schema: parse(REVISION_SCHEMA),
        execute: Box::new(move |input| {
            js::promise(async move {
                let result = match decode(input, "Provide a design revision.") {
                    Ok(Revision { revision }) => {
                        crate::change_design(
                            command,
                            &serde_json::to_string(&revision).expect("Serialize revision"),
                            Author::Agent,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                };
                tool_result(result)
            })
        }),
    }
}

pub async fn register_editor_tools() -> Result<bool, &'static JsError> {
    register(vec![
        Tool {
            name: "list_screens",
            description: "List the current project's screens, stable IDs, active screen and screen list revision.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations { read_only_hint: true }),
            execute: Box::new(|_| js::promise(async { tool_result(crate::screen_list()) })),
        },
        Tool {
            name: "edit_screens",
            description: "Create, name, duplicate, reorder or delete a screen at the revision from list_screens. Returns the updated list and the affected or next screen ID. Use open_screen to navigate.",
            input_schema: parse(SCREEN_EDIT_SCHEMA),
            annotations: None,
            execute: Box::new(|input| js::promise(async move {
                let result = match decode(input, "Provide a screen list revision and edit.") {
                    Ok(ScreenMutation { revision, edit }) => crate::change_screens(&revision, edit).await.map(|updated| {
                        serde_json::json!({ "revision": updated.catalog.revision, "screens": updated.catalog.screens, "screenId": updated.screen_id }).to_string()
                    }),
                    Err(error) => Err(error),
                };
                tool_result(result)
            })),
        },
        dashboard_tool("open_screen", "Open a screen by stable ID in this tab. Discover the editor's tools again after navigation.", SCREEN_OPEN_SCHEMA, false, open_screen),
        Tool {
            name: "show_workspace_panel",
            description: "Show the editor's File panel with screens and layers or the Assets panel with image and component placement.",
            input_schema: parse(PANEL_SCHEMA),
            annotations: None,
            execute: Box::new(|input| js::promise(async move {
                let result = decode(input, "Choose the file or assets panel.").map(|WorkspacePanel { panel }| {
                    let assets = matches!(panel, Panel::Assets);
                    crate::show_workspace_panel(assets);
                    if assets { r#"{"panel":"assets"}"#.into() } else { r#"{"panel":"file"}"#.into() }
                });
                tool_result(result)
            })),
        },
        Tool {
            name: "get_save_status",
            description: "Read the browser draft save state, durable revision, and any conflict recovery project.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations { read_only_hint: true }),
            execute: Box::new(|_| js::promise(async { tool_result(crate::save_status()) })),
        },
        Tool {
            name: "retry_draft_save",
            description: "Retry saving a design held in memory after a browser storage error.",
            input_schema: parse(READ_SCHEMA),
            annotations: None,
            execute: Box::new(|_| js::promise(async { tool_result(crate::retry_save().await) })),
        },
        Tool {
            name: "export_unsaved_project",
            description: "Export the current in-memory artboard after a failed save along with the other saved project screens. Keep the returned bundle before navigating and use import_project to restore it.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations { read_only_hint: true }),
            execute: Box::new(|_| js::promise(async { tool_result(crate::export_unsaved()) })),
        },
        Tool {
            name: "recover_checkpoint",
            description: "Copy this screen's previous saved snapshot into a separate project. Pass the current storageRevision from get_save_status.",
            input_schema: parse(REVISION_SCHEMA),
            annotations: None,
            execute: Box::new(|input| js::promise(async move {
                let result = match decode(input, "Provide the current storage revision.") {
                    Ok(Revision { revision }) => crate::recover_previous(&revision).await.and_then(|project| {
                        serde_json::to_string(&project).map_err(|error| error.to_string())
                    }),
                    Err(error) => Err(error),
                };
                tool_result(result)
            })),
        },
        history_tool("undo_edit", "Undo the latest transaction in this editor session.", "undo"),
        history_tool("redo_edit", "Redo the next transaction in this editor session.", "redo"),
        Tool {
            name: "apply_edits",
            description: "Apply up to 100 layer patches, image placements, artboard resizes, hierarchy changes or deletions atomically at the expected design revision. The batch is one undo step.",
            annotations: None,
            input_schema: parse(BATCH_SCHEMA),
            execute: Box::new(|input| js::promise(async move {
                let result = match decode::<Batch>(input, "Provide valid edits and a design revision.") {
                    Ok(batch) => crate::apply_batch(batch, Author::Agent).await,
                    Err(error) => Err(error),
                };
                tool_result(result)
            })),
        },
        Tool {
            name: "list_components",
            description: "List the Canopy UI component families and variant IDs available for insertion.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations { read_only_hint: true }),
            execute: Box::new(|_| js::promise(async {
                tool_result(serde_json::to_string(crate::components::variants()).map_err(|error| error.to_string()))
            })),
        },
        Tool {
            name: "insert_component",
            description: "Insert a linked Canopy UI instance at an artboard position. Returns its layer IDs and the scene with instance metadata.",
            input_schema: parse(INSERT_SCHEMA),
            annotations: None,
            execute: Box::new(|input| js::promise(async move {
                let result = match decode(input, "Provide a component variant ID and integer x and y coordinates.") {
                    Ok(InsertComponent { id, x, y }) => crate::place_component(&id, x, y, Author::Agent).await,
                    Err(error) => Err(error),
                };
                tool_result(result)
            })),
        },
        Tool {
            name: "switch_component_variant",
            description: "Switch a linked instance to another variant in its component family and return the updated design.",
            input_schema: parse(SWITCH_SCHEMA),
            annotations: None,
            execute: Box::new(|input| js::promise(async move {
                let result = match decode(input, "Provide an instance ID and a variant ID.") {
                    Ok(SwitchVariant { instance_id, variant_id }) => crate::change_variant(instance_id, &variant_id, Author::Agent).await,
                    Err(error) => Err(error),
                };
                tool_result(result)
            })),
        },
        Tool {
            name: "read_layer_tree",
            description: "Read the ordered layer tree with stable IDs and nested group or linked instance children. Use those IDs in apply_edits to organize layers.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations { read_only_hint: true }),
            execute: Box::new(|_| js::promise(async { tool_result(crate::layer_tree()) })),
        },
        Tool {
            name: "read_design",
            description: "Read the artboard and its ordered layers.",
            input_schema: parse(READ_SCHEMA),
            annotations: Some(Annotations {
                read_only_hint: true,
            }),
            execute: Box::new(|_| {
                js::promise(async { response(crate::read_design().await).await })
            }),
        },
        Tool {
            name: "edit_layer",
            description: "Patch a layer by ID or omit ID to create a shape. Coordinates are artboard pixels.",
            input_schema: parse(EDIT_SCHEMA),
            annotations: None,
            execute: Box::new(|input| {
                js::promise(async move {
                    let body = input.and_then(|value| json::stringify(value).ok().flatten());
                    let result = match body {
                        Some(body) => crate::edit_design(body, Author::Agent).await,
                        None => Err("Specify valid shape properties.".into()),
                    };
                    response(result).await
                })
            }),
        },
    ]).await
}

async fn register(tools: Vec<Tool>) -> Result<bool, &'static JsError> {
    let Some(context) = document_context().or_else(navigator_context) else {
        return Ok(false);
    };
    for tool in tools {
        if let Some(registered) = context.register(tool)? {
            js::settle(registered).await?;
        }
    }
    Ok(true)
}

const CREATE_SCHEMA: &str = r#"{
    "type":"object","required":["title"],"additionalProperties":false,
    "properties":{"title":{"type":"string","minLength":1,"maxLength":80}}
}"#;
const OPEN_SCHEMA: &str = r#"{
    "type":"object","required":["id"],"additionalProperties":false,
    "properties":{"id":{"type":"string","description":"Project ID returned by list_projects or create_project"}}
}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateProject {
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenProject {
    id: String,
}

const IMPORT_SCHEMA: &str = r#"{
    "type":"object","required":["bundle"],"additionalProperties":false,
    "properties":{"bundle":{"type":"object","description":"A complete project from export_project"}}
}"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportProject {
    bundle: serde_json::Value,
}

fn export_project(input: Option<&Unknown>) -> Result<String, String> {
    let OpenProject { id } = decode(input, "Provide a project ID.")?;
    crate::backup::export(&id)
}

async fn import_project(input: Option<&Unknown>) -> Result<String, String> {
    let ImportProject { bundle } = decode(input, "Provide a project bundle.")?;
    let project = crate::backup::import(&bundle.to_string()).await?;
    crate::render_dashboard();
    serde_json::to_string(&project).map_err(|error| error.to_string())
}

pub async fn register_dashboard_tools() -> Result<bool, &'static JsError> {
    register(vec![
        dashboard_tool("export_project", "Export a portable project bundle with screens, links, and component definitions.", OPEN_SCHEMA, true, export_project),
        Tool {
            name: "import_project",
            description: "Validate a project bundle and restore it as a new project.",
            input_schema: parse(IMPORT_SCHEMA),
            annotations: None,
            execute: Box::new(|args| js::promise(async move { tool_result(import_project(args).await) })),
        },
        dashboard_tool("list_projects", "List projects in this browser with their stable IDs and titles.", READ_SCHEMA, true, list_projects),
        Tool {
            name: "create_project",
            description: "Create a named blank project and return its ID. Call open_project to open its editor.",
            input_schema: parse(CREATE_SCHEMA),
            annotations: None,
            execute: Box::new(|args| js::promise(async move { tool_result(create_project(args).await) })),
        },
        dashboard_tool("open_project", "Open a project by ID in this tab. After navigation, discover the editor's WebMCP tools.", OPEN_SCHEMA, false, open_project),
    ]).await
}

type DashboardAction = fn(Option<&'static Unknown>) -> Result<String, String>;

fn dashboard_tool(
    name: &'static str,
    description: &'static str,
    schema: &str,
    read_only: bool,
    action: DashboardAction,
) -> Tool {
    Tool {
        name,
        description,
        input_schema: parse(schema),
        annotations: read_only.then_some(Annotations {
            read_only_hint: true,
        }),
        execute: Box::new(move |args| js::promise(async move { tool_result(action(args)) })),
    }
}

fn tool_result(result: Result<String, String>) -> &'static Unknown {
    match result {
        Ok(body) => parse(&body),
        Err(message) => parse(
            &serde_json::json!({
                "isError": true,
                "content": [{ "type": "text", "text": message }]
            })
            .to_string(),
        ),
    }
}

fn list_projects(_: Option<&Unknown>) -> Result<String, String> {
    let (projects, warning) = projects::list();
    if let Some(error) = warning {
        return Err(error);
    }
    serde_json::to_string(&projects).map_err(|error| error.to_string())
}

async fn create_project(input: Option<&Unknown>) -> Result<String, String> {
    let CreateProject { title } = decode(input, "Provide a project title.")?;
    let project = projects::create(&title).await?;
    crate::render_dashboard();
    serde_json::to_string(&project).map_err(|error| error.to_string())
}

fn open_project(input: Option<&Unknown>) -> Result<String, String> {
    let OpenProject { id } = decode(input, "Provide a project ID.")?;
    let project = projects::find(&id).ok_or("Project not found in this browser.")?;
    let result = serde_json::to_string(&project).map_err(|error| error.to_string())?;
    webapi::window_global::location().assign(&project.href());
    Ok(result)
}

fn decode<T: serde::de::DeserializeOwned>(
    input: Option<&Unknown>,
    error: &str,
) -> Result<T, String> {
    let text = input
        .and_then(|value| json::stringify(value).ok().flatten())
        .ok_or(error)?;
    serde_json::from_str(&text).map_err(|_| error.into())
}

fn parse(text: &str) -> &'static Unknown {
    json::parse(text).expect("Valid JSON").expect("JSON object")
}

async fn response(result: Result<String, String>) -> &'static Unknown {
    match result {
        Ok(json) => parse(&json),
        Err(message) => promise::reject(Some(&js::error::new(Some(&message), None))).await,
    }
}
