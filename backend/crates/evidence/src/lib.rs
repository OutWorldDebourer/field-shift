//! Analyst stage: gathers NASA and catalog data into one fixed evidence package per analysis, resolves crops,
//! and fills the result sections from the cited advice engine. The consultant may only cite numbers from here.
mod advise;
mod from_imagery;
mod from_power;
pub mod reading;

use axum::Router;
use common::{geo::Field, AppState};
use reading::gap;
use serde_json::{json, Value};

pub use advise::{advise, resolve_crops};

pub fn routes() -> Router<AppState> { Router::new() }

fn to_value<T: serde::Serialize>(r: anyhow::Result<T>, what: &str, gaps: &mut Vec<Value>, en: &str, es: &str) -> Option<Value> {
    match r.and_then(|v| Ok(serde_json::to_value(v)?)) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!("evidence: {what} failed: {e}");
            gaps.push(gap(what, en, es));
            None
        }
    }
}

/// Builds (or refreshes) the evidence package for an analysis, stores it in `reports` and returns it.
pub async fn build(state: &AppState, analysis_id: &str) -> anyhow::Result<Value> {
    let fo = analyses::store::field(&state.db, analysis_id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let field: Field = fo.geom();
    let (lat, lon) = (field.center.lat, field.center.lon);
    let opts = power::Options::default();
    let size_m = from_imagery::snapshot_size_m(&field);

    let (summary, monthly, ndvi, terrain, snap) = tokio::join!(
        power::summary(state, lat, lon, &opts),
        power::monthly(state, lat, lon, &opts),
        imagery::ndvi_series(state, &field, 24),
        imagery::terrain(state, &field),
        imagery::snapshot(state, lat, lon, size_m, 1024),
    );
    let mut gaps = vec![];
    let summary = to_value(summary, "power", &mut gaps, "NASA POWER climate data could not be read right now.", "No se pudieron leer los datos climáticos de NASA POWER.");
    let monthly = to_value(monthly, "power_monthly", &mut gaps, "Monthly climate comparison unavailable.", "Comparación climática mensual no disponible.");
    let ndvi = to_value(ndvi, "ndvi", &mut gaps, "Satellite crop vigor (Sentinel-2 / Landsat) could not be read right now.", "No se pudo leer el vigor del cultivo satelital (Sentinel-2 / Landsat).");
    let terrain = to_value(terrain, "terrain", &mut gaps, "Slope (NASADEM) unavailable.", "Pendiente (NASADEM) no disponible.");
    let snap = match snap {
        Ok((_, meta)) => serde_json::to_value(meta).ok(),
        Err(e) => { tracing::warn!("evidence: snapshot failed: {e}"); None }
    };

    let mut readings = vec![];
    if let Some(s) = &summary { readings.extend(from_power::readings(s, monthly.as_ref(), from_power::provisional(s))); }
    readings.extend(from_imagery::readings(ndvi.as_ref(), terrain.as_ref()));
    if field.ring.len() < 3 {
        gaps.push(gap("outline", "No field outline drawn: satellite values use a ~110 m square around the point.", "Sin contorno del campo: los valores satelitales usan un cuadrado de ~110 m alrededor del punto."));
    }
    gaps.push(gap("soil_lab", "No soil laboratory analysis: nutrient advice is general, not a fertilizer prescription.", "Sin análisis de suelo de laboratorio: la nutrición es general, no una receta de fertilización."));

    let imagery_v = snap.map(|m| json!({
        "snapshot_url": format!("/api/imagery/snapshot.png?lat={lat}&lon={lon}&size_m={size_m}&px=1024"),
        "credit": m["credit"], "date": m["date"], "bbox": m["bbox"], "scene_id": m["scene_id"],
    }));
    let ev = json!({
        "field": fo,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "readings": readings,
        "climate_monthly": monthly.as_ref().map(from_power::climate_monthly).unwrap_or_default(),
        "ndvi_series": from_imagery::series(ndvi.as_ref()),
        "bare_soil": ndvi.as_ref().map(|n| n["bare_soil"].clone()).unwrap_or(json!([])),
        "cycles": ndvi.as_ref().map(|n| n["cycles"].clone()).unwrap_or(json!([])),
        "pests": summary.as_ref().map(from_power::pests).unwrap_or_default(),
        "projection": Value::Null,
        "imagery": imagery_v,
        "gaps": gaps,
        "power_notes": summary.as_ref().map(|s| s["notes"].clone()).unwrap_or(Value::Null),
    });
    analyses::store::save_evidence(&state.db, analysis_id, &ev).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    mark_field_section(state, analysis_id, &ev).await;
    Ok(ev)
}

/// The "Your field" section summarises the package so it is ready as soon as the evidence exists.
async fn mark_field_section(state: &AppState, id: &str, ev: &Value) {
    let n = ev["readings"].as_array().map(|r| r.len()).unwrap_or(0);
    let state_s = if n == 0 { "gaps" } else { "ready" };
    let content = json!({
        "title": { "en": "Your field · NASA data", "es": "Tu campo · datos NASA" },
        "summary": { "en": format!("{n} readings with source, date and scale. Regional readings describe the area, not the plot."),
                     "es": format!("{n} lecturas con fuente, fecha y escala. Las lecturas regionales describen la zona, no la parcela.") },
        "items": [], "readings": ev["readings"].as_array().map(|r| r.iter().filter_map(|x| x["id"].as_str()).collect::<Vec<_>>()).unwrap_or_default(),
    });
    let existing = analyses::store::sections(&state.db, id).await.ok();
    let declared = existing.as_ref().and_then(|s| s.iter().find(|x| x.key == "field")).and_then(|f| f.content.as_ref()).and_then(|c| c.get("declared_gaps").cloned());
    let mut content = content;
    if let Some(d) = declared { content["declared_gaps"] = d; }
    let _ = analyses::store::set_section(&state.db, id, "field", state_s, None, Some(&content)).await;
}
