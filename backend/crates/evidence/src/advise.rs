//! Crops and advice (placeholder until the crops crate lands): crop words → records; advice → sections.
use common::AppState;
use serde_json::{json, Value};

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
