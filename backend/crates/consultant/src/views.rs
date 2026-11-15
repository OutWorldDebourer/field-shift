//! `get_view(view)`: what each tab of the app shows the farmer, read from the same stored sections and evidence the
//! frontend renders, compacted and in the conversation language. Lets the consultant answer "why does this one…?"
//! about the card the farmer is looking at, with the same numbers.
use analyses::AnalysisState;
use serde_json::{json, Value};

pub const VIEWS: [&str; 5] = ["dashboard", "rotation", "organic", "prevention", "sheet"];

fn l(v: &Value, lang: &str) -> Value {
    match v {
        Value::Object(o) if o.contains_key("en") => o.get(lang).or_else(|| o.get("en")).cloned().unwrap_or(Value::Null),
        other => other.clone(),
    }
}

/// Every `{en, es}` pair in a value reduced to one language.
pub fn deep(v: &Value, lang: &str) -> Value {
    match v {
        Value::Object(o) if o.len() <= 2 && o.contains_key("en") && o.contains_key("es") => l(v, lang),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), deep(x, lang))).collect()),
        Value::Array(a) => Value::Array(a.iter().map(|x| deep(x, lang)).collect()),
        other => other.clone(),
    }
}

fn list(v: &Value, lang: &str) -> Vec<Value> { v.as_array().into_iter().flatten().map(|x| l(x, lang)).collect() }

fn section<'a>(st: &'a AnalysisState, key: &str) -> Option<&'a Value> {
    st.sections.iter().find(|s| s.key == key).and_then(|s| s.content.as_ref())
}

/// The Rotation tab: strategies numbered as on screen (1 = left), with the five scores, best fit, why, evidence,
/// gaps and risks; or the perennial practices.
fn rotation(st: &AnalysisState, lang: &str) -> Value {
    let Some(c) = section(st, "rotation") else { return json!({ "view": "rotation", "available": false }) };
    let strategies: Vec<Value> = c["strategies"].as_array().into_iter().flatten().enumerate().map(|(i, s)| json!({
        "number": i + 1, "name": l(&s["name"], lang), "is_current_plan": s["current"],
        "sequence": if lang == "es" { s["timeline_es"].clone() } else { s["timeline"].clone() },
        "scores_1_to_5": { "soil_health": s["scores"]["soil"], "water_use": s["scores"]["water"], "pest_cycle_break": s["scores"]["pest_break"],
                           "extra_income": s["scores"]["income"], "low_risk": s["scores"]["risk"] },
        "best_fit": l(&s["best_for"], lang), "why": l(&s["why"], lang),
        "evidence": list(&s["evidence_list"], lang), "gaps": list(&s["gaps_list"], lang), "risks": list(&s["risks_list"], lang),
    })).collect();
    json!({
        "view": "rotation", "language": lang, "title": l(&c["title"], lang), "mode": c["mode"], "summary": l(&c["summary"], lang),
        "scale": "scores 1-5; 5 = best; for risk 5 = lowest risk",
        "strategies": strategies,
        "practices": c["items"].as_array().into_iter().flatten().map(|i| json!({ "title": l(&i["title"], lang), "body": l(&i["body"], lang) })).collect::<Vec<_>>(),
        "prediagnosis": { "headline": l(&c["prediagnosis"]["headline"], lang), "confidence": c["prediagnosis"]["confidence"] },
        "gaps": list(&c["gaps"], lang),
    })
}

/// The Organic tab: natural enemies and microorganisms (numbered as on screen), practices, timing, what to avoid.
fn organic(st: &AnalysisState, lang: &str) -> Value {
    let Some(c) = section(st, "organic") else { return json!({ "view": "organic", "available": false }) };
    json!({
        "view": "organic", "language": lang, "summary": l(&c["summary"], lang),
        "agents": c["agents"].as_array().into_iter().flatten().enumerate().map(|(i, a)| json!({
            "number": i + 1, "name": l(&a["name"], lang), "scientific": a["scientific"], "kind": a["kind"], "against": l(&a["targets"], lang),
            "for_reported_pest": a["for_reported_pest"], "recorded_on_crop": a["crop"], "source": a["citation"]["title"],
        })).collect::<Vec<_>>(),
        "practices": c["practices"].as_array().into_iter().flatten().map(|p| l(&p["title"], lang)).collect::<Vec<_>>(),
        "when_to_act": list(&c["timing"], lang), "avoid": list(&c["avoid"], lang), "gaps": list(&c["gaps"], lang),
    })
}

