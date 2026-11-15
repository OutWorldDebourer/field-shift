//! Pests in the evidence package: the field's own crops' pests (current first, then previous and planned crops),
//! each with generations from its published degree-day threshold where one exists (qualitative otherwise). The
//! three POWER reference pests (aphid, whitefly, armyworm) are used only while no crop is known, and are labelled
//! "general reference, not your crop". Also builds the `pest_gen` reading, named after the crop's leading pest.
use crate::reading::{bi, round, Reading};
use analyses::AnalysisState;
use common::AppState;
use crops::CropRecord;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Role of a crop in this analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role { Current, Previous, Next, Proposal }

impl Role {
    fn as_str(self) -> &'static str { match self { Role::Current => "current", Role::Previous => "previous", Role::Next => "next", Role::Proposal => "proposal" } }
}

/// The crops of the analysis: current, previous (as stored) and the planned next crop when it is in the seed
/// knowledge base (no AI research is started for it here).
pub async fn crops_of(state: &AppState, st: &AnalysisState) -> Vec<(CropRecord, Role)> {
    let mut out: Vec<(CropRecord, Role)> = vec![];
    let push = |r: CropRecord, role: Role, out: &mut Vec<(CropRecord, Role)>| { if !out.iter().any(|(c, _)| c.key == r.key) { out.push((r, role)); } };
    if let Some(k) = st.crop_current.as_ref().and_then(|c| c["key"].as_str()) {
        if let Ok(r) = crops::resolve(state, k).await { push(r, Role::Current, &mut out); }
    }
    for p in &st.crop_history {
        if let Some(k) = p["key"].as_str() { if let Ok(r) = crops::resolve(state, k).await { push(r, Role::Previous, &mut out); } }
    }
    if let Some(n) = st.facts.as_ref().and_then(|f| f["next_crop"].as_str()).filter(|s| !s.trim().is_empty()) {
        if let Some(m) = crops::normalize::best(n, crops::seed::all()) {
            if let Some(r) = crops::seed::get(&m.key) { push(r.clone(), Role::Next, &mut out); }
        }
    }
    if let Some(k) = st.facts.as_ref().and_then(|f| f["proposal_crop"].as_str()) {
        if let Ok(r) = crops::resolve(state, k).await { push(r, Role::Proposal, &mut out); }
    }
    out
}

fn first(v: &[String]) -> String { v.first().cloned().unwrap_or_default() }

fn cite_str(c: &crops::model::Citation) -> String {
    format!("{} ({}). {}. {}", c.author, c.year.map(|y| y.to_string()).unwrap_or_else(|| "n.d.".into()), c.title, c.url)
}

/// Pest rows for the package. With no crop known, the reference rows from the POWER summary.
pub async fn rows(state: &AppState, lat: f64, lon: f64, crops: &[(CropRecord, Role)], power_summary: Option<&Value>) -> Vec<Value> {
    if crops.is_empty() { return power_summary.map(reference_rows).unwrap_or_default(); }
    let mut out = vec![];
    let mut seen = HashSet::new();
    for (c, role) in crops {
        let th: Vec<Value> = crops::advice::pest_thresholds(c).iter().filter_map(|t| serde_json::to_value(t).ok()).collect();
        let gens = crate::crop_pests::generations(state, lat, lon, &th).await.unwrap_or_else(|e| { tracing::warn!("crop pests {}: {e}", c.key); vec![] });
        for p in &c.traits.pests {
            if !seen.insert(format!("{}|{}", c.key, p.scientific_name)) { continue; }
            let g = gens.iter().find(|g| g["scientific"].as_str() == Some(p.scientific_name.as_str()));
            let dd_cit = p.degree_days_per_generation.citation().map(cite_str);
            out.push(json!({
                "name": p.name, "name_bi": p.name, "scientific": p.scientific_name,
                "crop": { "key": c.key, "name": { "en": first(&c.names.en), "es": first(&c.names.es) } },
                "role": role.as_str(), "reference": false, "qualitative": g.is_none(),
                "generations_now": g.and_then(|g| g["generations_now"].as_f64()),
                "generations_normal": g.and_then(|g| g["generations_normal"].as_f64()),
                "difference": g.and_then(|g| g["difference"].as_f64()),
                "lower_threshold_c": g.and_then(|g| g["lower_threshold_c"].as_f64()),
                "degree_days": g.and_then(|g| g["degree_days"].as_f64()),
                "season": g.map(|g| g["season"].clone()).unwrap_or(Value::Null),
                "citation": dd_cit.unwrap_or_else(|| cite_str(&p.citation)),
                "source": { "title": p.citation.title, "url": p.citation.url },
            }));
        }
    }
    out
}

