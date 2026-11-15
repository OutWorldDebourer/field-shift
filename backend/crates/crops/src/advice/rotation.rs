//! Annual rotation strategies (current + 2 alternatives) and semi-perennial renewal options, scored 1–5.
use super::*;
use crate::model::RotationNotes;
use crate::seed;

/// Annual crops grown on the Peruvian coast that the alternatives may propose, in preference order.
const LEGUMES: &[&str] = &["pea", "common_bean"];
const OTHERS: &[&str] = &["maize", "quinoa", "sweet_potato", "onion", "lettuce", "potato", "tomato", "cotton"];

/// The different-family crop to propose next: a legume after a non-legume, otherwise a non-legume
/// (cereal first) to use the N credit; avoids every family already in the sequence when possible.
fn next_crop(current: &CropRecord, previous: &[CropRecord]) -> &'static CropRecord {
    let used: Vec<&str> = previous.iter().map(|p| p.family.as_str()).chain([current.family.as_str()]).collect();
    let order: Vec<&str> = if current.is_legume() { OTHERS.to_vec() } else { LEGUMES.iter().chain(OTHERS).copied().collect() };
    let pick = |avoid_all: bool| order.iter().filter_map(|k| seed::get(k)).find(|c| {
        c.key != current.key && if avoid_all { !used.contains(&c.family.as_str()) } else { c.family != current.family }
    });
    pick(true).or_else(|| pick(false)).unwrap_or_else(|| seed::get("maize").expect("maize seed"))
}

fn seq(previous: &[CropRecord], tail: Vec<L10n>) -> Vec<L10n> {
    previous.iter().map(CropRecord::name).chain(tail).collect()
}

fn kc_mid(c: &CropRecord) -> Option<f64> { c.traits.kc.mid.num() }

fn rotation_note(c: &CropRecord) -> Option<Evidence> {
    match &c.traits.rotation_notes {
        RotationNotes::Note { en, es, citation } => Some(Evidence {
            kind: EvidenceKind::Trait,
            text: L10n::new(format!("{}: {en}", c.name_en()), format!("{}: {es}", c.name_es())),
            citation: Some(citation.clone()),
        }),
        RotationNotes::Gap { .. } => None,
    }
}

fn signals_evidence(s: &FieldSignals) -> Vec<Evidence> {
    let mut v = vec![];
    if let Some(d) = s.heat_stress_days { v.push(signal(format!("Heat-stress days this season: {d:.0}"), format!("Días de estrés por calor esta campaña: {d:.0}"))); }
    if let Some(w) = s.water_balance_mm { v.push(signal(format!("Water balance: {w:.0} mm"), format!("Balance hídrico: {w:.0} mm"))); }
    if let (Some(g), Some(n)) = (s.pest_generation_delta, s.pest_name.as_ref()) {
        v.push(signal(format!("{} generations vs 40-year normal: {g:+.1}", n.en), format!("Generaciones de {} frente a la normal de 40 años: {g:+.1}", n.es.to_lowercase())));
    }
    v
}

/// Water score relative to the current crop: lower mid-season Kc saves water (more so under a deficit).
fn water_score(current: &CropRecord, other: Option<&CropRecord>, s: &FieldSignals, gaps: &mut Vec<L10n>) -> u8 {
    let deficit = s.water_balance_mm.is_some_and(|w| w < 0.0);
    let Some(o) = other else { return clamp(if deficit { 2 } else { 3 }) };
    match (kc_mid(current), kc_mid(o)) {
        (Some(a), Some(b)) => {
            let d = if b < a - 0.05 { 1 } else if b > a + 0.05 { -1 } else { 0 };
            clamp(3 + d + if deficit { d } else { 0 })
        }
        _ => {
            gaps.push(L10n::new("Kc mid missing for one crop: water score is neutral.", "Falta el Kc medio de un cultivo: el puntaje de agua es neutro."));
            3
        }
    }
}

fn pest_pressure(s: &FieldSignals) -> bool {
    s.pest_generation_delta.is_some_and(|g| g > 0.0) || s.heat_stress_days.is_some_and(|d| d >= 10.0)
}

fn income_gap() -> L10n {
    L10n::new("No local price or yield data: income is a relative estimate; check prices in your market.",
              "Sin datos locales de precio o rendimiento: el ingreso es una estimación relativa; consulte precios en su mercado.")
}

