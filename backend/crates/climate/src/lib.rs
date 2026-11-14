//! Climate service: NEX-GDDP-CMIP6 projection (~2040 vs 1995-2014) at a point, from precomputed example-site files.
//! Route: GET /projection?lat=&lon=
pub mod projection;

use axum::{extract::{Query, State}, routing::get, Json, Router};
use common::{ApiError, ApiResult, AppState};
use serde::Deserialize;

pub use projection::{ProjectionFile, ProjectionReport};

pub fn routes() -> Router<AppState> {
    Router::new().route("/projection", get(projection_handler))
}

/// Projection for a point from `<root>/backend/data/projection_*.json`, or a gap.
pub fn projection(state: &AppState, lat: f64, lon: f64) -> anyhow::Result<ProjectionReport> {
    projection::report_from_dir(&state.config.root.join("backend").join("data"), lat, lon)
}

#[derive(Deserialize)]
struct PointQuery { lat: f64, lon: f64 }

async fn projection_handler(State(st): State<AppState>, Query(q): Query<PointQuery>) -> ApiResult<Json<ProjectionReport>> {
    if !(-90.0..=90.0).contains(&q.lat) || !(-180.0..=180.0).contains(&q.lon) {
        return Err(ApiError::BadRequest("lat must be within -90..90 and lon within -180..180".into()));
    }
    Ok(Json(projection(&st, q.lat, q.lon).map_err(ApiError::Other)?))
}