/// The three general reference pests from the NASA POWER summary, labelled as not specific to the field's crop.
fn reference_rows(s: &Value) -> Vec<Value> {
    s["pests"].as_array().into_iter().flatten().map(|p| {
        let c = &p["pest"]["citation"];
        json!({
            "name": { "en": p["pest"]["common_name"], "es": power::pests::name_es(p["pest"]["key"].as_str().unwrap_or("")) },
            "name_bi": { "en": p["pest"]["common_name"], "es": power::pests::name_es(p["pest"]["key"].as_str().unwrap_or("")) },
            "scientific": p["pest"]["scientific_name"], "crop": Value::Null, "role": "reference", "reference": true, "qualitative": false,
            "generations_now": p["generations"].as_f64().map(|x| round(x, 1)),
            "generations_normal": p["normal_generations"]["mean"].as_f64().map(|x| round(x, 1)),
            "difference": p["generations"].as_f64().zip(p["normal_generations"]["mean"].as_f64()).map(|(a, b)| round(round(a, 1) - round(b, 1), 1)),
            "lower_threshold_c": p["pest"]["lower_c"], "degree_days": p["pest"]["dd_per_generation"],
            "season": p["meta"]["period"].as_str().map(|x| x.replace("..", " – ")),
            "citation": format!("{} ({}). {}. {}. {}", c["authors"].as_str().unwrap_or(""), c["year"], c["title"].as_str().unwrap_or(""), c["published_in"].as_str().unwrap_or(""), c["url"].as_str().unwrap_or("")),
            "source": { "title": c["title"], "url": c["url"] },
            "note": { "en": "General reference, not your crop", "es": "Referencia general, no tu cultivo" },
        })
    }).collect()
}

/// The leading pest: the current crop's computable pest with the largest rise over normal (reference rows only
/// when no crop is known).
pub fn leading(rows: &[Value]) -> Option<&Value> {
    let computable = |p: &&Value| p["generations_now"].as_f64().is_some() && p["generations_normal"].as_f64().is_some();
    let diff = |p: &&Value| p["difference"].as_f64().unwrap_or(f64::MIN);
    let pick = |want: &str| rows.iter().filter(|p| p["role"] == want).filter(computable)
        .max_by(|a, b| diff(a).partial_cmp(&diff(b)).unwrap_or(std::cmp::Ordering::Equal));
    pick("current").or_else(|| if rows.iter().any(|p| p["reference"] == false) { None } else { pick("reference") })
}

/// The `pest_gen` reading, named after the leading pest.
pub fn reading(rows: &[Value]) -> Option<Reading> {
    let p = leading(rows)?;
    let reference = p["reference"] == true;
    let (en, es) = (p["name"]["en"].as_str().unwrap_or("Pest").to_string(), p["name"]["es"].as_str().unwrap_or("Plaga").to_string());
    let d = p["difference"].as_f64().unwrap_or(0.0);
    let label = if reference {
        bi("Pest generations (general reference, not your crop)", "Generaciones de plaga (referencia general, no tu cultivo)")
    } else {
        bi(format!("{en} generations this season"), format!("Generaciones de {} esta campaña", es.to_lowercase()))
    };
    let verdict = if d >= 0.5 { bi(format!("about {d:.1} more generations than normal"), format!("unas {d:.1} generaciones más que lo normal")) }
        else if d <= -0.5 { bi(format!("about {:.1} fewer generations than normal", -d), format!("unas {:.1} generaciones menos que lo normal", -d)) }
        else { bi("about as many generations as normal", "generaciones similares a lo normal") };
    Some(Reading {
        id: "pest_gen".into(), label, value: p["generations_now"].as_f64(), unit: "generations".into(), verdict, key: false,
        source: "NASA POWER + published threshold".into(), dataset: "POWER daily T2M_MIN / T2M_MAX".into(),
        date: p["season"].as_str().unwrap_or("").to_string(), resolution: "~0.5° × 0.625°".into(), badge: "regional context".into(),
        license: "NASA open data".into(),
        method: Some(format!("Single-sine degree-days, {} °C lower threshold, {} °C·day per generation. {}",
            p["lower_threshold_c"], p["degree_days"], p["citation"].as_str().unwrap_or(""))),
        series: vec![], provisional: false,
        note: reference.then(|| bi(format!("{en} model; tell the consultant your crop to see its own pests"), format!("modelo de {}; dile al consultor tu cultivo para ver sus propias plagas", es.to_lowercase()))),
    })
}

