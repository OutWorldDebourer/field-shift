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
        { "name": "record_gap", "description": "Record an information gap the farmer could not answer (e.g. last fertilizer unknown). It is shown as a declared gap, never guessed.",
          "strict": true,
          "input_schema": { "type": "object", "additionalProperties": false, "required": ["text_en", "text_es"],
            "properties": { "text_en": { "type": "string" }, "text_es": { "type": "string" } } } }
    ])
}

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
        "get_evidence" => evidence::build(state, id).await.map(compact_evidence).map_err(|e| e.to_string()),
        "get_advice" => evidence::advise(state, id).await.map_err(|e| e.to_string()),
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
        "record_gap" => {
            let current = store::sections(&state.db, id).await.map_err(|e| e.to_string())?;
            let field = current.iter().find(|x| x.key == "field");
            let mut content = field.and_then(|x| x.content.clone()).unwrap_or_else(|| json!({ "title": {"en": "Your field", "es": "Tu campo"}, "summary": {"en": "", "es": ""}, "items": [] }));
            let gaps = content.as_object_mut().map(|o| o.entry("declared_gaps").or_insert_with(|| json!([])));
            if let Some(Value::Array(g)) = gaps { g.push(json!({ "en": s("text_en"), "es": s("text_es") })); }
            let st = field.map(|f| f.state.clone()).unwrap_or_else(|| "progress".into());
            let note = field.and_then(|f| f.note.clone());
            store::set_section(&state.db, id, "field", &st, note.as_ref(), Some(&content)).await.map_err(|e| e.to_string())?;
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

/// The model gets the readings without long series (keeps the request small); the UI gets the full package.
fn compact_evidence(mut ev: Value) -> Value {
    if let Some(rs) = ev.get_mut("readings").and_then(Value::as_array_mut) {
        for r in rs { if let Some(o) = r.as_object_mut() { o.remove("series"); } }
    }
    if let Some(o) = ev.as_object_mut() {
        if let Some(n) = o.remove("ndvi_series").and_then(|v| v.as_array().cloned()) {
            o.insert("ndvi_latest".into(), json!(n.last()));
            o.insert("ndvi_points".into(), json!(n.len()));
        }
        o.remove("climate_monthly");
    }
    ev
}