fn current_strategy(c: &CropRecord, previous: &[CropRecord], s: &FieldSignals, renewal: bool) -> Strategy {
    let repeated = previous.iter().any(|p| p.family == c.family);
    let mut gaps = vec![income_gap()];
    let water = water_score(c, Some(c), s, &mut gaps);
    let mut risk = 3 - i32::from(repeated) - i32::from(pest_pressure(s));
    if renewal { risk -= 1; }
    let mut evidence: Vec<Evidence> = [trait_evidence(c, "Kc mid", "Kc medio", &c.traits.kc.mid),
        trait_evidence(c, "seasonal water need", "necesidad de agua por campaña", &c.traits.seasonal_water_need)]
        .into_iter().flatten().collect();
    evidence.extend(rotation_note(c));
    evidence.extend(signals_evidence(s));
    evidence.push(guidance("Repeating a crop family keeps its pests and soil-borne diseases supplied with a host.",
        "Repetir la misma familia mantiene un hospedero para sus plagas y enfermedades del suelo.", refs::sare_rotation()));
    let (title, why) = if renewal {
        (L10n::new(format!("Replant {} right away", c.name_en()), format!("Volver a plantar {} de inmediato", c.name_es())),
         L10n::new("Baseline: the plan you have now. Keeps the known crop and market, but the soil gets no break.",
                   "Línea base: su plan actual. Mantiene el cultivo y el mercado conocidos, pero el suelo no descansa."))
    } else {
        (L10n::new(format!("Keep {} next season (current plan)", c.name_en()), format!("Repetir {} la próxima campaña (plan actual)", c.name_es())),
         L10n::new("Baseline for comparison: the crop you grow now, again.", "Línea base para comparar: el mismo cultivo otra vez."))
    };
    let mut risks = vec![L10n::new(format!("Same family ({}) again: pests and root diseases carry over.", c.family),
                                   format!("Misma familia ({}) otra vez: plagas y enfermedades de raíz se mantienen.", c.family))];
    if pest_pressure(s) {
        risks.push(L10n::new("Warmer seasons speed up pest reproduction; more sprays may be needed.",
                             "Con más calor las plagas se reproducen más rápido; podrían necesitarse más aplicaciones."));
    }
    Strategy {
        id: if renewal { "replant-now".into() } else { "current".into() },
        is_current: true,
        title,
        sequence: seq(previous, vec![c.name(), c.name()]),
        scores: Scores { soil: clamp(2 + i32::from(c.is_legume()) - i32::from(repeated)), water, pest_break: 1, income: 3, risk: clamp(risk) },
        why,
        evidence,
        gaps,
        risks,
    }
}