/// Pest models for the season outlook: the current crop's computable pests (reference pests only without a crop).
fn outlook_models(rows: &[Value]) -> Vec<power::outlook::PestModel> {
    if !rows.is_empty() && rows.iter().all(|p| p["reference"] == true) { return power::outlook::reference_models(); }
    rows.iter().filter(|p| p["role"] == "current").filter_map(|p| Some(power::outlook::PestModel {
        name: power::outlook::Bi { en: p["name"]["en"].as_str()?.into(), es: p["name"]["es"].as_str()?.into() },
        scientific: p["scientific"].as_str().unwrap_or("").into(),
        lower_c: p["lower_threshold_c"].as_f64()?, upper_c: None, dd_per_generation: p["degree_days"].as_f64()?,
    })).collect()
}

/// Puts `rows`, the matching `pest_gen` reading and the outlook's pest projection into a package.
pub async fn apply(state: &AppState, lat: f64, lon: f64, ev: &mut Value, rows: Vec<Value>) {
    if ev["outlook"].is_object() {
        match power::outlook_with(state, lat, lon, 3, &outlook_models(&rows)).await {
            Ok(o) => ev["outlook"]["pests"] = serde_json::to_value(o.pests).unwrap_or(json!([])),
            Err(e) => tracing::warn!("outlook pests: {e}"),
        }
    }
    apply_rows(ev, rows);
}

/// Puts `rows` and the matching `pest_gen` reading into a package (replacing earlier ones).
pub fn apply_rows(ev: &mut Value, rows: Vec<Value>) {
    let r = reading(&rows).and_then(|r| serde_json::to_value(r).ok());
    if let Some(list) = ev["readings"].as_array_mut() {
        let pos = list.iter().position(|x| x["id"] == "pest_gen");
        if let Some(i) = pos { list.remove(i); }
        if let Some(r) = r { list.insert(pos.unwrap_or(list.len()).min(list.len()), r); }
    }
    ev["pests"] = json!(rows);
}

/// Recomputes the stored package's pests for the analysis's current crops (after the crops change).
pub async fn refresh(state: &AppState, analysis_id: &str) -> anyhow::Result<Option<Value>> {
    let st = analyses::store::load(&state.db, analysis_id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let Some(mut ev) = st.evidence.clone().filter(|e| e["readings"].is_array()) else { return Ok(None) };
    let crops = crops_of(state, &st).await;
    let summary = if crops.is_empty() { power::summary(state, st.field.center.lat, st.field.center.lon, &power::Options::default()).await.ok().and_then(|s| serde_json::to_value(s).ok()) } else { None };
    let rows = rows(state, st.field.center.lat, st.field.center.lon, &crops, summary.as_ref()).await;
    apply(state, st.field.center.lat, st.field.center.lon, &mut ev, rows).await;
    analyses::store::save_evidence(&state.db, analysis_id, &ev).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    // The Prevention lights follow the crops too (a crop named in the chat appears there at once).
    if !crops.is_empty() {
        let p = crate::prevention::build(&crops, &ev);
        analyses::store::set_section(&state.db, analysis_id, "prevention", "ready", None, Some(&p)).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    Ok(Some(ev))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(role: &str, en: &str, now: f64, norm: f64) -> Value {
        json!({ "name": { "en": en, "es": en }, "role": role, "reference": role == "reference", "generations_now": now, "generations_normal": norm, "difference": now - norm, "season": "2026-08-01 – 2026-11-10", "lower_threshold_c": 10.0, "degree_days": 100.0, "citation": "x" })
    }

    #[test]
    fn leading_is_the_current_crops_biggest_rise() {
        let rows = vec![row("current", "Pea aphid", 13.0, 10.0), row("current", "Leafminer", 5.0, 4.5), row("previous", "Looper", 9.0, 2.0)];
        assert_eq!(leading(&rows).unwrap()["name"]["en"], "Pea aphid");
        let r = reading(&rows).unwrap();
        assert_eq!(r.label.en, "Pea aphid generations this season");
        assert!(r.note.is_none());
    }

    #[test]
    fn reference_only_without_a_crop() {
        let refs = vec![row("reference", "Green peach aphid", 5.0, 4.0)];
        assert!(reading(&refs).unwrap().label.en.contains("not your crop"));
        // A known crop with only qualitative pests gets no tile, never the reference aphid.
        let mut q = row("current", "Avocado spider mite", 0.0, 0.0);
        q["generations_now"] = Value::Null;
        assert!(reading(&[q]).is_none());
    }

    #[test]
    fn avocado_and_grape_never_get_reference_pests() {
        for key in ["avocado", "grape"] {
            let c = crops::seed::get(key).unwrap();
            for p in &c.traits.pests {
                for bad in ["Myzus", "Bemisia", "Spodoptera"] { assert!(!p.scientific_name.contains(bad), "{key} lists {bad}"); }
            }
        }
    }
}
