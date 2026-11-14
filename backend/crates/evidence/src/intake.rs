//! "Load information": structured answers that feed the five guided topics at once. Resolves the crops, records the
//! facts, completes every topic the answers cover ("I don't know" counts as a declared gap), builds the evidence and
//! advice, and leaves a short consultant message saying what was loaded and what comes next.
use analyses::{store, Bi};
use common::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IntakeForm {
    #[serde(default)] pub crop_now: Option<String>,
    #[serde(default)] pub crops_before: Vec<String>,
    #[serde(default)] pub irrigation: Option<String>,
    #[serde(default)] pub soil_type: Option<String>,
    #[serde(default)] pub organic_matter: Option<String>,
    #[serde(default)] pub residues: Option<String>,
    #[serde(default)] pub pests_seen: Option<String>,
    #[serde(default)] pub products_recent: Option<String>,
    #[serde(default)] pub last_fertilizer: Option<String>,
    #[serde(default)] pub soil_test: Option<String>,
    #[serde(default)] pub next_crop: Option<String>,
    #[serde(default)] pub harvest_use: Option<String>,
}

fn val(o: &Option<String>) -> Option<&str> { o.as_deref().map(str::trim).filter(|s| !s.is_empty()) }

fn unknown(s: &str) -> bool {
    let l = s.to_lowercase();
    ["no sé", "no se", "don't know", "dont know", "unknown", "desconocido", "ninguno", "none"].iter().any(|k| l.contains(k))
}

/// "label: value; label: value" in each language, for the fields that were answered.
fn summary(parts: &[(&str, &str, Option<&str>)]) -> Option<Bi> {
    let got: Vec<_> = parts.iter().filter_map(|(en, es, v)| v.map(|v| (*en, *es, v))).collect();
    if got.is_empty() { return None; }
    Some(Bi::new(
        got.iter().map(|(en, _, v)| format!("{en}: {v}")).collect::<Vec<_>>().join("; "),
        got.iter().map(|(_, es, v)| format!("{es}: {v}")).collect::<Vec<_>>().join("; "),
    ))
}

pub async fn apply(state: &AppState, id: &str, f: &IntakeForm, lang: &str) -> anyhow::Result<()> {
    store::merge_facts(&state.db, id, &serde_json::to_value(f)?).await.map_err(|x| anyhow::anyhow!("{x}"))?;
    let es = lang == "es";
    store::add_message(&state.db, id, "user",
        if es { "📋 Cargué la información del formulario." } else { "📋 I loaded the information from the form." }, &[]).await.map_err(|x| anyhow::anyhow!("{x}"))?;

    if let Some(c) = val(&f.crop_now) {
        crate::resolve_crops(state, id, c, &f.crops_before).await?;
    }
    let before = if f.crops_before.is_empty() { None } else { Some(f.crops_before.join(", ")) };
    let topics: [(&str, Option<Bi>); 5] = [
        ("field", val(&f.crop_now).and(summary(&[("crop now", "cultivo actual", val(&f.crop_now)), ("before", "antes", before.as_deref()), ("irrigation", "riego", val(&f.irrigation))]))),
        ("soil", summary(&[("soil", "suelo", val(&f.soil_type)), ("organic matter", "materia orgánica", val(&f.organic_matter)), ("residues", "rastrojos", val(&f.residues))])),
        ("pests", summary(&[("seen", "observado", val(&f.pests_seen)), ("recent products", "productos recientes", val(&f.products_recent))])),
        ("nutrition", summary(&[("last fertilizer", "último abono", val(&f.last_fertilizer)), ("soil test", "análisis de suelo", val(&f.soil_test))])),
        ("rotation", summary(&[("next crop", "próximo cultivo", val(&f.next_crop)), ("harvest use", "destino de la cosecha", val(&f.harvest_use))])),
    ];
    let mut done = vec![];
    for (key, s) in topics.iter() {
        if let Some(s) = s {
            store::complete_step(&state.db, id, key, s).await.map_err(|x| anyhow::anyhow!("{x}"))?;
            done.push(*key);
        }
    }
    // Answers that are "I don't know" become declared gaps on the sheet.
    let gaps: Vec<Value> = [(&f.last_fertilizer, "Last fertilizer not known", "Último abono desconocido"), (&f.soil_test, "No soil test available", "Sin análisis de suelo"), (&f.pests_seen, "Pests not identified yet", "Plagas aún no identificadas")]
        .iter().filter(|(v, _, _)| val(v).map(unknown).unwrap_or(false)).map(|(_, en, es)| serde_json::json!({ "en": en, "es": es })).collect();
    if !gaps.is_empty() {
        let secs = store::sections(&state.db, id).await.map_err(|x| anyhow::anyhow!("{x}"))?;
        if let Some(fs) = secs.iter().find(|x| x.key == "field") {
            let mut c = fs.content.clone().unwrap_or_else(|| serde_json::json!({ "title": { "en": "Your field", "es": "Tu campo" }, "summary": { "en": "", "es": "" }, "items": [] }));
            c["declared_gaps"] = Value::Array(gaps);
            store::set_section(&state.db, id, "field", &fs.state, fs.note.as_ref(), Some(&c)).await.map_err(|x| anyhow::anyhow!("{x}"))?;
        }
    }
    if val(&f.crop_now).is_some() {
        crate::build(state, id).await?;
        crate::advise(state, id).await?;
    }
    let st = store::load(&state.db, id).await.map_err(|x| anyhow::anyhow!("{x}"))?;
    let title = |k: &str| { let t = analyses::steps::title(k); if es { t.es } else { t.en } };
    let done_names = done.iter().map(|k| title(k)).collect::<Vec<_>>().join(", ");
    let next = st.next_step.as_deref().and_then(|k| st.steps.iter().find(|x| x.key == k)).map(|s| {
        let need = s.needs.first().map(|n| if es { n.es.clone() } else { n.en.clone() }).unwrap_or_default();
        if es { format!("Siguiente tema: **{}** — ¿{}?", if es { &s.title.es } else { &s.title.en }, need) } else { format!("Next topic: **{}** — {}?", s.title.en, need) }
    });
    let body = match (done.is_empty(), next) {
        (true, _) => if es { "No encontré respuestas en el formulario. Cuéntame qué cultivas ahora y qué hubo antes.".to_string() } else { "I found no answers in the form. Tell me what you grow now and what grew before.".to_string() },
        (false, Some(n)) => if es { format!("Gracias, registré: {done_names}. Con eso preparé esas secciones con datos de la NASA y fuentes citadas.\n\n{n}") } else { format!("Thanks, I recorded: {done_names}. With that I prepared those sections from NASA data and cited sources.\n\n{n}") },
        (false, None) => if es { "Gracias, con esto ya tenemos suficiente información para todos los temas. Tu ficha técnica está lista en el panel de la derecha, y la pestaña Orgánico reúne los controladores biológicos para tus plagas. Es un prediagnóstico: confírmalo con tu agrónomo.".to_string() } else { "Thanks, we now have enough information for every topic. Your technical sheet is ready in the panel on the right, and the Organic tab gathers the biological controls for your pests. This is a pre-diagnosis: confirm it with your agronomist.".to_string() },
    };
    store::add_message(&state.db, id, "assistant", &body, &[]).await.map_err(|x| anyhow::anyhow!("{x}"))?;
    Ok(())
}
