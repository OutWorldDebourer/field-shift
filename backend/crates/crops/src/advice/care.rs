//! Soil, pest care and nutrition guidance blocks, grounded in the agronomist interview themes and cited sources.
use super::*;

fn item(en: impl Into<String>, es: impl Into<String>, citations: Vec<Citation>, interview: bool) -> Item {
    Item { text: L10n::new(en, es), citations, basis: interview.then(|| refs::INTERVIEW.to_string()) }
}

pub fn soil(c: &CropRecord, s: &FieldSignals) -> Block {
    let mut items = vec![
        item("Feed the soil with compost, humus or well-made manure every season; heavy use of agrochemicals harms the soil microorganisms that make nutrients available.",
             "Alimente el suelo con compost, humus o estiércol bien descompuesto cada campaña; el uso excesivo de agroquímicos daña a los microorganismos del suelo que ponen los nutrientes a disposición.",
             vec![refs::sare_cover_crops()], true),
        item("Use compost or humus enriched with Trichoderma: this beneficial fungus protects roots against soil-borne diseases and supports nutrition.",
             "Use compost o humus enriquecido con Trichoderma: este hongo benéfico protege las raíces contra enfermedades del suelo y favorece la nutrición.",
             vec![refs::trichoderma_review(), refs::senasa_biocontrol()], true),
        item("Grow green manures (legumes, alone or mixed with a cereal) and incorporate them before they mature.",
             "Siembre abonos verdes (leguminosas solas o con un cereal) e incorpórelos antes de que maduren.",
             vec![refs::sare_cover_crops()], true),
    ];
    if s.bare_soil == Some(true) {
        items.push(item("Satellite images show periods of bare soil on this field: cover it between crops (cover crop or mulch) to protect soil life.",
                        "Las imágenes satelitales muestran periodos de suelo desnudo en este campo: cúbralo entre cultivos (cobertura o mulch) para proteger la vida del suelo.",
                        vec![refs::sare_cover_crops()], false));
    }
    if c.crop_type == CropType::Perennial {
        items.push(item("In a permanent plantation, build soil between rows and under the canopy with mulch and covers.",
                        "En una plantación permanente, recupere el suelo entre hileras y bajo la copa con mulch y coberturas.",
                        vec![refs::sare_cover_crops()], false));
    }
    Block { title: L10n::new("Soil", "Suelo"), items }
}

