use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use pixel_geometry::scene::Document;
use std::{io::ErrorKind, sync::Arc};
use tokio::{fs, sync::Mutex};
use tower_http::services::ServeDir;

type Store = Arc<Mutex<Document>>;
async fn read(State(store): State<Store>) -> Json<Document> {
    Json(store.lock().await.clone())
}
async fn save(
    State(store): State<Store>,
    Json(mut next): Json<Document>,
) -> Result<Json<Document>, StatusCode> {
    if !next
        .scene
        .as_ref()
        .is_some_and(|scene| scene.validate().is_ok())
    {
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
