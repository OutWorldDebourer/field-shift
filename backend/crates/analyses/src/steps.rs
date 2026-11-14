//! The guided conversation: five topics in a fixed order, then the technical sheet. Each topic lists what the
//! consultant still needs, in plain words; the consultant marks a topic done when it has it (or a declared gap).
use crate::model::Bi;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const ORDER: [&str; 6] = ["field", "soil", "pests", "nutrition", "rotation", "sheet"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub key: String,
    pub state: String, // pending | active | done
    pub title: Bi,
    pub needs: Vec<Bi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Bi>,
}

pub fn title(key: &str) -> Bi {
    match key {
        "field" => Bi::new("Your field", "Tu campo"),
        "soil" => Bi::new("Soil", "Suelo"),
        "pests" => Bi::new("Pest care", "Manejo de plagas"),
        "nutrition" => Bi::new("Nutrition", "Nutrición"),
        "rotation" => Bi::new("Rotation", "Rotación"),
        _ => Bi::new("Technical sheet", "Ficha técnica"),
    }
}

/// What the consultant asks for on each topic (one or two questions at a time).
pub fn needs(key: &str) -> Vec<Bi> {
    let b = Bi::new;
    match key {
        "field" => vec![b("crop growing now", "cultivo actual"), b("crops grown before (last one or two)", "cultivos anteriores (uno o dos)"), b("how the field is irrigated", "cómo se riega")],
        "soil" => vec![b("soil type (sandy, loamy, clay…)", "tipo de suelo (arenoso, franco, arcilloso…)"), b("organic matter added (manure, compost, green manure)", "materia orgánica que agrega (estiércol, compost, abono verde)"), b("what is done with crop residues", "qué hace con los rastrojos")],
        "pests" => vec![b("insects or damage seen, and where on the plant", "insectos o daños vistos y en qué parte de la planta"), b("close, sharp photos of the affected leaves", "fotos cercanas y nítidas de las hojas afectadas"), b("products applied recently", "productos aplicados recientemente")],
        "nutrition" => vec![b("last fertilizer used and when", "último abono usado y cuándo"), b("any soil test available", "si tiene análisis de suelo")],
        "rotation" => vec![b("what the farmer would like to plant next", "qué le gustaría sembrar después"), b("market or family use of the harvest", "si la cosecha es para venta o consumo")],
        _ => vec![],
    }
}

/// Current steps from the stored JSON (`{key: {state, summary}}`): done topics, the first open one active.
pub fn from_stored(stored: Option<&Value>) -> (Vec<Step>, Option<String>) {
    let mut steps = vec![];
    let mut next: Option<String> = None;
    for key in ORDER {
        let rec = stored.and_then(|s| s.get(key));
        let mut done = rec.and_then(|r| r["state"].as_str()) == Some("done");
        if key == "sheet" { done = next.is_none(); }
        let state = if done { "done" } else if next.is_none() { next = Some(key.to_string()); "active" } else { "pending" };
        steps.push(Step {
            key: key.into(), state: state.into(), title: title(key),
            needs: if done { vec![] } else { needs(key) },
            summary: rec.and_then(|r| serde_json::from_value(r["summary"].clone()).ok()),
        });
    }
    (steps, next)
}

pub fn mark_done(stored: Option<Value>, key: &str, summary: &Bi) -> Value {
    let mut v = stored.filter(|v| v.is_object()).unwrap_or_else(|| json!({}));
    v[key] = json!({ "state": "done", "summary": summary });
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_open_topic_is_active_and_sheet_waits() {
        let (s, next) = from_stored(None);
        assert_eq!(next.as_deref(), Some("field"));
        assert_eq!(s[0].state, "active");
        assert_eq!(s[5].state, "pending");
        let v = mark_done(None, "field", &Bi::new("pea after potato", "arveja después de papa"));
        let (s, next) = from_stored(Some(&v));
        assert_eq!(s[0].state, "done");
        assert_eq!(next.as_deref(), Some("soil"));
        let mut all = None;
        for k in ["field", "soil", "pests", "nutrition", "rotation"] { all = Some(mark_done(all, k, &Bi::new("x", "x"))); }
        let (s, next) = from_stored(all.as_ref());
        assert!(next.is_none());
        assert_eq!(s[5].state, "done");
    }
}
