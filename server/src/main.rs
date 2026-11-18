use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use pixel_geometry::fits_board;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::ErrorKind, sync::Arc};
use tokio::{fs, sync::Mutex};
use tower_http::services::ServeDir;

#[derive(Clone, Deserialize, Serialize)]
struct Layer {
    id: String,
    kind: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fill: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Scene {
    version: u8,
    layers: Vec<Layer>,
}

impl Scene {
    fn valid(&self) -> bool {
        let mut ids = HashSet::new();
        self.version == 1
            && self.layers.len() <= 1000
            && self.layers.iter().all(|l| {
                (1..=64).contains(&l.id.len())
                    && l.id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    && ids.insert(&l.id)
                    && matches!(l.kind.as_str(), "rectangle" | "ellipse")
                    && fits_board(l.x, l.y, l.width, l.height)
                    && l.fill.len() == 7
                    && l.fill.starts_with('#')
                    && l.fill.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            })
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Document {
    revision: u64,
    scene: Option<Scene>,
}

type Store = Arc<Mutex<Document>>;
async fn read(State(store): State<Store>) -> Json<Document> {
    Json(store.lock().await.clone())
}
async fn save(
    State(store): State<Store>,
    Json(mut next): Json<Document>,
) -> Result<Json<Document>, StatusCode> {
    if !next.scene.as_ref().is_some_and(Scene::valid) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let mut current = store.lock().await;
    if current.revision != next.revision {
        return Err(StatusCode::CONFLICT);
    }
    next.revision += 1;
    let bytes = serde_json::to_vec(&next).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write("data/document.tmp", bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::rename("data/document.tmp", "data/document.json")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *current = next.clone();
    Ok(Json(next))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AGENT_PIXEL_PORT")
        .unwrap_or_else(|_| "4173".into())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    fs::create_dir_all("data").await?;
    let document = match fs::read("data/document.json").await {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(e) if e.kind() == ErrorKind::NotFound => Document::default(),
        Err(e) => return Err(e.into()),
    };
    let app = Router::new()
        .route("/api/document", get(read).put(save))
        .fallback_service(ServeDir::new("dist"))
        .with_state(Arc::new(Mutex::new(document)));
    println!("Open http://127.0.0.1:{port}/?server");
    axum::serve(listener, app).await?;
    Ok(())
}