pub fn pest_care(c: &CropRecord, s: &FieldSignals) -> Block {
    let mut items = vec![];
    let warmer = s.pest_generation_delta.is_some_and(|g| g > 0.0) || s.heat_stress_days.is_some_and(|d| d > 0.0);
    items.push(item(
        if warmer { "Monitor pests every week: with this season's heat, pests develop faster and populations grow sooner." } else { "Monitor pests every week, especially in warm spells: pests develop faster with heat." },
        if warmer { "Monitoree las plagas cada semana: con el calor de esta campaña se desarrollan más rápido y las poblaciones crecen antes." } else { "Monitoree las plagas cada semana, sobre todo en épocas de calor: con más calor se desarrollan más rápido." },
        vec![refs::temperature_pest_development()], true));
    if let Some(g) = s.pest_generation_delta.filter(|g| *g != 0.0) {
        items.push(item(format!("Estimated pest generations versus the 40-year baseline: {g:+.1} per season (NASA POWER degree-days, regional context)."),
                        format!("Generaciones de plaga estimadas frente a la línea base de 40 años: {g:+.1} por campaña (grados-día NASA POWER, contexto regional)."), vec![], false));
    }
    if c.crop_type == CropType::Annual {
        items.push(item("Rotate to a crop of another botanical family to break pest and disease cycles.",
                        "Rote a un cultivo de otra familia botánica para cortar el ciclo de plagas y enfermedades.",
                        vec![refs::sare_rotation()], true));
    }
    items.push(item("Incorporate or compost crop residues soon after harvest; do not pile them at the field edge, where they become a source of pests and diseases, and avoid burning.",
                    "Incorpore o composte los residuos poco después de la cosecha; no los amontone en el borde del campo, donde se vuelven foco de plagas y enfermedades, y evite quemarlos.",
                    vec![refs::residue_incorporation()], true));
    for b in &c.traits.beneficials {
        items.push(item(format!("Biological control: {} ({}) against {}.", b.name.en, b.kind, b.controls.en),
                        format!("Control biológico: {} ({}) contra {}.", b.name.es, b.kind, b.controls.es),
                        vec![b.citation.clone()], true));
    }
    let computable: Vec<&str> = c.traits.pests.iter()
        .filter(|p| p.lower_threshold.num().is_some() && p.degree_days_per_generation.num().is_some())
        .map(|p| p.name.en.as_str()).collect();
    let qualitative: Vec<String> = c.traits.pests.iter()
        .filter(|p| p.lower_threshold.num().is_none() || p.degree_days_per_generation.num().is_none())
        .map(|p| p.name.en.clone()).collect();
    if !qualitative.is_empty() {
        items.push(item(format!("This crop's knowledge record has no published degree-day threshold for: {}. Guidance for these pests is qualitative unless the reference-pest table covers them.", qualitative.join(", ")),
                        format!("La ficha de este cultivo no tiene umbral de grados-día publicado para: {}. La guía para estas plagas es cualitativa salvo que la tabla de plagas de referencia las incluya.",
                                c.traits.pests.iter().filter(|p| p.lower_threshold.num().is_none() || p.degree_days_per_generation.num().is_none())
                                    .map(|p| p.name.es.as_str()).collect::<Vec<_>>().join(", ")),
                        vec![], false));
    }
    if !computable.is_empty() {
        let cits = c.traits.pests.iter().filter(|p| computable.contains(&p.name.en.as_str()))
            .filter_map(|p| p.degree_days_per_generation.citation().cloned()).collect();
        items.push(item(format!("Published degree-day thresholds exist for: {} (cited). This version computes generations only for the reference pests in the table.", computable.join(", ")),
                        format!("Existen umbrales de grados-día publicados para: {} (citados). Esta versión calcula generaciones solo para las plagas de referencia de la tabla.",
                                c.traits.pests.iter().filter(|p| computable.contains(&p.name.en.as_str())).map(|p| p.name.es.as_str()).collect::<Vec<_>>().join(", ")),
                        cits, false));
    }
    Block { title: L10n::new("Pest care", "Sanidad y plagas"), items }
}

pub fn nutrition(c: &CropRecord, previous: &[CropRecord]) -> Block {
    let mut items = vec![];
    if let Some(l) = previous.iter().rev().find(|p| p.is_legume()) {
        let range = l.traits.n_fixation.typical_range.display();
        let cits: Vec<Citation> = l.traits.n_fixation.typical_range.citation().cloned().into_iter().chain([refs::sare_rotation()]).collect();
        items.push(item(
            format!("The previous {} (legume) left nitrogen in the soil{}; ask your agronomist how much N fertilizer you can save.",
                    l.name_en(), range.as_ref().map(|r| format!(" (it fixes about {r})")).unwrap_or_default()),
            format!("La {} anterior (leguminosa) dejó nitrógeno en el suelo{}; pregunte a su agrónomo cuánto fertilizante nitrogenado puede ahorrar.",
                    l.name_es(), range.as_ref().map(|r| format!(" (fija alrededor de {r})")).unwrap_or_default()),
            cits, false));
    }
    if c.is_legume() {
        let range = c.traits.n_fixation.typical_range.display();
        items.push(item(
            format!("{} fixes its own nitrogen{}; the crop that follows benefits from it.", c.name_en(), range.as_ref().map(|r| format!(" (about {r})")).unwrap_or_default()),
            format!("{} fija su propio nitrógeno{}; el cultivo siguiente se beneficia.", c.name_es(), range.as_ref().map(|r| format!(" (alrededor de {r})")).unwrap_or_default()),
            c.traits.n_fixation.typical_range.citation().cloned().into_iter().chain([refs::nmsu_legume_n()]).collect(), false));
    }
    items.push(item("Base nutrition on organic matter (compost, humus, green manures) and adjust fertilizer with a soil analysis.",
                    "Base la nutrición en materia orgánica (compost, humus, abonos verdes) y ajuste el fertilizante con un análisis de suelo.",
                    vec![refs::sare_cover_crops()], true));
    items.push(item("No soil analysis in the evidence package: fertilizer doses are an information gap.",
                    "No hay análisis de suelo en el paquete de evidencias: las dosis de fertilizante son un vacío de información.", vec![], false));
    Block { title: L10n::new("Nutrition", "Nutrición"), items }
}
