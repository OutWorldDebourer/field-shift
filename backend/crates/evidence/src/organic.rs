//! The "Organic" section: biological and ecological pest management. Natural enemies and beneficial microorganisms
//! (each with its citation) for this crop and for the pests the farmer reported, matched across the whole knowledge
//! base; practices that keep them working; when to act from NASA data; and what harms them. No product doses.
use crops::advice::refs;
use crops::CropRecord;
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Words a farmer may use for a pest → words that appear in the "controls" text of the knowledge base.
const SYNONYMS: &[(&str, &[&str])] = &[
    ("arañita", &["spider mite", "tetranychus", "mite", "ácaro", "arañita"]),
    ("spider mite", &["spider mite", "tetranychus", "mite", "ácaro"]),
    ("ácaro", &["mite", "ácaro", "tetranychus"]),
    ("pulgón", &["aphid", "pulgón", "pulgon"]),
    ("aphid", &["aphid", "pulgón"]),
    ("insectos verdes", &["aphid", "pulgón"]),
    ("small green insects", &["aphid"]),
    ("mosca blanca", &["whitefly", "mosca blanca", "bemisia"]),
    ("whitefly", &["whitefly", "bemisia"]),
    ("minador", &["leafminer", "liriomyza", "minadora"]),
    ("galería", &["leafminer", "liriomyza", "minadora"]),
    ("tunnel", &["leafminer", "liriomyza"]),
    ("leafminer", &["leafminer", "liriomyza"]),
    ("gusano", &["armyworm", "caterpillar", "lepidopter", "larva", "spodoptera", "gusano"]),
    ("caterpillar", &["caterpillar", "lepidopter", "armyworm"]),
    ("trips", &["thrips"]),
    ("thrips", &["thrips"]),
    ("chupadera", &["damping-off", "chupadera", "root rot"]),
    ("pudrición", &["root rot", "pudrición", "rot"]),
    ("hongo", &["fung", "rot", "mildew"]),
    ("cochinilla", &["mealybug", "scale", "cochinilla"]),
    ("mosca de la fruta", &["fruit fly", "anastrepha", "ceratitis"]),
];

fn keywords(text: &str) -> Vec<&'static str> {
    let l = text.to_lowercase();
    let mut out = vec![];
    for (k, words) in SYNONYMS { if l.contains(k) { out.extend_from_slice(words); } }
    out.sort();
    out.dedup();
    out
}

fn agent(b: &crops::model::Beneficial, crop: &CropRecord, reported: bool) -> Value {
    json!({
        "name": b.name, "scientific": b.scientific_name, "kind": b.kind, "targets": b.controls,
        "for_reported_pest": reported, "crop": crop.names.en.first().cloned().unwrap_or_default(),
        "citation": { "title": b.citation.title, "url": b.citation.url }, "note": b.note,
    })
}

fn cite(c: crops::model::Citation) -> Value { json!({ "title": c.title, "url": c.url }) }

