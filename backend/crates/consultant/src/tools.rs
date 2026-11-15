//! Client tools the consultant may call, and their execution against the analysis.
use analyses::{store, Bi};
use common::AppState;
use serde_json::{json, Value};

pub fn definitions() -> Value {
    let bi = |d: &str| json!({ "type": "string", "description": d });
    json!([
        { "name": "set_crops", "description": "Record the crop growing now and the previous crops, as the farmer named them (any crop, any language). Returns normalized crop records with crop type (annual / semi-perennial / perennial).",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["current", "previous"],
            "properties": { "current": { "type": "string" }, "previous": { "type": "array", "items": { "type": "string" } } } } },
        { "name": "get_evidence", "description": "Build or fetch the evidence package for this field: NASA POWER climate vs the 40-year normal, pest degree-days, SMAP soil moisture, IMERG rain, GRACE water storage, Sentinel-2/Landsat crop vigor and bare-soil periods, NASADEM slope, climate projection. Every value has source, date and scale badge. The only numbers you may cite.",
          "input_schema": { "type": "object", "additionalProperties": false, "properties": {} } },
        { "name": "get_advice", "description": "Run the cited advice engine for the recorded crops and this field's signals. Fills the Soil, Pest care, Nutrition and Rotation sections and returns a summary.",
          "input_schema": { "type": "object", "additionalProperties": false, "properties": {} } },
        { "name": "update_section", "description": "Set a result section's state and a short note, optionally adding what you observed in the farmer's photos (hedged wording).",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["key", "state", "note_en", "note_es", "observation_en", "observation_es"],
            "properties": {
              "key": { "type": "string", "enum": ["field", "soil", "pests", "nutrition", "rotation"] },
              "state": { "type": "string", "enum": ["progress", "ready", "gaps"] },
              "note_en": bi("Short note, English, max 8 words, e.g. 'needs a closer photo'. Empty string for none."),
              "note_es": bi("Same note in Spanish. Empty string for none."),
              "observation_en": bi("What the photos show, hedged, English, 1-2 sentences. Empty string for none."),
              "observation_es": bi("Same observation in Spanish. Empty string for none.") } } },
        { "name": "tag_photo", "description": "Tag one photo of the farmer's latest message (1-based index) so the interface marks it, e.g. 'blurry', 'too far', 'too dark', 'good'.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["index", "tag"],
            "properties": { "index": { "type": "integer" }, "tag": { "type": "string" } } } },
        { "name": "complete_step", "description": "Mark the active guided topic (or a later one already covered) as done when you have what it needs, or the farmer said they don't know (after record_gap). Gives a one-line summary of what was learned. Returns the next topic.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["key", "summary_en", "summary_es"],
            "properties": { "key": { "type": "string", "enum": ["field", "soil", "pests", "nutrition", "rotation"] },
              "summary_en": bi("One line, English, what was learned for this topic."), "summary_es": bi("Same in Spanish.") } } },
        { "name": "get_view", "description": "Read exactly what one tab of the app shows the farmer, with the same numbers, in the conversation language: dashboard (reading tiles, pests, forecast, outlook), rotation (strategies numbered 1-3 left to right, with their 1-5 scores, best fit, why, evidence, gaps, risks), organic (natural enemies numbered as on screen, practices, timing), prevention (pest risk lights), sheet (sections and gaps). Use it when the farmer asks about something they are looking at; the context note says which tab and item.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["view"],
            "properties": { "view": { "type": "string", "enum": ["dashboard", "rotation", "organic", "prevention", "sheet"] } } } },
        { "name": "evaluate_crop", "description": "Score a crop the farmer is thinking of growing ('what do you think of growing avocado?') with the same rotation engine and this field's NASA signals: soil, water, pest-cycle break by botanical family, income, risk, with why, gaps and risks. Adds a 'Your proposal' column to the Rotation tab. Any crop name, any language. Perennials are evaluated as plantations, never as a rotation.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["crop"], "properties": { "crop": { "type": "string" } } } },
        { "name": "get_sheet", "description": "Read the technical sheet as it stands: each section's state, summary and item titles, the guided topics and the information gaps. Use it to point the farmer to what the sheet already says and to give the closing recommendations.",
          "input_schema": { "type": "object", "additionalProperties": false, "properties": {} } },
        { "name": "record_facts", "description": "Record what the farmer told you (or what an attached document says) as short plain answers. Use only the fields you learned; they appear on the technical sheet as 'What the farmer told us'.",
          "input_schema": { "type": "object", "additionalProperties": false, "properties": {
            "crop_now": { "type": "string" }, "crops_before": { "type": "array", "items": { "type": "string" } }, "irrigation": { "type": "string" },
            "soil_type": { "type": "string" }, "organic_matter": { "type": "string" }, "residues": { "type": "string" },
            "pests_seen": { "type": "string" }, "products_recent": { "type": "string" }, "last_fertilizer": { "type": "string" },
            "soil_test": { "type": "string" }, "next_crop": { "type": "string" }, "harvest_use": { "type": "string" } } } },
        { "name": "record_gap", "description": "Record an information gap the farmer could not answer (e.g. last fertilizer unknown). It is shown as a declared gap, never guessed.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["text_en", "text_es"],
            "properties": { "text_en": { "type": "string" }, "text_es": { "type": "string" } } } }
    ])
}

