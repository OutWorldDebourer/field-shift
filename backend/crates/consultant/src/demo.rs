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

pub async fn reply(state: &AppState, st: &AnalysisState, input: &TurnInput) -> anyhow::Result<String> {
    let es = input.lang == "es";
    let t = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    let label = t(LABEL_EN, LABEL_ES);
    if st.crop_current.is_none() {
        if let Some((cur, prev)) = parse_crops(&input.text) {
            evidence::resolve_crops(state, &st.id, &cur, &prev).await?;
            return Ok(format!("{label}\n\n{}", t(
                "Thank you. Now send 2 to 4 photos: the whole plant, and one leaf from above and from below, in daylight.",
                "Gracias. Ahora envía de 2 a 4 fotos: la planta entera y una hoja por arriba y por debajo, con luz de día.")));
        }
        return Ok(format!("{label}\n\n{}", t(
            "What are you growing now, and what grew here before? For example: \"peas now, lettuce before\".",
            "¿Qué cultivas ahora y qué hubo antes en este campo? Por ejemplo: \"arverja ahora, lechuga antes\".")));
    }
    // Once the summary has been given, move the conversation forward instead of repeating it.
    let summarised = st.messages.iter().any(|m| m.role == "assistant" && (m.body.contains("NASA data says") || m.body.contains("dicen los datos de la NASA")));
    if summarised {
        let photos = if input.photos.is_empty() { String::new() } else {
            t("The scripted demo does not look at photos; the live consultant checks each one and asks for a closer shot when needed.

",
              "El demo guionizado no revisa fotos; el consultor en vivo revisa cada una y pide una toma más cercana si hace falta.

")
        };
        return Ok(format!("{label}

{photos}{}", t(
            "Your pre-diagnosis is in the panels on the right: Soil, Pest care, Nutrition and Rotation, each with its sources. Two questions would sharpen it: where does your water come from, and what fertilizer or manure did you use last? \"I don't know\" is fine; I will note it as a gap.",
            "Tu prediagnóstico está en los paneles de la derecha: Suelo, Plagas, Nutrición y Rotación, cada uno con sus fuentes. Dos preguntas lo afinarían: ¿de dónde viene tu agua y qué abono o estiércol usaste por última vez? \"No sé\" está bien; lo anoto como vacío.")));
    }
    let ev = evidence::build(state, &st.id).await?;
    evidence::advise(state, &st.id).await?;
    let mut lines = vec![];
    if let Some(rs) = ev["readings"].as_array() {
        for r in rs.iter().filter(|r| r["key"] == true || ["temp_anomaly", "soil_moisture", "ndvi"].contains(&r["id"].as_str().unwrap_or(""))).take(3) {
            let label = r.pointer(if es { "/label/es" } else { "/label/en" }).and_then(Value::as_str).unwrap_or("");
            let verdict = r.pointer(if es { "/verdict/es" } else { "/verdict/en" }).and_then(Value::as_str).unwrap_or("");
            lines.push(format!("· {label}: {verdict} ({}, {})", r["source"].as_str().unwrap_or(""), r["date"].as_str().unwrap_or("")));
        }
    }
    let photos = if input.photos.is_empty() { String::new() } else {
        t("The scripted demo does not look at photos; the live consultant checks each one.\n", "El demo guionizado no revisa fotos; el consultor en vivo revisa cada una.\n")
    };
    Ok(format!("{label}\n\n{photos}{}\n{}\n\n{}", t("Here is what NASA data says about your field:", "Esto dicen los datos de la NASA sobre tu campo:"), lines.join("\n"),
        t("The Soil, Pest care, Nutrition and Rotation panels are filled. This is a pre-diagnosis: confirm it with your agronomist.",
          "Los paneles de Suelo, Plagas, Nutrición y Rotación están listos. Es un prediagnóstico: confírmalo con tu agrónomo.")))
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