fn different_family_strategy(c: &CropRecord, previous: &[CropRecord], s: &FieldSignals, renewal: bool) -> Strategy {
    let n = next_crop(c, previous);
    let in_prev = previous.iter().any(|p| p.family == n.family);
    let mut gaps = vec![income_gap()];
    let water = water_score(c, Some(n), s, &mut gaps);
    let legume_link = n.is_legume() || c.is_legume();
    let mut evidence: Vec<Evidence> = [
        trait_evidence(c, "Kc mid", "Kc medio", &c.traits.kc.mid),
        trait_evidence(n, "Kc mid", "Kc medio", &n.traits.kc.mid),
        trait_evidence(n, "seasonal water need", "necesidad de agua por campaña", &n.traits.seasonal_water_need),
    ].into_iter().flatten().collect();
    if n.is_legume() { evidence.extend(trait_evidence(n, "N fixation", "fijación de N", &n.traits.n_fixation.typical_range)); }
    if c.is_legume() { evidence.extend(trait_evidence(c, "N fixation", "fijación de N", &c.traits.n_fixation.typical_range)); }
    evidence.extend(rotation_note(n));
    evidence.extend(signals_evidence(s));
    evidence.push(guidance("Rotating to a crop of another family breaks pest and disease cycles; legumes leave N for the next crop.",
        "Rotar a otra familia corta el ciclo de plagas y enfermedades; las leguminosas dejan N al cultivo siguiente.", refs::sare_rotation()));
    let why = if n.is_legume() {
        L10n::new(format!("Adds a legume ({}): a different family that fixes nitrogen and breaks the {} pest cycle.", n.family, c.family),
                  format!("Suma una leguminosa ({}): otra familia que fija nitrógeno y corta el ciclo de plagas de {}.", n.family, c.family))
    } else if c.is_legume() {
        L10n::new(format!("Uses the nitrogen left by the legume with a {} crop, a different family.", n.family),
                  format!("Aprovecha el nitrógeno que deja la leguminosa con un cultivo de {}, otra familia.", n.family))
    } else {
        L10n::new(format!("Changes family ({} → {}) to break pest and disease cycles.", c.family, n.family),
                  format!("Cambia de familia ({} → {}) para cortar el ciclo de plagas y enfermedades.", c.family, n.family))
    };
    let (title, tail) = if renewal {
        (L10n::new(format!("One season of {} before replanting {}", n.name_en(), c.name_en()),
                   format!("Una campaña de {} antes de volver a plantar {}", n.name_es(), c.name_es())),
         vec![c.name(), n.name(), c.name()])
    } else {
        (L10n::new(format!("Follow with {} ({})", n.name_en(), n.family), format!("Seguir con {} ({})", n.name_es(), n.family)),
         vec![c.name(), n.name()])
    };
    if n.origin == Origin::Seed && n.traits.pests.is_empty() {
        gaps.push(L10n::new(format!("{}: no pest data.", n.name_en()), format!("{}: sin datos de plagas.", n.name_es())));
    }
    gaps.push(L10n::new(format!("Check local seed, water turn and market for {} before choosing.", n.name_en()),
                        format!("Verifique semilla, turno de riego y mercado local para {} antes de elegir.", n.name_es())));
    Strategy {
        id: if renewal { "break-crop".into() } else { "different-family".into() },
        is_current: false,
        title,
        sequence: seq(previous, tail),
        scores: Scores {
            soil: clamp(3 + i32::from(legume_link)),
            water,
            pest_break: clamp(if in_prev { 3 } else { 4 } + i32::from(pest_pressure(s) && !in_prev)),
            income: if renewal { 2 } else { 3 },
            risk: 3,
        },
        why,
        evidence,
        gaps,
        risks: vec![L10n::new(format!("A new crop needs know-how, seed and buyers for {}.", n.name_en()),
                              format!("Un cultivo nuevo requiere conocimiento, semilla y compradores para {}.", n.name_es()))],
    }
}

fn green_manure_strategy(c: &CropRecord, previous: &[CropRecord], s: &FieldSignals, renewal: bool) -> Strategy {
    let gm = L10n::new("Green manure (legume + cereal mix, incorporated at flowering)", "Abono verde (mezcla leguminosa + cereal, incorporada en floración)");
    let deficit = s.water_balance_mm.is_some_and(|w| w < 0.0);
    let mut evidence = vec![
        guidance("Cover crops incorporated before maturity (green manures) add organic matter, feed soil life, and legumes in the mix supply nitrogen to the next crop.",
                 "Los cultivos de cobertura incorporados antes de madurar (abonos verdes) aportan materia orgánica, alimentan la vida del suelo y la leguminosa de la mezcla aporta nitrógeno al cultivo siguiente.",
                 refs::sare_cover_crops()),
        guidance("Compost or humus enriched with Trichoderma protects roots against soil-borne diseases.",
                 "El compost o humus enriquecido con Trichoderma protege las raíces contra enfermedades del suelo.",
                 refs::trichoderma_review()),
    ];
    evidence.extend(signals_evidence(s));
    if s.bare_soil == Some(true) {
        evidence.push(signal_field("Satellite NDVI shows bare-soil periods on this field".into(), "El NDVI satelital muestra periodos de suelo desnudo en este campo".into()));
    }
    let mut gaps = vec![income_gap(), L10n::new("No soil analysis in the evidence package: organic matter gain cannot be quantified.",
                                                  "No hay análisis de suelo en el paquete de evidencias: no se puede cuantificar la ganancia de materia orgánica.")];
    if s.water_balance_mm.is_none() {
        gaps.push(L10n::new("Water balance unknown: check that the green manure can be irrigated.", "Balance hídrico desconocido: verifique que el abono verde pueda regarse."));
    }
    let next = if renewal { c.name() } else { next_crop(c, previous).name() };
    Strategy {
        id: "green-manure".into(),
        is_current: false,
        title: if renewal {
            L10n::new(format!("Green-manure fallow, then replant {}", c.name_en()), format!("Descanso con abono verde y luego replantar {}", c.name_es()))
        } else {
            L10n::new(format!("Green-manure break, then {}", next.en), format!("Pausa con abono verde y luego {}", next.es))
        },
        sequence: seq(previous, vec![c.name(), gm, next]),
        scores: Scores { soil: 5, water: if deficit { 2 } else { 3 }, pest_break: 5, income: 1, risk: 4 },
        why: L10n::new("Rebuilds the soil and fully breaks the pest host between cash crops, at the cost of one season without harvest.",
                       "Recupera el suelo y corta por completo el hospedero de plagas entre cultivos comerciales, a costa de una campaña sin cosecha."),
        evidence,
        gaps,
        risks: vec![L10n::new("One season without sales; needs irrigation water for the cover.", "Una campaña sin ventas; necesita agua de riego para la cobertura.")],
    }
}

