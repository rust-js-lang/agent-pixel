use js::{JsError, Promise};

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#requestWorker")]
pub fn request_worker(command: &str, json: &str) -> Promise<Result<String, &'static JsError>> {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#workspaceQuery")]
pub fn workspace_query(name: &str) -> bool {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#loadDraft")]
pub fn load_draft(key: &str) -> Result<Option<String>, &'static JsError> {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#saveDraft")]
pub fn save_draft(key: &str, value: Option<&str>) -> Result<(), &'static JsError> {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#documentRequest")]
pub fn document_request(body: Option<&str>) -> Promise<Result<HttpResponse, &'static JsError>> {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "../../site/browser.js#registerTools")]
pub fn register_tools(
    schema: &str,
    read: fn() -> Promise<Result<String, String>>,
    edit: fn(String) -> Promise<Result<String, String>>,
) -> Promise<Result<bool, &'static JsError>> {
    unreachable!()
}
