//! Crops and advice: the farmer's crop words → crop records (any crop); the cited advice engine → result sections.
use analyses::{store, Bi as SBi};
use common::AppState;
use crops::{CropRecord, FieldSignals};
use serde_json::{json, Value};

/// Contract shape of a crop for the frontend and the analysis row.
fn summary(r: &CropRecord) -> Value {
    let first = |v: &Vec<String>| v.first().cloned().unwrap_or_default();
    json!({
        "key": r.key, "names": { "en": first(&r.names.en), "es": first(&r.names.es) },
        "scientific": r.scientific_name, "family": r.family, "crop_type": r.crop_type, "origin": r.origin, "label": r.label,
    })
}

/// Normalizes the farmer's crop words into crop records and stores them on the analysis.
pub async fn resolve_crops(state: &AppState, analysis_id: &str, current: &str, previous: &[String]) -> anyhow::Result<Value> {
    let cur = crops::resolve(state, current).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut prev = vec![];
    for p in previous.iter().filter(|p| !p.trim().is_empty()).take(6) {
        match crops::resolve(state, p).await {
            Ok(r) => prev.push(summary(&r)),
            Err(e) => tracing::warn!("crop '{p}' not resolved: {e}"),
        }
    }
    let cur_v = summary(&cur);
    store::set_crops(&state.db, analysis_id, Some(&cur_v), Some(&prev)).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(json!({ "current": cur_v, "previous": prev }))
}

fn signals(ev: &Value) -> FieldSignals {
    let reading = |id: &str| ev["readings"].as_array().and_then(|r| r.iter().find(|x| x["id"] == id)).and_then(|x| x["value"].as_f64());
    let aphid = ev["pests"].as_array().and_then(|p| p.iter().find(|x| x["scientific"].as_str().unwrap_or("").contains("Myzus")));
    FieldSignals {
        heat_stress_days: reading("heat_days"),
        water_balance_mm: reading("water_balance"),
        pest_generation_delta: aphid.and_then(|a| Some(a["generations_now"].as_f64()? - a["generations_normal"].as_f64()?)),
        bare_soil: ev["bare_soil"].as_array().map(|b| !b.is_empty()),
    }
}

fn cites(v: &Value) -> Vec<Value> {
    v.as_array().into_iter().flatten().map(|c| json!({ "title": c["title"], "url": c["url"] })).collect()
}

/// A block item has one bilingual text: its first clause becomes the title, the rest the body.
fn item_from(it: &Value) -> Value {
    let split = |s: &str| -> (String, String) {
        match s.find([':', '.']) {
            Some(i) if i < 70 && i + 1 < s.len() => (s[..i].trim().to_string(), s[i + 1..].trim().to_string()),
            _ => (String::new(), s.to_string()),
        }
    };
    let (te, be) = split(it["text"]["en"].as_str().unwrap_or(""));
    let (ts, bs) = split(it["text"]["es"].as_str().unwrap_or(""));
    json!({ "title": { "en": te, "es": ts }, "body": { "en": be, "es": bs }, "citations": cites(&it["citations"]), "basis": it["basis"] })
}

fn join(list: &Value, lang: &str) -> String {
    list.as_array().into_iter().flatten().filter_map(|x| x[lang].as_str().or_else(|| x["text"][lang].as_str())).collect::<Vec<_>>().join(" ")
}

fn bi_join(list: &Value) -> Value { json!({ "en": join(list, "en"), "es": join(list, "es") }) }

fn block_content(b: &Value, summary: Value) -> Value {
    json!({ "title": b["title"], "summary": summary, "items": b["items"].as_array().into_iter().flatten().map(item_from).collect::<Vec<_>>() })
}