/// Annual crop: current plan + a different-family crop + a green-manure break.
pub fn annual(c: &CropRecord, previous: &[CropRecord], s: &FieldSignals) -> Vec<Strategy> {
    vec![current_strategy(c, previous, s, false), different_family_strategy(c, previous, s, false), green_manure_strategy(c, previous, s, false)]
}

/// Semi-perennial crop at renewal: replant now, a break crop of another family, or a green-manure fallow.
pub fn renewal(c: &CropRecord, previous: &[CropRecord], s: &FieldSignals) -> Vec<Strategy> {
    vec![current_strategy(c, previous, s, true), different_family_strategy(c, previous, s, true), green_manure_strategy(c, previous, s, true)]
}

/// Cover between the last harvest of a semi-perennial and its replanting.
pub fn interim_cover(c: &CropRecord) -> Block {
    Block {
        title: L10n::new("Interim cover until replanting", "Cobertura temporal hasta replantar"),
        items: vec![
            Item {
                text: L10n::new("Sow a legume–cereal cover right after the last harvest so the soil is never bare; incorporate it at flowering.",
                                "Siembre una cobertura de leguminosa y cereal justo después de la última cosecha para que el suelo nunca quede desnudo; incorpórela en floración."),
                citations: vec![refs::sare_cover_crops()],
                basis: Some(refs::INTERVIEW.into()),
            },
            Item {
                text: L10n::new(format!("Chop and incorporate or compost the old {} plants promptly; do not pile them at the field edge, where they become a source of pests and diseases.", c.name_en()),
                                format!("Pique e incorpore o composte pronto las plantas viejas de {}; no las amontone en el borde del campo, donde se vuelven foco de plagas y enfermedades.", c.name_es())),
                citations: vec![refs::residue_incorporation()],
                basis: Some(refs::INTERVIEW.into()),
            },
        ],
    }
}

