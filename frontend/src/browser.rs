use js::{JsError, Promise, promise};
use pixel_geometry::assets::{MAX_IMAGE_BYTES, RasterImage};
use webapi::{
    BodyInit, EventTargetExt, File, RequestInit, StrOrArrayBuffer, events, file_reader,
    global::fetch, headers, tags, window_global,
};

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
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

pub fn download(href: &str, filename: &str) {
    let anchor = window_global::window().document().create_element(tags::A);
    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.click();
}

pub async fn read_image(file: &File) -> Result<RasterImage, String> {
    if file.size() > MAX_IMAGE_BYTES as f64 {
        return Err("Each image must be 512 KiB or smaller.".into());
    }
    let reader = file_reader::new();
    let ready: Promise<Result<String, String>> = promise::new(|resolve, _| {
        reader.add_event_listener(
            events::Loadend,
            move |_: &webapi::ProgressEvent| {
                resolve(match reader.result() {
                    Some(StrOrArrayBuffer::Str(data)) => Ok(data.to_owned()),
                    _ => Err("Could not read the image file.".into()),
                });
            },
            None,
        );
    });
    reader.read_as_data_url(file);
    Ok(RasterImage {
        name: file.name(),
        data_url: ready.await?,
    })
}

pub async fn image_dimensions(image: &RasterImage) -> Result<(f64, f64), String> {
    image.validate()?;
    let element = window_global::window().document().create_element(tags::Img);
    element.set_src(image.data_url.as_str());
    js::settle(element.decode())
        .await
        .map_err(|_| "The image could not be decoded.")?;
    let width = element.natural_width();
    let height = element.natural_height();
    if width <= 0.0 || height <= 0.0 || width * height > 16_000_000.0 {
        return Err("Image dimensions must be positive and at most 16 megapixels.".into());
    }
    Ok((width, height))
}
