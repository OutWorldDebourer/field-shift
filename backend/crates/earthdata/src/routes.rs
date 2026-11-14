//! HTTP handlers for /api/earthdata.
use axum::{extract::{Query, State}, Json};
use common::{ApiError, ApiResult, AppState};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PointQuery {
    lat: f64,
    lon: f64,
    days: Option<u32>,
}

fn check(q: &PointQuery) -> ApiResult<()> {
    crate::validate(q.lat, q.lon).map_err(|e| ApiError::BadRequest(e.to_string()))
}

fn upstream(e: anyhow::Error) -> ApiError { ApiError::Upstream(e.to_string()) }

pub async fn smap(State(st): State<AppState>, Query(q): Query<PointQuery>) -> ApiResult<Json<crate::SmapReport>> {
    check(&q)?;
    Ok(Json(crate::smap(&st, q.lat, q.lon, q.days.unwrap_or(crate::SMAP_DEFAULT_DAYS)).await.map_err(upstream)?))
}

pub async fn imerg(State(st): State<AppState>, Query(q): Query<PointQuery>) -> ApiResult<Json<crate::ImergReport>> {
    check(&q)?;
    Ok(Json(crate::imerg(&st, q.lat, q.lon, q.days.unwrap_or(crate::IMERG_DEFAULT_DAYS)).await.map_err(upstream)?))
}

pub async fn grace(State(st): State<AppState>, Query(q): Query<PointQuery>) -> ApiResult<Json<crate::GraceReport>> {
    check(&q)?;
    Ok(Json(crate::grace(&st, q.lat, q.lon).map_err(ApiError::Other)?))
}