/// "What if I grow …?": the farmer's own proposal, scored with the same engine and the same field signals.
/// A perennial proposal is a long-term plantation (never "rotate the main crop"); an annual or semi-perennial one
/// is the next season after the current crop.
pub fn proposal(c: &CropRecord, previous: &[CropRecord], p: &CropRecord, s: &FieldSignals) -> Strategy {
    let same_family = p.family == c.family;
    let in_prev = previous.iter().any(|x| x.family == p.family);
    let mut gaps = vec![income_gap()];
    let water = water_score(c, Some(p), s, &mut gaps);
    let mut evidence: Vec<Evidence> = [
        trait_evidence(c, "Kc mid", "Kc medio", &c.traits.kc.mid),
        trait_evidence(p, "Kc mid", "Kc medio", &p.traits.kc.mid),
        trait_evidence(p, "seasonal water need", "necesidad de agua por campaña", &p.traits.seasonal_water_need),
    ].into_iter().flatten().collect();
    if p.is_legume() { evidence.extend(trait_evidence(p, "N fixation", "fijación de N", &p.traits.n_fixation.typical_range)); }
    evidence.extend(rotation_note(p));
    evidence.extend(signals_evidence(s));
    if p.origin == Origin::AiResearched {
        gaps.push(L10n::new(format!("{}: crop data researched by AI from open sources, verify.", p.name_en()),
                            format!("{}: datos del cultivo investigados por IA en fuentes abiertas, verificar.", p.name_es())));
    }
    if p.crop_type == CropType::Perennial {
        evidence.push(guidance("A perennial plantation is kept for years: it is not rotated; inter-row covers, mulch and residue management protect its soil.",
            "Una plantación perenne se mantiene por años: no se rota; las coberturas entre hileras, el mulch y el manejo de residuos protegen su suelo.", refs::sare_cover_crops()));
        // Reasoning, not a sourced figure: no citation is attached.
        evidence.push(Evidence { kind: EvidenceKind::Guidance, citation: None,
            text: L10n::new("A perennial is irrigated all year round, not only during one crop season, so its water score is not taken from Kc alone.",
                            "Un perenne se riega todo el año, no solo durante una campaña, así que su puntaje de agua no sale solo del Kc.") });
        gaps.push(L10n::new(format!("Years to the first {} harvest, plant cost and local market are not in the data.", p.name_en()),
                            format!("Los años hasta la primera cosecha de {}, el costo de plantación y el mercado local no están en los datos.", p.name_es())));
        return Strategy {
            id: "proposal".into(), is_current: false,
            title: L10n::new(format!("Your proposal: plant {} (perennial)", p.name_en()), format!("Tu propuesta: plantar {} (perenne)", p.name_es())),
            sequence: seq(previous, vec![c.name(), L10n::new(format!("{} (plantation, years)", p.name_en()), format!("{} (plantación, años)", p.name_es()))]),
            scores: Scores {
                soil: clamp(3 + i32::from(s.bare_soil == Some(true))),
                // Kc alone flatters a perennial: it is irrigated all year, not for one season.
                water: clamp(2 - i32::from(s.water_balance_mm.is_some_and(|w| w < 0.0))),
                // A different family ends the current crop's pests, but a perennial hosts its own pests for years.
                pest_break: clamp(if same_family { 1 } else { 3 }),
                income: 2,
                risk: clamp(2 - i32::from(s.water_balance_mm.is_some_and(|w| w < -50.0) && water <= 2)),
            },
            why: L10n::new(format!("{} is a perennial: planted once and kept for years, with covers between the rows instead of rotation. It changes the farm for a long time.", p.name_en()),
                           format!("{} es perenne: se planta una vez y se mantiene por años, con coberturas entre hileras en vez de rotación. Cambia el campo por mucho tiempo.", p.name_es())),
            evidence, gaps,
            risks: vec![L10n::new("Years without a harvest while the plants grow; a long-term commitment of land and water.",
                                  "Años sin cosecha mientras crecen las plantas; un compromiso largo de tierra y agua.")],
        };
    }
    let legume_link = p.is_legume() || c.is_legume();
    let why = if same_family {
        L10n::new(format!("{} is in the same family as {} ({}): it shares their pests and soil diseases, so it does not break the cycle.", p.name_en(), c.name_en(), c.family),
                  format!("{} es de la misma familia que {} ({}): comparte sus plagas y enfermedades del suelo, así que no corta el ciclo.", p.name_es(), c.name_es(), c.family))
    } else if p.is_legume() {
        L10n::new(format!("A legume ({}) after {}: another family that fixes nitrogen and breaks the pest cycle.", p.family, c.name_en()),
                  format!("Una leguminosa ({}) después de {}: otra familia que fija nitrógeno y corta el ciclo de plagas.", p.family, c.name_es()))
    } else {
        L10n::new(format!("Changes family ({} → {}) to break pest and disease cycles{}.", c.family, p.family, if c.is_legume() { ", using the nitrogen the legume left" } else { "" }),
                  format!("Cambia de familia ({} → {}) para cortar el ciclo de plagas y enfermedades{}.", c.family, p.family, if c.is_legume() { ", aprovechando el nitrógeno que dejó la leguminosa" } else { "" }))
    };
    let mut risks = vec![L10n::new(format!("Check seed, water turn and buyers for {} before choosing.", p.name_en()),
                                   format!("Verifique semilla, turno de riego y compradores para {} antes de elegir.", p.name_es()))];
    if same_family {
        risks.push(L10n::new(format!("Same family ({}) again: pests and root diseases carry over.", p.family), format!("Misma familia ({}) otra vez: plagas y enfermedades de raíz se mantienen.", p.family)));
    }
    Strategy {
        id: "proposal".into(), is_current: false,
        title: L10n::new(format!("Your proposal: {} next", p.name_en()), format!("Tu propuesta: {} después", p.name_es())),
        sequence: seq(previous, vec![c.name(), p.name()]),
        scores: Scores {
            soil: clamp(3 + i32::from(legume_link) - i32::from(same_family)),
            water,
            pest_break: clamp(if same_family { 1 } else if in_prev { 3 } else { 4 } + i32::from(pest_pressure(s) && !same_family && !in_prev)),
            income: 3,
            risk: clamp(3 - i32::from(same_family) - i32::from(pest_pressure(s) && same_family)),
        },
        why, evidence, gaps, risks,
    }
}
