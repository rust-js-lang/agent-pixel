use js::JsError;
use webapi::{BodyInit, RequestInit, global::fetch, headers, url_search_params, window_global};

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub fn workspace_query(name: &str) -> bool {
    let search = window_global::location().search();
    url_search_params::new(Some(&search)).has(name, None)
}

pub fn query_value(name: &str) -> Option<String> {
    let search = window_global::location().search();
    url_search_params::new(Some(&search)).get(name)
}

pub fn load_draft(key: &str) -> Result<Option<String>, &'static JsError> {
    Ok(window_global::local_storage()?.get_item(key))
}

pub fn save_draft(key: &str, body: Option<&str>) -> Result<(), &'static JsError> {
    let storage = window_global::local_storage()?;
    match body {
        Some(body) => storage.set_item(key, body),
        None => {
            storage.remove_item(key);
            Ok(())
        }
    }
}

pub async fn document_request(body: Option<&str>) -> Result<HttpResponse, &'static JsError> {
    let options = body.map(|body| {
        let headers = headers::new(None);
        headers.append("Content-Type", "application/json");
        RequestInit {
            method: Some("PUT"),
            headers: Some(headers),
            body: Some(Some(BodyInit::Str(body))),
            ..RequestInit::default()
        }
    });
    let response = js::settle(fetch("./api/document", options)).await?;
    Ok(HttpResponse {
        status: response.status() as u16,
        body: js::settle(response.text()).await?,
    })
}
