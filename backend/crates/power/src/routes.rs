//! HTTP handlers: GET /summary, /series (JSON or `format=csv`), /monthly.
use crate::{indicators::Options, validate};
use axum::{extract::{Query, State}, http::header, response::{IntoResponse, Response}, Json};
use chrono::NaiveDate;
use common::{ApiError, ApiResult, AppState};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SummaryQuery {
    lat: f64,
    lon: f64,
    kc: Option<f64>,
    heat_c: Option<f64>,
    gdd_base: Option<f64>,
    /// YYYY-MM-DD
    season_start: Option<NaiveDate>,
}

impl SummaryQuery {
    fn options(&self) -> ApiResult<Options> {
        validate(self.lat, self.lon).map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let d = Options::default();
        let o = Options {
            kc: self.kc.unwrap_or(d.kc),
            heat_c: self.heat_c.unwrap_or(d.heat_c),
            gdd_base: self.gdd_base.unwrap_or(d.gdd_base),
            season_start: self.season_start,
        };
        if !(0.0..=2.0).contains(&o.kc) { return Err(ApiError::BadRequest("kc must be within 0..2".into())); }
        if !(-10.0..=60.0).contains(&o.heat_c) { return Err(ApiError::BadRequest("heat_c must be within -10..60".into())); }
        if !(-10.0..=40.0).contains(&o.gdd_base) { return Err(ApiError::BadRequest("gdd_base must be within -10..40".into())); }
        Ok(o)
    }
}

fn upstream(e: anyhow::Error) -> ApiError { ApiError::Upstream(e.to_string()) }

pub async fn summary(State(st): State<AppState>, Query(q): Query<SummaryQuery>) -> ApiResult<Json<crate::PowerSummary>> {
    let o = q.options()?;
    Ok(Json(crate::summary(&st, q.lat, q.lon, &o).await.map_err(upstream)?))
}

pub async fn monthly(State(st): State<AppState>, Query(q): Query<SummaryQuery>) -> ApiResult<Json<crate::MonthlyResponse>> {
    let o = q.options()?;
    Ok(Json(crate::monthly(&st, q.lat, q.lon, &o).await.map_err(upstream)?))
}

#[derive(Deserialize)]
pub struct SeriesQuery {
    lat: f64,
    lon: f64,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    /// "json" (default) or "csv".
    format: Option<String>,
}

pub async fn series(State(st): State<AppState>, Query(q): Query<SeriesQuery>) -> ApiResult<Response> {
    validate(q.lat, q.lon).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    if let (Some(f), Some(t)) = (q.from, q.to) {
        if f > t { return Err(ApiError::BadRequest("from must be on or before to".into())); }
    }
    let d = crate::daily(&st, q.lat, q.lon).await.map_err(upstream)?;
    let s = crate::summary::build_series(&d, q.from, q.to).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    if q.format.as_deref() == Some("csv") {
        let body = format!("# {} | {} | {} | {}\n{}", crate::meta::SOURCE, crate::meta::RESOLUTION, crate::meta::BADGE, crate::meta::LICENSE, crate::summary::series_csv(&s));
        return Ok(([(header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                    (header::CONTENT_DISPOSITION, "attachment; filename=\"power_daily.csv\"")], body).into_response());
    }
    Ok(Json(s).into_response())
}

#[derive(serde::Deserialize)]
pub struct OutlookQuery { lat: f64, lon: f64, months: Option<u32> }

pub async fn outlook(State(st): State<AppState>, Query(q): Query<OutlookQuery>) -> ApiResult<Json<crate::outlook::Outlook>> {
    crate::validate(q.lat, q.lon).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok(Json(crate::outlook(&st, q.lat, q.lon, q.months.unwrap_or(3)).await.map_err(|e| ApiError::Upstream(e.to_string()))?))
}
