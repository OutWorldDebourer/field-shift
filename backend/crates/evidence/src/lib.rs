//! Analyst stage: gathers NASA and catalog data into one fixed evidence package per analysis, resolves crops,
//! and fills the result sections from the cited advice engine. The consultant may only cite numbers from here.
//! (Interface first; the service calls are wired in once each data service lands.)
use axum::Router;
use common::AppState;
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> { Router::new() }

/// Builds (or refreshes) the evidence package for an analysis, stores it in `reports` and returns it.
pub async fn build(state: &AppState, analysis_id: &str) -> anyhow::Result<Value> {
    let field = analyses::store::field(&state.db, analysis_id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let ev = json!({
        "field": field, "generated_at": chrono::Utc::now().to_rfc3339(), "readings": [], "climate_monthly": [],
        "ndvi_series": [], "pests": [], "projection": null, "imagery": null,
        "gaps": [{ "id": "pending", "text": { "en": "Data services are still being connected.", "es": "Los servicios de datos aún se están conectando." } }]
    });
    analyses::store::save_evidence(&state.db, analysis_id, &ev).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(ev)
}

/// Normalizes the farmer's crop words into crop records and stores them on the analysis.
pub async fn resolve_crops(state: &AppState, analysis_id: &str, current: &str, previous: &[String]) -> anyhow::Result<Value> {
    let rec = |q: &str| json!({ "key": q.to_lowercase(), "names": { "en": q, "es": q }, "scientific": "", "family": "", "crop_type": "annual", "origin": "seed" });
    let cur = rec(current);
    let prev: Vec<Value> = previous.iter().map(|p| rec(p)).collect();
    analyses::store::set_crops(&state.db, analysis_id, Some(&cur), Some(&prev)).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(json!({ "current": cur, "previous": prev }))
}

/// Runs the advice engine on crops + evidence signals and writes the soil, pests, nutrition and rotation sections.
pub async fn advise(state: &AppState, analysis_id: &str) -> anyhow::Result<Value> {
    let _ = (state, analysis_id);
    Ok(json!({ "status": "advice engine not connected yet" }))
}
