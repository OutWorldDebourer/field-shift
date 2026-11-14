//! NASA POWER service: daily series 1981 → latest for a point, 40-year normals (1985-2024), agro indicators
//! (anomalies, heat and frost days, rain extremes, FAO-56 ET0, water balance, GDD) and pest generations.
//! Every returned value carries source, dataset, period, resolution, the "regional context" badge, license and method.
pub mod client;
pub mod days;
pub mod et0;
pub mod gdd;
pub mod indicators;
pub mod meta;
pub mod normals;
pub mod pests;
mod routes;
pub mod stats;
pub mod summary;
pub mod waterbalance;

use axum::{routing::get, Router};
use chrono::{NaiveDate, Utc};
use common::AppState;

pub use client::PowerDaily;
pub use indicators::Options;
pub use summary::{MonthlyResponse, PowerSummary, SeriesResponse};

/// Cache freshness for the daily series (POWER updates daily).
pub const CACHE_TTL_HOURS: i64 = 24;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/summary", get(routes::summary))
        .route("/series", get(routes::series))
        .route("/monthly", get(routes::monthly))
}

pub fn cache_key(lat: f64, lon: f64) -> String {
    format!("power:daily:v1:{:.2}:{:.2}", client::round_coord(lat), client::round_coord(lon))
}

/// Daily series for a point, from `source_cache` when younger than 24 h, otherwise from POWER (stale copy on failure).
/// If the database itself is unreachable, falls back to a direct POWER request.
pub async fn daily(state: &AppState, lat: f64, lon: f64) -> anyhow::Result<PowerDaily> {
    validate(lat, lon)?;
    let end = Utc::now().date_naive();
    let key = cache_key(lat, lon);
    let http = state.http.clone();
    match common::cache::get_or_fetch(&state.db, "power", &key, CACHE_TTL_HOURS, || async move {
        client::fetch_daily(&http, lat, lon, end).await
    }).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::warn!("power cache path failed ({e}); fetching POWER directly");
            client::fetch_daily(&state.http, lat, lon, end).await
        }
    }
}

/// Indicators for the last 30 / 90 days and the season to date vs the 1985-2024 normal, plus pest generations.
pub async fn summary(state: &AppState, lat: f64, lon: f64, opts: &Options) -> anyhow::Result<PowerSummary> {
    summary::build_summary(&daily(state, lat, lon).await?, opts)
}

/// Daily rows with ET0 and day-of-year normals for charts and CSV.
pub async fn series(state: &AppState, lat: f64, lon: f64, from: Option<NaiveDate>, to: Option<NaiveDate>) -> anyhow::Result<SeriesResponse> {
    summary::build_series(&daily(state, lat, lon).await?, from, to)
}

/// The last 12 months, each with its indicators and the 1985-2024 normal band for the same dates.
pub async fn monthly(state: &AppState, lat: f64, lon: f64, opts: &Options) -> anyhow::Result<MonthlyResponse> {
    summary::build_monthly(&daily(state, lat, lon).await?, opts)
}

pub fn validate(lat: f64, lon: f64) -> anyhow::Result<()> {
    anyhow::ensure!(lat.is_finite() && (-90.0..=90.0).contains(&lat), "lat must be within -90..90");
    anyhow::ensure!(lon.is_finite() && (-180.0..=180.0).contains(&lon), "lon must be within -180..180");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_and_validation() {
        assert_eq!(cache_key(-11.4712, -77.2149), "power:daily:v1:-11.47:-77.21");
        assert!(validate(-11.47, -77.21).is_ok());
        assert!(validate(91.0, 0.0).is_err());
    }

    /// Live POWER request for the Huaral pilot (bypasses the DB cache). Run with `cargo test -p power -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn live_huaral() {
        let http = reqwest::Client::builder().user_agent("FieldShift/0.1 (NASA Space Apps 2026)").build().unwrap();
        let t0 = std::time::Instant::now();
        let d = client::fetch_daily(&http, -11.47, -77.21, Utc::now().date_naive()).await.unwrap();
        let fetch_s = t0.elapsed().as_secs_f64();
        assert!(d.len() > 16_000, "{} days", d.len());
        assert!((d.elevation_m - 224.43).abs() < 5.0, "elevation {}", d.elevation_m);
        let s = summary::build_summary(&d, &Options::default()).unwrap();
        let json = serde_json::to_string(&d).unwrap();
        println!("fetch {fetch_s:.1}s, {} days, cache payload {} KB, data through {}", d.len(), json.len() / 1024, s.data_through);
        println!("rain p95 {:?} mm from {} wet days", s.rain_p95_mm, s.wet_days_in_normal);
        for p in &s.periods {
            println!("== {} {}..{} ({} days, {} complete)", p.name, p.from, p.to, p.days, p.complete_days);
            for m in &p.metrics {
                println!("  {:<18} {:>9?} normal {:>9?} anomaly {:?}", m.key, m.value, m.normal.as_ref().map(|n| n.mean), m.anomaly);
            }
        }
        for g in &s.pests {
            println!("pest {:<22} dd {:?} gens {:?} normal {:?} diff {:?}", g.pest.key, g.degree_days, g.generations,
                     g.normal_generations.as_ref().map(|n| n.mean), g.difference);
        }
        let et0 = s.periods[0].metrics.iter().find(|m| m.key == "et0_mm").unwrap().value.unwrap();
        assert!(et0 > 30.0 && et0 < 200.0, "30-day ET0 {et0}");
        let m = summary::build_monthly(&d, &Options::default()).unwrap();
        assert_eq!(m.months.len(), 12);
        let ser = summary::build_series(&d, None, None).unwrap();
        assert_eq!(ser.rows.len(), 365);
        std::fs::write(std::env::temp_dir().join("power_summary_huaral.json"), serde_json::to_string_pretty(&s).unwrap()).unwrap();
    }
}
