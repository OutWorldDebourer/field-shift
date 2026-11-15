//! Scripted demo mode: a clearly labelled, deterministic consultant used when the live model is unavailable,
//! rate-limited, or explicitly requested. It never looks at photos and only quotes numbers from the evidence package.
use crate::turn::TurnInput;
use analyses::AnalysisState;
use common::AppState;
use serde_json::Value;

pub const LABEL_EN: &str = "Demo mode (scripted answers, the live consultant is off).";
pub const LABEL_ES: &str = "Modo demo (respuestas guionizadas, el consultor en vivo está apagado).";

/// Splits "peas now, lettuce before" / "arverjita ahora, antes lechuga" into (current, previous).
pub fn parse_crops(text: &str) -> Option<(String, Vec<String>)> {
    let lower = text.to_lowercase();
    let markers = ["before", "previous", "antes", "anterior", "last season", "la campaña pasada"];
    let cut = markers.iter().filter_map(|m| lower.find(m).map(|i| (i, m.len()))).min();
    let clean = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphabetic() && c != ' ')
            .flat_map(|p| p.split(" and ").flat_map(|q| q.split(" y ")))
            .map(|w| w.split_whitespace().filter(|t| !STOP.contains(t)).collect::<Vec<_>>().join(" "))
            .filter(|w| w.len() > 2).collect()
    };
    let (now, before) = match cut {
        Some((i, len)) => {
            let (a, b) = (&lower[..i], &lower[i + len..]);
            // "lettuce before" puts the crop ahead of the marker: take the last item of the first half as previous.
            let mut a_items = clean(a);
            let b_items = clean(b);
            if b_items.is_empty() && a_items.len() >= 2 { let p = a_items.pop().unwrap(); (a_items, vec![p]) } else { (a_items, b_items) }
        }
        None => (clean(&lower), vec![]),
    };
    let current = now.into_iter().next()?;
    Some((current, before))
}

const STOP: [&str; 24] = ["now", "ahora", "i", "grow", "growing", "we", "have", "tengo", "sembré", "planted", "is", "es", "el", "la", "los", "las", "de", "en", "mi", "my", "field", "campo", "there", "hay"];

/// The scripted reply; the demo label is said once per conversation (the header flag keeps showing it).
pub async fn reply(state: &AppState, st: &AnalysisState, input: &TurnInput) -> anyhow::Result<String> {
    reply_raw(state, st, input).await.map(|s| s.trim_start().to_string())
}