/// The Prevention tab: per crop, each pest's light, reason and monitoring (no long texts, no citations list).
fn prevention(c: &Value, lang: &str) -> Value {
    json!({
        "view": "prevention", "language": lang, "summary": l(&c["summary"], lang), "notice": l(&c["notice"], lang),
        "forecast": c["forecast"], "temp_anomaly_c": c["temp_anomaly"],
        "crops": c["crops"].as_array().into_iter().flatten().map(|b| json!({
            "crop": l(&b["crop"]["name"], lang), "role": b["crop"]["role"], "crop_type": b["crop"]["crop_type"],
            "ai_researched_unreviewed": b["researched"] == true && b["reviewed"] != true,
            "pests": b["pests"].as_array().into_iter().flatten().map(|p| json!({
                "name": l(&p["name"], lang), "risk": p["light"]["level"], "why": l(&p["light"]["reason"], lang),
                "favoured_by": l(&p["light"]["favourable"], lang), "monitoring": l(&p["monitoring"], lang),
                "generations_now": p["generations_now"], "generations_normal": p["generations_normal"],
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// The Dashboard tab: the reading tiles, pests, forecast and outlook, as the compact evidence package.
fn dashboard(st: &AnalysisState, lang: &str) -> Value {
    let Some(ev) = st.evidence.as_ref() else { return json!({ "view": "dashboard", "available": false }) };
    let mut v = crate::compact::evidence(ev, lang);
    v["view"] = json!("dashboard");
    v
}

/// The technical sheet: each section's state, summary and item titles, the guided topics and the gaps.
fn sheet(st: &AnalysisState, lang: &str) -> Value {
    json!({
        "view": "sheet", "language": lang,
        "sections": st.sections.iter().map(|s| {
            let c = s.content.clone().unwrap_or(Value::Null);
            json!({ "key": s.key, "state": s.state, "summary": l(&c["summary"], lang),
                    "items": c["items"].as_array().map(|a| a.iter().map(|i| l(&i["title"], lang)).filter(|t| t.as_str().is_some_and(|s| !s.is_empty())).collect::<Vec<_>>()) })
        }).collect::<Vec<_>>(),
        "topics": st.steps.iter().map(|x| json!({ "key": x.key, "state": x.state })).collect::<Vec<_>>(),
        "gaps": st.evidence.as_ref().map(|e| e["gaps"].as_array().into_iter().flatten().map(|g| l(&g["text"], lang)).collect::<Vec<_>>()),
        "sheet_url": format!("/api/export/{}/sheet", st.id),
    })
}

pub fn view(st: &AnalysisState, name: &str, lang: &str) -> Value {
    match name {
        "rotation" => rotation(st, lang),
        "organic" => organic(st, lang),
        "dashboard" => dashboard(st, lang),
        "sheet" => sheet(st, lang),
        "prevention" => match section(st, "prevention") {
            Some(c) => prevention(c, lang),
            None => json!({ "view": "prevention", "available": false, "note": "The Prevention tab is not built for this analysis yet." }),
        },
        other => json!({ "error": format!("unknown view {other}; use one of {}", VIEWS.join(", ")) }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st() -> AnalysisState {
        serde_json::from_value(json!({
            "id": "x", "lang": "en", "location_mode": "map", "example_site": null,
            "field": { "center": { "lat": -11.5, "lon": -77.2 }, "ring": [], "area_ha": 1.0, "label": "f" },
            "crop_current": null, "crop_history": [], "messages": [], "evidence": null, "demo_mode": false, "prediagnosis": null, "steps": [], "facts": null, "next_step": null,
            "sections": [{ "key": "rotation", "state": "ready", "content": { "mode": "rotation", "title": { "en": "Rotation", "es": "Rotación" },
              "strategies": [
                { "name": { "en": "Keep pea", "es": "Seguir con arveja" }, "current": true, "scores": { "soil": 3, "water": 3, "pest_break": 1, "income": 3, "risk": 2 }, "why": { "en": "a", "es": "b" } },
                { "name": { "en": "Follow with maize", "es": "Seguir con maíz" }, "current": false, "scores": { "soil": 4, "water": 3, "pest_break": 5, "income": 3, "risk": 4 }, "why": { "en": "c", "es": "d" } } ] } }],
        })).unwrap()
    }

    #[test]
    fn rotation_view_numbers_strategies_as_on_screen() {
        let v = view(&st(), "rotation", "es");
        assert_eq!(v["strategies"][1]["number"], 2);
        assert_eq!(v["strategies"][1]["name"], "Seguir con maíz");
        assert_eq!(v["strategies"][1]["scores_1_to_5"]["low_risk"], 4);
        assert_eq!(view(&st(), "prevention", "en")["available"], false);
    }

    #[test]
    fn prevention_view_is_compact_and_localized() {
        let mut st = st();
        st.sections.push(serde_json::from_value(json!({ "key": "prevention", "state": "ready", "content": {
            "summary": { "en": "1 high", "es": "1 alto" }, "notice": { "en": "model", "es": "modelo" }, "forecast": null, "temp_anomaly": 5.0,
            "sources": [{ "name": { "en": "NASA POWER", "es": "NASA POWER" }, "url": "https://power.larc.nasa.gov/" }],
            "crops": [{ "crop": { "key": "pea", "name": { "en": "pea", "es": "arveja" }, "role": "current", "crop_type": "annual" }, "researched": true, "reviewed": false,
              "pests": [{ "name": { "en": "Pea aphid", "es": "Pulgón" }, "light": { "level": "high", "reason": { "en": "Three agree", "es": "Tres coinciden" }, "favourable": { "en": "mild", "es": "templado" }, "citation": { "title": "t", "url": "https://x.org" } },
                          "monitoring": { "en": "Check twice", "es": "Revisa dos veces" }, "generations_now": 13.4, "generations_normal": 10.4 }] }] } })).unwrap());
        let v = view(&st, "prevention", "es");
        assert_eq!(v["crops"][0]["crop"], "arveja");
        assert_eq!(v["crops"][0]["pests"][0]["risk"], "high");
        assert_eq!(v["crops"][0]["pests"][0]["why"], "Tres coinciden");
        assert_eq!(v["crops"][0]["ai_researched_unreviewed"], true);
        assert!(!v.to_string().contains("https://x.org"), "no citation lists for the model");
    }

    #[test]
    fn viewing_note_names_tab_and_card() {
        let n = crate::prompt::viewing_note(Some("rotation"), Some("strategy 2 (Follow with maize)")).unwrap();
        assert!(n.contains("rotation tab · strategy 2") && n.contains("get_view(\"rotation\")"));
        assert!(crate::prompt::viewing_note(Some("chat"), None).is_none());
    }
}
