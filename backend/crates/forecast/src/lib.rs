//! NASA GEOS-FP weather forecast (GMAO) for one point: up to 10 days of daily min/max/mean 2 m temperature and rain.
//! Read anonymously from the NCCS OPeNDAP server, one 48-hour block per request (longer ranges time out there),
//! cached per cell and model run. Regional context (~25 km cell); a forecast, never a measurement.
mod geosfp;

use axum::{extract::{Query, State}, routing::get, Json, Router};
use common::{ApiError, ApiResult, AppState};
use serde::{Deserialize, Serialize};

pub use geosfp::{daily, Day};

pub fn routes() -> Router<AppState> { Router::new().route("/", get(route)) }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Forecast {
    pub source: String,
    pub run: String,
    pub resolution: String,
    pub badge: String,
    pub license: String,
    pub method: String,
    pub cell: [f64; 2],
    pub days: Vec<Day>,
}

/// Forecast for the next `days` days (1–10), cached ~6 h per cell and run.
pub async fn forecast(state: &AppState, lat: f64, lon: f64, days: usize) -> anyhow::Result<Forecast> {
    let days = days.clamp(1, 10);
    let run = geosfp::latest_full_run(&state.http).await?;
    let (i, j) = geosfp::index(lat, lon);
    let key = format!("geosfp:v1:{run}:{i}:{j}");
    let full: Forecast = common::cache::get_or_fetch(&state.db, "geos-fp", &key, 6, || async {
        let d = geosfp::daily(&state.http, &run, i, j, lon).await?;
        Ok(Forecast {
            source: "NASA GEOS-FP (GMAO)".into(),
            run: geosfp::run_label(&run),
            resolution: "0.25° × 0.3125° (~25 km)".into(),
            badge: "regional context".into(),
            license: "NASA GMAO GEOS-FP data, no restrictions on use; cite NASA GMAO".into(),
            method: "Hourly time-averaged 2 m temperature (tavg1_2d_slv_Nx) and total precipitation (tavg1_2d_flx_Nx) of the latest 00 UTC run, grouped into local solar days".into(),
            cell: geosfp::center(i, j),
            days: d,
        })
    }).await?;
    Ok(Forecast { days: full.days.into_iter().take(days).collect(), ..full })
}

#[derive(Deserialize)]
struct Q { lat: f64, lon: f64, days: Option<usize> }

async fn route(State(s): State<AppState>, Query(q): Query<Q>) -> ApiResult<Json<Forecast>> {
    if q.lat.abs() > 90.0 || q.lon.abs() > 180.0 { return Err(ApiError::BadRequest("coordinates out of range".into())); }
    Ok(Json(forecast(&s, q.lat, q.lon, q.days.unwrap_or(10)).await.map_err(|e| ApiError::Upstream(e.to_string()))?))
}