pub fn build(crop: &CropRecord, facts: Option<&Value>, ev: &Value) -> Value {
    let reported = facts.and_then(|f| f["pests_seen"].as_str()).unwrap_or("");
    let kws = keywords(reported);
    let matches = |b: &crops::model::Beneficial| {
        let t = format!("{} {}", b.controls.en, b.controls.es).to_lowercase();
        !kws.is_empty() && kws.iter().any(|k| t.contains(k))
    };
    let mut seen = BTreeSet::new();
    let mut agents = vec![];
    // 1) This crop's own allies, 2) allies of the reported pests recorded on other crops.
    for b in &crop.traits.beneficials {
        if seen.insert(format!("{}|{}", b.scientific_name, b.name.en)) { agents.push(agent(b, crop, matches(b))); }
    }
    for other in crops::seed::all() {
        for b in &other.traits.beneficials {
            if matches(b) && agents.len() < 18 && seen.insert(format!("{}|{}", b.scientific_name, b.name.en)) {
                agents.push(agent(b, other, true));
            }
        }
    }
    // Reported-pest matches first, then the rest.
    agents.sort_by_key(|a| if a["for_reported_pest"] == true { 0 } else { 1 });

    let practices = json!([
        { "title": { "en": "Feed the soil life: Trichoderma-enriched compost or humus", "es": "Alimente la vida del suelo: compost o humus enriquecido con Trichoderma" },
          "body": { "en": "Trichoderma is a beneficial fungus that protects roots from soil-borne diseases and helps nutrition; it is added to compost or humus before use.", "es": "Trichoderma es un hongo benéfico que protege las raíces de enfermedades del suelo y ayuda a la nutrición; se incorpora al compost o humus antes de usarlo." },
          "citations": [cite(refs::trichoderma_review())], "basis": "agronomist interview" },
        { "title": { "en": "Give natural enemies a home: flowering borders and cover crops", "es": "Dé refugio a los enemigos naturales: bordes con flores y cultivos de cobertura" },
          "body": { "en": "Cover crops and flowering strips keep lady beetles, lacewings and parasitoid wasps near the field between crops.", "es": "Los cultivos de cobertura y las franjas con flores mantienen mariquitas, crisopas y avispitas parasitoides cerca del campo entre campañas." },
          "citations": [cite(refs::sare_cover_crops())] },
        { "title": { "en": "Clean up residues the right way", "es": "Maneje bien los rastrojos" },
          "body": { "en": "Incorporate or compost crop residues soon after harvest; piles at the field edge become a source of pests and diseases, and burning harms the soil.", "es": "Incorpore o composte los rastrojos poco después de la cosecha; los montones al borde del campo se vuelven foco de plagas y enfermedades, y la quema daña el suelo." },
          "citations": [cite(refs::residue_incorporation())], "basis": "agronomist interview" },
        { "title": { "en": "Break pest cycles with rotation", "es": "Corte el ciclo de las plagas con rotación" },
          "body": { "en": "Rotating to a crop of another botanical family leaves pests without their host.", "es": "Rotar a un cultivo de otra familia botánica deja a las plagas sin su hospedero." },
          "citations": [cite(refs::sare_rotation())] },
        { "title": { "en": "Biological controllers in Peru", "es": "Controladores biológicos en el Perú" },
          "body": { "en": "SENASA produces and supports biological controllers (beneficial insects and fungi) for farmers; ask your agronomist or the local SENASA office which ones are available for your pest.", "es": "SENASA produce y promueve controladores biológicos (insectos y hongos benéficos) para agricultores; consulte a su agrónomo o a la oficina local de SENASA cuáles hay para su plaga." },
          "citations": [cite(refs::senasa_biocontrol())] },
    ]);

    // When to act: NASA degree-days (pest generations) and the GEOS-FP forecast.
    let mut timing: Vec<Value> = vec![];
    for p in ev["pests"].as_array().into_iter().flatten() {
        if let (Some(now), Some(norm)) = (p["generations_now"].as_f64(), p["generations_normal"].as_f64()) {
            if now - norm >= 1.0 {
                let en = p["name_bi"]["en"].as_str().or(p["name"].as_str()).unwrap_or("");
                let es = p["name_bi"]["es"].as_str().unwrap_or(en);
                timing.push(json!({
                    "en": format!("{en}: {:.1} generations this season vs {:.1} normally (NASA POWER degree-days). Check twice a week and protect or release its natural enemies early, while numbers are low.", now, norm),
                    "es": format!("{es}: {:.1} generaciones esta campaña vs {:.1} lo normal (grados-día NASA POWER). Revise dos veces por semana y proteja o libere a sus enemigos naturales temprano, cuando aún son pocas.", now, norm) }));
            }
        }
    }
    if let Some(days) = ev["forecast"]["days"].as_array() {
        if let Some(hot) = days.iter().filter(|d| d["tmax"].as_f64().unwrap_or(0.0) >= 29.0).max_by(|a, b| a["tmax"].as_f64().partial_cmp(&b["tmax"].as_f64()).unwrap()) {
            timing.push(json!({
                "en": format!("Warm days ahead (up to {:.0} °C on {}, NASA GEOS-FP forecast): pests develop faster in heat, so inspect leaf undersides more often.", hot["tmax"].as_f64().unwrap_or(0.0), hot["date"].as_str().unwrap_or("")),
                "es": format!("Vienen días cálidos (hasta {:.0} °C el {}, pronóstico NASA GEOS-FP): con calor las plagas se desarrollan más rápido; revise el envés de las hojas más seguido.", hot["tmax"].as_f64().unwrap_or(0.0), hot["date"].as_str().unwrap_or("")) }));
        }
    }
    if timing.is_empty() {
        timing.push(json!({ "en": "Inspect the crop every week, especially the underside of young leaves.", "es": "Revise el cultivo cada semana, sobre todo el envés de las hojas tiernas." }));
    }
    let avoid = json!([
        { "en": "Broad-spectrum insecticides also kill lady beetles, lacewings, parasitoid wasps and soil life; frequent sprays raise costs. Ask your agronomist for selective options first (agronomist interview).", "es": "Los insecticidas de amplio espectro también matan mariquitas, crisopas, avispitas parasitoides y la vida del suelo; aplicar seguido sube los costos. Pida primero a su agrónomo opciones selectivas (entrevista a la agrónoma)." },
        { "en": "Burning residues destroys organic matter and beneficial organisms.", "es": "Quemar los rastrojos destruye la materia orgánica y los organismos benéficos." },
    ]);
    let n_rep = agents.iter().filter(|a| a["for_reported_pest"] == true).count();
    json!({
        "title": { "en": "Organic and biological pest management", "es": "Manejo orgánico y biológico de plagas" },
        "summary": {
            "en": format!("{} natural enemies and beneficial microorganisms with published sources{}. Release rates and products are decided with your agronomist.", agents.len(), if n_rep > 0 { format!(", {n_rep} of them for the pests you reported") } else { String::new() }),
            "es": format!("{} enemigos naturales y microorganismos benéficos con fuente publicada{}. Las dosis y productos se deciden con su agrónomo.", agents.len(), if n_rep > 0 { format!(", {n_rep} para las plagas que reportó") } else { String::new() }),
        },
        "agents": agents, "practices": practices, "timing": timing, "avoid": avoid,
        "gaps": [{ "en": "Local availability and prices of biological controllers are not in the data: ask SENASA or your agronomist.", "es": "La disponibilidad y precios locales de controladores biológicos no están en los datos: consulte a SENASA o a su agrónomo." }],
    })
}