/// Runs the advice engine on crops + evidence signals and writes the soil, pests, nutrition and rotation sections.
pub async fn advise(state: &AppState, analysis_id: &str) -> anyhow::Result<Value> {
    let st = store::load(&state.db, analysis_id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let Some(cur) = st.crop_current.as_ref() else { anyhow::bail!("call set_crops first: the current crop is unknown") };
    let key = cur["key"].as_str().unwrap_or_default();
    let cur_rec = crops::resolve(state, key).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut prev = vec![];
    for p in &st.crop_history {
        if let Some(k) = p["key"].as_str() { if let Ok(r) = crops::resolve(state, k).await { prev.push(r); } }
    }
    let ev = match st.evidence.clone() { Some(e) if e["readings"].as_array().map(|r| !r.is_empty()).unwrap_or(false) => e, _ => crate::build(state, analysis_id).await? };
    let adv = serde_json::to_value(crops::advice(&cur_rec, &prev, &signals(&ev)))?;
    let notice = adv["notice"].clone();

    // Soil and nutrition
    store::set_section(&state.db, analysis_id, "soil", "ready", None, Some(&block_content(&adv["soil"], notice.clone()))).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    store::set_section(&state.db, analysis_id, "nutrition", "ready", None,
        Some(&block_content(&adv["nutrition"], json!({ "en": "General guidance: without a soil test this is not a fertilizer prescription.", "es": "Guía general: sin análisis de suelo no es una receta de fertilización." })))).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    // Pest care: generations from degree-days where a published threshold exists, then the cited care block
    let mut pests = block_content(&adv["pest_care"], json!({ "en": "", "es": "" }));
    let gens: Vec<String> = ev["pests"].as_array().into_iter().flatten().filter_map(|p| {
        let (n, m) = (p["generations_now"].as_f64()?, p["generations_normal"].as_f64()?);
        Some(format!("{}: {:.1} vs {:.1}", p["name"].as_str()?, n, m))
    }).collect();
    pests["summary"] = json!({
        "en": if gens.is_empty() { "Guidance is qualitative: no published degree-day threshold for this crop's pests.".to_string() } else { format!("Estimated generations this season vs the 40-year normal (NASA POWER degree-days): {}.", gens.join("; ")) },
        "es": if gens.is_empty() { "Guía cualitativa: no hay umbrales publicados de grados-día para las plagas de este cultivo.".to_string() } else { format!("Generaciones estimadas esta campaña vs la normal de 40 años (grados-día NASA POWER): {}.", gens.join("; ")) },
    });
    pests["thresholds"] = adv["pest_thresholds"].clone();
    store::set_section(&state.db, analysis_id, "pests", "ready", None, Some(&pests)).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    // Rotation (annual, semi-perennial) or the perennial system
    let mode = adv["mode"].as_str().unwrap_or("rotation");
    let strategies: Vec<Value> = adv["strategies"].as_array().into_iter().flatten().map(|s| {
        let sc = &s["scores"];
        json!({
            "name": s["title"], "current": s["is_current"],
            "timeline": s["sequence"].as_array().into_iter().flatten().filter_map(|x| x["en"].as_str()).collect::<Vec<_>>(),
            "timeline_es": s["sequence"].as_array().into_iter().flatten().filter_map(|x| x["es"].as_str()).collect::<Vec<_>>(),
            "scores": { "soil": sc["soil"], "water": sc["water"], "pest_break": sc["pest_break"], "income": sc["income"], "risk": sc["risk"] },
            "why": s["why"], "evidence": bi_join(&s["evidence"]), "gaps": bi_join(&s["gaps"]), "risks": bi_join(&s["risks"]),
            "citations": s["evidence"].as_array().into_iter().flatten().filter_map(|e| e.get("citation").filter(|c| !c.is_null()).map(|c| json!({ "title": c["title"], "url": c["url"] }))).collect::<Vec<_>>(),
        })
    }).collect();
    let mut items: Vec<Value> = adv["practices"].as_array().into_iter().flatten().map(|p| {
        let actions = |l: &str| p["actions"].as_array().into_iter().flatten().filter_map(|a| a[l].as_str()).collect::<Vec<_>>().join(" · ");
        json!({ "title": p["title"],
                "body": { "en": format!("{} {}", p["why"]["en"].as_str().unwrap_or(""), actions("en")), "es": format!("{} {}", p["why"]["es"].as_str().unwrap_or(""), actions("es")) },
                "citations": p["evidence"].as_array().into_iter().flatten().filter_map(|e| e.get("citation").filter(|c| !c.is_null()).map(|c| json!({ "title": c["title"], "url": c["url"] }))).collect::<Vec<_>>() })
    }).collect();
    if let Some(ic) = adv.get("interim_cover").filter(|x| !x.is_null()) {
        items.extend(ic["items"].as_array().into_iter().flatten().map(item_from));
    }
    let (title, summary) = match mode {
        "perennial-system" => (json!({ "en": "Perennial system", "es": "Sistema perenne" }),
            json!({ "en": "A perennial crop is not rotated: these practices protect the soil and the plantation.", "es": "Un cultivo perenne no se rota: estas prácticas protegen el suelo y la plantación." })),
        "renewal" => (json!({ "en": "Renewal and interim cover", "es": "Renovación y cobertura intermedia" }),
            json!({ "en": "Three options for the next renewal, compared on your field.", "es": "Tres opciones para la próxima renovación, comparadas en tu campo." })),
        _ => (json!({ "en": "Rotation: three strategies", "es": "Rotación: tres estrategias" }),
            json!({ "en": "Your current plan and two alternatives, scored 1–5 (5 = best; for risk, 5 = lowest).", "es": "Tu plan actual y dos alternativas, puntuadas de 1 a 5 (5 = mejor; en riesgo, 5 = menor)." })),
    };
    let rotation = json!({ "title": title, "summary": summary, "mode": mode, "strategies": strategies, "items": items, "gaps": adv["gaps"] });
    let rot_state = if adv["gaps"].as_array().map(|g| g.is_empty()).unwrap_or(true) { "ready" } else { "gaps" };
    let note = (rot_state == "gaps").then(|| SBi::new("some information gaps", "algunos vacíos de información"));
    store::set_section(&state.db, analysis_id, "rotation", rot_state, note.as_ref(), Some(&rotation)).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    store::set_section(&state.db, analysis_id, "sheet", "ready", None, None).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    // What the consultant gets back: a compact summary it may cite
    Ok(json!({
        "crop": adv["crop"], "mode": mode, "notice": notice,
        "strategies": strategies.iter().map(|s| json!({ "name": s["name"], "current": s["current"], "scores": s["scores"], "why": s["why"] })).collect::<Vec<_>>(),
        "practices": items.iter().map(|i| i["title"].clone()).collect::<Vec<_>>(),
        "soil": adv["soil"]["items"].as_array().map(|a| a.iter().map(|i| i["text"]["en"].clone()).collect::<Vec<_>>()),
        "pest_care": adv["pest_care"]["items"].as_array().map(|a| a.iter().map(|i| i["text"]["en"].clone()).collect::<Vec<_>>()),
        "nutrition": adv["nutrition"]["items"].as_array().map(|a| a.iter().map(|i| i["text"]["en"].clone()).collect::<Vec<_>>()),
        "gaps": adv["gaps"],
        "sections_written": ["soil", "pests", "nutrition", "rotation"],
    }))
}
