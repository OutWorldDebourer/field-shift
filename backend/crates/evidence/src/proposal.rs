//! "Evaluate your idea" (iteration 4, phase C): any crop the farmer proposes becomes a fourth rotation column,
//! "Your proposal", scored by the same engine on the same NASA signals (soil, water against the water balance,
//! pest-cycle break by botanical family, income, risk) with why, evidence, gaps and risks. The crop is resolved from
//! the knowledge base (AI research only for an unknown crop) and remembered in the facts as `proposal_crop`, so the
//! column is rebuilt whenever the advice is, and its pests join the analysis's crops.
use analyses::store;
use common::AppState;
use crops::{CropRecord, FieldSignals};
use serde_json::{json, Value};

fn bi_join(list: &Value) -> Value {
    let j = |l: &str| list.as_array().into_iter().flatten().filter_map(|x| x[l].as_str().or_else(|| x["text"][l].as_str())).collect::<Vec<_>>().join(" ");
    json!({ "en": j("en"), "es": j("es") })
}

/// A strategy of the advice engine in the shape the Rotation view renders.
pub fn strategy_json(s: &Value) -> Value {
    let sc = &s["scores"];
    json!({
        "name": s["title"], "current": s["is_current"],
        "timeline": s["sequence"].as_array().into_iter().flatten().filter_map(|x| x["en"].as_str()).collect::<Vec<_>>(),
        "timeline_es": s["sequence"].as_array().into_iter().flatten().filter_map(|x| x["es"].as_str()).collect::<Vec<_>>(),
        "scores": { "soil": sc["soil"], "water": sc["water"], "pest_break": sc["pest_break"], "income": sc["income"], "risk": sc["risk"] },
        "why": s["why"], "evidence": bi_join(&s["evidence"]), "gaps": bi_join(&s["gaps"]), "risks": bi_join(&s["risks"]),
        "evidence_list": s["evidence"].as_array().into_iter().flatten().map(|e| e["text"].clone()).collect::<Vec<_>>(),
        "gaps_list": s["gaps"].clone(), "risks_list": s["risks"].clone(),
        "citations": s["evidence"].as_array().into_iter().flatten().filter_map(|e| e.get("citation").filter(|c| !c.is_null()).map(|c| json!({ "title": c["title"], "url": c["url"] }))).collect::<Vec<_>>(),
    })
}

/// The proposal column for the crop remembered in the facts (None when the farmer has not proposed one).
pub async fn column(state: &AppState, facts: Option<&Value>, current: &CropRecord, previous: &[CropRecord], signals: &FieldSignals) -> Option<Value> {
    let key = facts?.get("proposal_crop")?.as_str()?;
    let cand = crops::resolve(state, key).await.ok()?;
    let s = serde_json::to_value(crops::advice::evaluate(current, previous, &cand, signals)).ok()?;
    let mut v = strategy_json(&s);
    v["proposal"] = json!(true);
    v["crop"] = json!({ "key": cand.key, "name": cand.name(), "crop_type": cand.crop_type, "family": cand.family, "origin": cand.origin });
    Some(v)
}

/// Resolves the farmer's crop words, remembers them and rebuilds the advice; returns the proposal column.
pub async fn evaluate(state: &AppState, id: &str, crop_text: &str) -> anyhow::Result<Value> {
    let st = store::load(&state.db, id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    if st.crop_current.is_none() { anyhow::bail!("tell the current crop first: the proposal is compared with it"); }
    let cand = crops::resolve(state, crop_text).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    store::merge_facts(&state.db, id, &json!({ "proposal_crop": cand.key })).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    crate::advise(state, id).await?;
    let st = store::load(&state.db, id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    st.sections.iter().find(|s| s.key == "rotation").and_then(|s| s.content.as_ref()).and_then(|c| c.get("proposal").cloned())
        .filter(|p| !p.is_null()).ok_or_else(|| anyhow::anyhow!("the proposal could not be scored"))
}
