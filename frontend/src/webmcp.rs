//! Browser bindings for the experimental WebMCP model context.
use js::{JsError, JsObject, Promise, Unknown, json, promise};
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

pub async fn register_tools() -> Result<bool, &'static JsError> {
    let Some(context) = document_context().or_else(navigator_context) else {
        return Ok(false);
    };
    for tool in [
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
                        Some(body) => crate::edit_design(body).await,
                        None => Err("Specify valid shape properties.".into()),
                    };
                    response(result).await
                })
            }),
        },
    ] {
        if let Some(registered) = context.register(tool)? {
            js::settle(registered).await?;
        }
    }
    Ok(true)
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
