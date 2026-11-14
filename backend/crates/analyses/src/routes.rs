//! HTTP routes under /api/analyses.
use crate::{examples, model::*, photos, store};
use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use common::{ApiResult, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create))
        .route("/examples", get(list_examples))
        .route("/{id}", get(load))
        .route("/{id}/photos/{photo}", get(photo))
}

async fn list_examples() -> Json<Vec<examples::ExampleSite>> { Json(examples::all()) }

async fn create(State(s): State<AppState>, Json(mut n): Json<NewAnalysis>) -> ApiResult<Json<AnalysisState>> {
    // An example site always uses its curated outline, whatever the client sent.
    if let Some(site) = n.example_site.as_deref().and_then(examples::get) {
        n.field = site.field.geom();
        n.label = Some(site.field.label.clone());
        n.location_mode = "example".into();
    }
    let id = store::create(&s.db, &n, None).await?;
    store::add_greeting(&s.db, &id, &n.lang).await?;
    Ok(Json(store::load(&s.db, &id).await?))
}

async fn load(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<AnalysisState>> {
    Ok(Json(store::load(&s.db, &id).await?))
}

async fn photo(State(s): State<AppState>, Path((id, photo)): Path<(String, i64)>) -> ApiResult<impl IntoResponse> {
    let (mime, bytes) = photos::read(&s, &id, photo).await?;
    Ok(([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "private, max-age=86400".into())], bytes))
}