/// The text sent back to the model for a tool's output.
pub fn result_text(v: &Value) -> String { v.to_string() }

/// Side effects collected while running tools that the turn applies at the end.
#[derive(Default)]
pub struct Effects {
    pub photo_tags: Vec<(usize, String)>,
}

pub async fn execute(state: &AppState, id: &str, name: &str, input: &Value, fx: &mut Effects) -> Result<Value, String> {
    let s = |k: &str| input.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    match name {
        "set_crops" => {
            let prev: Vec<String> = input["previous"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            evidence::resolve_crops(state, id, &s("current"), &prev).await.map_err(|e| e.to_string())
        }
        // The stored package (built once, refreshed when the crops change), compacted in the conversation language.
        "get_evidence" => {
            let st = store::load(&state.db, id).await.map_err(|e| e.to_string())?;
            let ev = match st.evidence.filter(|e| e["readings"].as_array().is_some_and(|r| !r.is_empty())) {
                Some(e) => e,
                None => evidence::build(state, id).await.map_err(|e| e.to_string())?,
            };
            Ok(crate::compact::evidence(&ev, lang(&st.lang)))
        }
        "get_advice" => {
            let a = evidence::advise(state, id).await.map_err(|e| e.to_string())?;
            let st_lang = store::load(&state.db, id).await.map(|s| s.lang).unwrap_or_default();
            Ok(crate::compact::advice(&a, lang(&st_lang)))
        }
        "update_section" => {
            let key = s("key");
            let note = (!s("note_en").is_empty()).then(|| Bi::new(s("note_en"), s("note_es")));
            let obs = (!s("observation_en").is_empty()).then(|| Bi::new(s("observation_en"), s("observation_es")));
            let current = store::sections(&state.db, id).await.map_err(|e| e.to_string())?;
            let mut content = current.iter().find(|x| x.key == key).and_then(|x| x.content.clone()).unwrap_or(Value::Null);
            if let Some(o) = obs { add_observation(&mut content, &key, o); }
            let content = (!content.is_null()).then_some(content);
            store::set_section(&state.db, id, &key, &s("state"), note.as_ref(), content.as_ref()).await.map_err(|e| e.to_string())?;
            Ok(json!({ "ok": true }))
        }
        "tag_photo" => {
            let i = input["index"].as_u64().unwrap_or(0) as usize;
            fx.photo_tags.push((i, s("tag")));
            Ok(json!({ "ok": true }))
        }
        "complete_step" => {
            let summary = Bi::new(s("summary_en"), s("summary_es"));
            let next = store::complete_step(&state.db, id, &s("key"), &summary).await.map_err(|e| e.to_string())?;
            let st = store::load(&state.db, id).await.map_err(|e| e.to_string())?;
            let next_needs = st.steps.iter().find(|x| Some(&x.key) == next.as_ref()).map(|x| x.needs.iter().map(|n| n.en.clone()).collect::<Vec<_>>());
            Ok(json!({ "ok": true, "next_topic": next, "next_topic_needs": next_needs, "all_done": next.is_none() }))
        }
        "record_facts" => {
            let merged = store::merge_facts(&state.db, id, input).await.map_err(|e| e.to_string())?;
            Ok(json!({ "ok": true, "facts": merged }))
        }
        "evaluate_crop" => {
            let p = evidence::proposal::evaluate(state, id, &s("crop")).await.map_err(|e| e.to_string())?;
            let st_lang = store::load(&state.db, id).await.map(|x| x.lang).unwrap_or_default();
            Ok(crate::views::deep(&json!({ "proposal": { "name": p["name"], "crop": p["crop"], "sequence": if st_lang == "es" { p["timeline_es"].clone() } else { p["timeline"].clone() },
                "scores_1_to_5": p["scores"], "why": p["why"], "gaps": p["gaps_list"], "risks": p["risks_list"] },
                "note": "Shown as the 'Your proposal' column in the Rotation tab; compare it with get_view('rotation')." }), lang(&st_lang)))
        }
        "get_view" => {
            let st = store::load(&state.db, id).await.map_err(|e| e.to_string())?;
            let l = lang(&st.lang);
            Ok(crate::views::view(&st, &s("view"), l))
        }
        "get_sheet" => {
            let st = store::load(&state.db, id).await.map_err(|e| e.to_string())?;
            let sections: Vec<Value> = st.sections.iter().map(|x| {
                let c = x.content.clone().unwrap_or(Value::Null);
                json!({ "key": x.key, "state": x.state, "summary": c["summary"]["en"], "items": c["items"].as_array().map(|a| a.iter().filter_map(|i| i["title"]["en"].as_str().map(String::from)).collect::<Vec<_>>()),
                        "strategies": c["strategies"].as_array().map(|a| a.iter().map(|s| json!({ "name": s["name"]["en"], "current": s["current"], "best_for": s["best_for"]["en"] })).collect::<Vec<_>>()) })
            }).collect();
            Ok(json!({ "sections": sections, "steps": st.steps.iter().map(|x| json!({ "key": x.key, "state": x.state, "summary": x.summary.as_ref().map(|b| b.en.clone()) })).collect::<Vec<_>>(),
                       "prediagnosis": st.prediagnosis, "gaps": st.evidence.as_ref().map(|e| e["gaps"].clone()), "sheet_url": format!("/api/export/{id}/sheet") }))
        }
        "record_gap" => {
            let current = store::sections(&state.db, id).await.map_err(|e| e.to_string())?;
            let field = current.iter().find(|x| x.key == "field");
            let mut content = field.and_then(|x| x.content.clone()).unwrap_or_else(|| json!({ "title": {"en": "Your field", "es": "Tu campo"}, "summary": {"en": "", "es": ""}, "items": [] }));
            let gaps = content.as_object_mut().map(|o| o.entry("declared_gaps").or_insert_with(|| json!([])));
            if let Some(Value::Array(g)) = gaps { g.push(json!({ "en": s("text_en"), "es": s("text_es") })); }
            let st = field.map(|f| f.state.clone()).unwrap_or_else(|| "progress".into());
            let note = field.and_then(|f| f.note.clone());
            store::set_section(&state.db, id, "field", &st, note.as_ref(), Some(&content)).await.map_err(|e| e.to_string())?;
            // An unknown fertilizer history is a nutrition gap: show it on that section too.
            let en = s("text_en").to_lowercase();
            if en.contains("fertili") || en.contains("manure") {
                if let Some(n) = current.iter().find(|x| x.key == "nutrition") {
                    let note = Bi::new("needs: last fertilizer used", "falta: último abono usado");
                    store::set_section(&state.db, id, "nutrition", "gaps", Some(&note), n.content.as_ref()).await.map_err(|e| e.to_string())?;
                }
            }
            Ok(json!({ "ok": true }))
        }
        other => Err(format!("unknown tool {other}")),
    }
}

fn add_observation(content: &mut Value, key: &str, o: Bi) {
    if content.is_null() {
        *content = json!({ "title": { "en": key, "es": key }, "summary": { "en": "", "es": "" }, "items": [] });
    }
    let item = json!({
        "title": { "en": "From your photos (AI observation, verify)", "es": "De tus fotos (observación de IA, verificar)" },
        "body": o, "ai_observation": true
    });
    if let Some(items) = content.get_mut("items").and_then(Value::as_array_mut) {
        items.retain(|i| i.get("ai_observation").is_none());
        items.insert(0, item);
    }
}

fn lang(l: &str) -> &'static str { if l == "es" { "es" } else { "en" } }
