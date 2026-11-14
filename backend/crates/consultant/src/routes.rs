//! POST /api/consultant/{id}/message (multipart: text, lang, demo, photos[] up to 10 images).
use crate::{ratelimit, turn::{run_turn, NewPhoto, TurnInput}};
use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Multipart, Path, State},
    http::HeaderMap,
    routing::post,
    Json, Router,
};
use common::{ApiError, ApiResult, AppState};
use std::net::SocketAddr;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/message", post(message))
        .layer(DefaultBodyLimit::max(60 * 1024 * 1024))
}

async fn message(
    State(s): State<AppState>,
    Path(id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut mp: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    // Behind the local reverse proxy the client address arrives in X-Forwarded-For.
    let ip = if addr.ip().is_loopback() {
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).and_then(|v| v.split(',').next()).map(|v| v.trim().to_string())
            .unwrap_or_else(|| addr.ip().to_string())
    } else { addr.ip().to_string() };
    let mut input = TurnInput { text: String::new(), lang: "en".into(), photos: vec![], visitor: ratelimit::visitor_id(&ip), force_demo: false };
    while let Some(field) = mp.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "text" => input.text = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?,
            "lang" => input.lang = if field.text().await.unwrap_or_default() == "es" { "es".into() } else { "en".into() },
            "demo" => input.force_demo = field.text().await.unwrap_or_default() == "1",
            "photos" | "photos[]" => {
                let mime = field.content_type().unwrap_or("application/octet-stream").to_string();
                let bytes = field.bytes().await.map_err(|e| ApiError::BadRequest(e.to_string()))?.to_vec();
                input.photos.push(NewPhoto { mime, bytes });
            }
            _ => {}
        }
    }
    let out = run_turn(&s, &id, input).await?;
    Ok(Json(serde_json::to_value(out).map_err(|e| ApiError::Other(e.into()))?))
}
