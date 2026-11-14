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
        pest_generation_delta: aphid.and_then(|a| a["difference"].as_f64()),
        bare_soil: ev["bare_soil"].as_array().map(|b| !b.is_empty()),
    }
}

fn cites(v: &Value) -> Vec<Value> {
    v.as_array().into_iter().flatten().map(|c| json!({ "title": c["title"], "url": c["url"] })).collect()
}

/// A block item has one bilingual text: its first clause becomes the title, the rest the body.
fn item_from(it: &Value) -> Value {
    let split = |s: &str| -> (String, String) {
        let cut = s.find(": ").filter(|i| { let head = s[..*i].trim_end(); !(head.ends_with(" for") || head.ends_with(" para") || head.ends_with(" en") || head.ends_with(" de")) });
        match cut {
            Some(i) if i < 70 => (s[..i].trim().to_string(), s[i + 2..].trim().to_string()),
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

/// Summary = the key actions (item titles), so a one-line view says what to do, not only the disclaimer.
fn block_content(b: &Value, _note: Value) -> Value {
    let items: Vec<Value> = b["items"].as_array().into_iter().flatten().map(item_from).collect();
    let key = |l: &str| items.iter().filter_map(|i| { let t = i["title"][l].as_str().unwrap_or(""); (!t.is_empty() && t.len() < 70).then(|| t.to_string()) })
        .take(3).collect::<Vec<_>>().join(" · ");
    json!({ "title": b["title"], "summary": { "en": key("en"), "es": key("es") }, "items": items })
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
    let thresholds = adv["pest_thresholds"].as_array().cloned().unwrap_or_default();
    let (lat, lon) = (st.field.center.lat, st.field.center.lon);
    let crop_pests = crate::crop_pests::generations(state, lat, lon, &thresholds).await.unwrap_or_else(|e| { tracing::warn!("crop pests: {e}"); vec![] });
    let line = |p: &Value, es: bool| -> Option<String> {
        let name = p["name"][if es { "es" } else { "en" }].as_str()?;
        Some(format!("{name}: {:.1} {} {:.1} ({}{:.1})", p["generations_now"].as_f64()?, if es { "vs" } else { "vs" }, p["generations_normal"].as_f64()?,
            if p["difference"].as_f64()? >= 0.0 { "+" } else { "" }, p["difference"].as_f64()?))
    };
    let season = crop_pests.first().and_then(|p| p["season"].as_str()).unwrap_or("").to_string();
    let en_l: Vec<String> = crop_pests.iter().filter_map(|p| line(p, false)).collect();
    let es_l: Vec<String> = crop_pests.iter().filter_map(|p| line(p, true)).collect();
    pests["summary"] = if en_l.is_empty() {
        json!({ "en": "Guidance is qualitative: no published degree-day threshold for this crop's pests.", "es": "Guía cualitativa: no hay umbrales publicados de grados-día para las plagas de este cultivo." })
    } else {
        json!({
            "en": format!("Estimated generations of this crop's pests this season vs the 40-year normal, from published thresholds and NASA POWER temperature (regional context; season window {season}, sowing date not given): {}.", en_l.join("; ")),
            "es": format!("Generaciones estimadas de las plagas de este cultivo esta campaña vs la normal de 40 años, con umbrales publicados y temperatura NASA POWER (contexto regional; ventana de campaña {season}, sin fecha de siembra): {}.", es_l.join("; ")),
        })
    };
    pests["crop_pests"] = json!(crop_pests);
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
            "evidence_list": s["evidence"].as_array().into_iter().flatten().map(|e| e["text"].clone()).collect::<Vec<_>>(),
            "gaps_list": s["gaps"].clone(), "risks_list": s["risks"].clone(),
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
    // Recommended strategy: the best total score among the alternatives (ties: first); it names its strongest criterion.
    let mut strategies = strategies;
    let total = |s: &Value| ["soil", "water", "pest_break", "income", "risk"].iter().map(|k| s["scores"][k].as_u64().unwrap_or(0)).sum::<u64>();
    if let Some((bi_, _)) = strategies.iter().enumerate().filter(|(_, s)| s["current"] != true).max_by_key(|(i, s)| (total(s), std::cmp::Reverse(*i))) {
        let sc = &strategies[bi_]["scores"];
        let (k, _) = [("soil", "improve soil", "mejorar el suelo"), ("water", "save water", "ahorrar agua"), ("pest_break", "break pest cycles", "cortar el ciclo de plagas"), ("income", "income", "ingreso"), ("risk", "lower risk", "menor riesgo")]
            .iter().map(|(k, en, es)| ((*k, *en, *es), sc[*k].as_u64().unwrap_or(0))).max_by_key(|(_, v)| *v).unwrap();
        strategies[bi_]["best_for"] = json!({ "en": format!("Best fit: {}", k.1), "es": format!("Mejor opción: {}", k.2) });
    }
    let rotation = json!({ "title": title, "summary": summary, "mode": mode, "strategies": strategies, "items": items, "gaps": adv["gaps"], "prediagnosis": prediagnosis(&ev) });
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

/// One-line pre-diagnosis from the package's own signals (never from photos), with a confidence that reflects scale:
/// "medium" when field-level vigor and regional climate agree, "low" when only regional data exist.
fn prediagnosis(ev: &Value) -> Value {
    let r = |id: &str| ev["readings"].as_array().and_then(|a| a.iter().find(|x| x["id"] == id)).and_then(|x| x["value"].as_f64());
    let mut en = vec![]; let mut es = vec![];
    if r("temp_anomaly").is_some_and(|a| a >= 2.0) { en.push("a much warmer season than normal"); es.push("una campaña mucho más cálida que lo normal"); }
    if r("water_balance").is_some_and(|w| w < -50.0) { en.push("a large irrigation deficit"); es.push("un gran déficit de riego"); }
    if r("pest_gen").is_some() && ev["pests"].as_array().map(|p| p.iter().any(|x| x["difference"].as_f64().unwrap_or(0.0) >= 1.0)).unwrap_or(false) {
        en.push("pest pressure above normal"); es.push("presión de plagas mayor a lo normal");
    }
    let field = r("ndvi").is_some();
    let join = |v: &Vec<&str>, and: &str| match v.len() { 0 => String::new(), 1 => v[0].to_string(), _ => format!("{} {and} {}", v[..v.len() - 1].join(", "), v[v.len() - 1]) };
    let (h_en, h_es) = if en.is_empty() {
        ("No strong warning signal in the NASA data for this field".to_string(), "Sin señales fuertes de alerta en los datos NASA para este campo".to_string())
    } else {
        (format!("Likely {}", join(&en, "and")), format!("Probablemente {}", join(&es, "y")))
    };
    json!({ "headline": { "en": h_en, "es": h_es }, "confidence": if field && en.len() >= 2 { "medium" } else { "low" },
            "basis": { "en": "From NASA readings only; photos and a field visit by your agronomist decide.", "es": "Solo a partir de lecturas NASA; las fotos y la visita de tu agrónomo deciden." } })
}