async fn reply_raw(state: &AppState, st: &AnalysisState, input: &TurnInput) -> anyhow::Result<String> {
    let es = input.lang == "es";
    let t = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    let said = st.messages.iter().any(|m| m.role == "assistant" && (m.body.starts_with(LABEL_EN) || m.body.starts_with(LABEL_ES)));
    let label = if said { String::new() } else { t(LABEL_EN, LABEL_ES) };
    let photos = if input.photos.is_empty() { String::new() } else {
        t("The scripted demo does not look at photos; the live consultant checks each one and asks for a closer shot when needed.

",
          "El demo guionizado no revisa fotos; el consultor en vivo revisa cada una y pide una toma más cercana si hace falta.

")
    };
    let pick = |b: &analyses::Bi| if es { b.es.clone() } else { b.en.clone() };
    // What to ask next, from the guided topics.
    let ask_next = |next: Option<String>, st2: &AnalysisState| -> String {
        match next.and_then(|k| st2.steps.iter().find(|x| x.key == k).cloned()) {
            Some(step) => {
                let need = step.needs.first().map(&pick).unwrap_or_default();
                if es { format!("Siguiente tema: **{}** — ¿{}?", pick(&step.title), need) } else { format!("Next topic: **{}** — {}?", pick(&step.title), need) }
            }
            None => t("We now have enough information. Your technical sheet is ready in the panel on the right; the dashboard shows the forecast and the 40-year comparison, Prevention shows each pest's risk and Organic its natural allies. This is a pre-diagnosis: confirm it with your agronomist.",
                      "Ya tenemos suficiente información. Tu ficha técnica está lista en el panel de la derecha; el panel de datos muestra el pronóstico y la comparación con 40 años, Prevención el riesgo de cada plaga y Orgánico sus aliados naturales. Es un prediagnóstico: confírmalo con tu agrónomo."),
        }
    };
    if st.crop_current.is_none() {
        let Some((cur, prev)) = parse_crops(&input.text) else {
            return Ok(format!("{label}

{}", t(
                "What are you growing now, and what grew here before? For example: \"peas now, lettuce before\".",
                "¿Qué cultivas ahora y qué hubo antes en este campo? Por ejemplo: \"arveja ahora, lechuga antes\".")));
        };
        evidence::resolve_crops(state, &st.id, &cur, &prev).await?;
        let ev = evidence::build(state, &st.id).await?;
        let summary = analyses::Bi::new(format!("{cur} now; before: {}", if prev.is_empty() { "not stated".into() } else { prev.join(", ") }),
                                        format!("{cur} ahora; antes: {}", if prev.is_empty() { "no indicado".into() } else { prev.join(", ") }));
        let next = analyses::store::complete_step(&state.db, &st.id, "field", &summary).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        evidence::advise(state, &st.id).await?;
        let st2 = analyses::store::load(&state.db, &st.id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut lines = vec![];
        for r in ev["readings"].as_array().into_iter().flatten().filter(|r| ["temp_anomaly", "soil_moisture", "ndvi"].contains(&r["id"].as_str().unwrap_or(""))).take(3) {
            let label = r.pointer(if es { "/label/es" } else { "/label/en" }).and_then(Value::as_str).unwrap_or("");
            let verdict = r.pointer(if es { "/verdict/es" } else { "/verdict/en" }).and_then(Value::as_str).unwrap_or("");
            let badge = match (r["badge"].as_str().unwrap_or(""), es) {
                ("field-level", false) => "field-level".to_string(), ("field-level", true) => "nivel de campo".to_string(),
                (_, false) => format!("regional context, {}", r["resolution"].as_str().unwrap_or("")), (_, true) => format!("contexto regional, {}", r["resolution"].as_str().unwrap_or("")),
            };
            let prov = if r["provisional"] == true { if es { ", provisional" } else { ", provisional" } } else { "" };
            lines.push(format!("· {label}: {verdict} ({}, {}; {badge}{prov})", r["source"].as_str().unwrap_or(""), r["date"].as_str().unwrap_or("")));
        }
        return Ok(format!("{label}

{photos}{}
{}

{}", t("Here is what NASA data says about the plot you marked:", "Esto dicen los datos de la NASA sobre la parcela que marcaste:"), lines.join("
"), ask_next(next, &st2)));
    }
    let Some(active) = st.next_step.clone() else { return Ok(format!("{label}

{photos}{}", ask_next(None, st))) };
    let said: String = input.text.trim().chars().take(90).collect();
    let summary = analyses::Bi::new(format!("Farmer said: {said}"), format!("El agricultor dijo: {said}"));
    let next = analyses::store::complete_step(&state.db, &st.id, &active, &summary).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let _ = evidence::advise(state, &st.id).await;
    let st2 = analyses::store::load(&state.db, &st.id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(format!("{label}

{photos}{} {}", t("Noted.", "Anotado."), ask_next(next, &st2)))
}

#[cfg(test)]
mod tests {
    use super::parse_crops;
    #[test]
    fn parses_common_answers() {
        assert_eq!(parse_crops("Peas now. Lettuce before."), Some(("peas".into(), vec!["lettuce".into()])));
        assert_eq!(parse_crops("arverjita ahora, antes lechuga"), Some(("arverjita".into(), vec!["lechuga".into()])));
        assert_eq!(parse_crops("grapes"), Some(("grapes".into(), vec![])));
    }
}
