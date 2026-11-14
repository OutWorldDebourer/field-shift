//! Earthdata service: SMAP L4 soil moisture, IMERG Late daily rain and GRACE/GRACE-FO water storage at a point.
//! Routes: GET /smap?lat=&lon=[&days=], /imerg?lat=&lon=[&days=], /grace?lat=&lon=
pub mod client;
pub mod ctx;
pub mod grace;
pub mod grid;
pub mod imerg;
pub mod meta;
mod routes;
pub mod smap;

use axum::{routing::get, Router};
use common::AppState;

pub use grace::GraceReport;
pub use imerg::ImergReport;
pub use meta::Provenance;
pub use smap::SmapReport;

pub const SMAP_DEFAULT_DAYS: u32 = 14;
pub const IMERG_DEFAULT_DAYS: u32 = 60;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/smap", get(routes::smap))
        .route("/imerg", get(routes::imerg))
        .route("/grace", get(routes::grace))
}

pub fn validate(lat: f64, lon: f64) -> anyhow::Result<()> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        anyhow::bail!("lat must be within -90..90 and lon within -180..180");
    }
    Ok(())
}

/// SMAP L4 report (latest, daily series of `days` ≤ 30, same-date comparison), cached 3 h per cell.
pub async fn smap(state: &AppState, lat: f64, lon: f64, days: u32) -> anyhow::Result<SmapReport> {
    let ctx = ctx::Ctx::from_state(state)?;
    let cell = grid::ease9_cell(lat, lon).ok_or_else(|| anyhow::anyhow!("point outside the SMAP grid"))?;
    let key = format!("earthdata:smap-report:{}:{}:{}", cell.row, cell.col, days.clamp(1, 30));
    let c = ctx.clone();
    ctx.cached("smap", &key, 3, || async move { smap::report(&c, lat, lon, days).await }).await
}

/// IMERG Late daily rain for the last `days` ≤ 90 days, cached 3 h per cell.
pub async fn imerg(state: &AppState, lat: f64, lon: f64, days: u32) -> anyhow::Result<ImergReport> {
    let ctx = ctx::Ctx::from_state(state)?;
    let (i, j) = grid::imerg_index(lat, lon).ok_or_else(|| anyhow::anyhow!("point outside the IMERG grid"))?;
    let key = format!("earthdata:imerg-report:{i}:{j}:{}", days.clamp(1, 90));
    let c = ctx.clone();
    ctx.cached("imerg", &key, 3, || async move { imerg::report(&c, lat, lon, days).await }).await
}

/// GRACE series from the example-site files in `<root>/backend/data`, or a gap for other points.
pub fn grace(state: &AppState, lat: f64, lon: f64) -> anyhow::Result<GraceReport> {
    grace::report_from_dir(&state.config.root.join("backend").join("data"), lat, lon)
}
