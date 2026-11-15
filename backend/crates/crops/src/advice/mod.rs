//! Advice engine: crop type drives the plan (annual → 3 rotation strategies; semi-perennial → renewal options and
//! interim cover; perennial → inter-row practices, never rotating the main crop), plus soil, pest care and nutrition
//! blocks. Every text is in English and Spanish; every figure cites its trait source or names the field signal.
mod care;
mod perennial;
pub mod refs;
mod rotation;

use crate::model::{Citation, CropRecord, CropType, L10n, Origin, Trait};
use serde::{Deserialize, Serialize};

/// Field indicators from the evidence package (all optional; a missing one becomes a declared gap).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FieldSignals {
    /// Days above the heat-stress threshold in the analysed season (NASA POWER).
    pub heat_stress_days: Option<f64>,
    /// Season water balance, mm (rain + irrigation − crop ET); negative = deficit.
    pub water_balance_mm: Option<f64>,
    /// Estimated extra pest generations per season versus the 40-year baseline (computed by the caller).
    pub pest_generation_delta: Option<f64>,
    /// The pest `pest_generation_delta` refers to: the current crop's leading pest (its own published threshold).
    #[serde(default)]
    pub pest_name: Option<L10n>,
    /// True when NDVI shows bare-soil periods on the field (Sentinel-2/Landsat).
    pub bare_soil: Option<bool>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AdviceMode {
    /// Annual crop: three rotation strategies, the current plan included.
    Rotation,
    /// Semi-perennial: what to plant at renewal, plus interim cover.
    Renewal,
    /// Perennial: the main crop stays; inter-row covers, intercropping, residues, biological control.
    PerennialSystem,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// A cited trait value from the crop record.
    Trait,
    /// A field indicator from the evidence package.
    Signal,
    /// A cited guidance source.
    Guidance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub text: L10n,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<Citation>,
}

/// 1–5, higher is better for the farmer (risk: 5 = lowest risk).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scores {
    pub soil: u8,
    pub water: u8,
    pub pest_break: u8,
    pub income: u8,
    pub risk: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub is_current: bool,
    pub title: L10n,
    /// Season by season, the last item being the next season.
    pub sequence: Vec<L10n>,
    pub scores: Scores,
    pub why: L10n,
    pub evidence: Vec<Evidence>,
    pub gaps: Vec<L10n>,
    pub risks: Vec<L10n>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Practice {
    pub id: String,
    pub title: L10n,
    pub why: L10n,
    pub actions: Vec<L10n>,
    pub evidence: Vec<Evidence>,
    pub gaps: Vec<L10n>,
    pub risks: Vec<L10n>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub text: L10n,
    pub citations: Vec<Citation>,
    /// Non-published basis (e.g. the agronomist interview), when the theme comes from it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub title: L10n,
    pub items: Vec<Item>,
}

/// A pest with both a published lower threshold and degree-days, so the caller can compute generations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PestThreshold {
    pub name: L10n,
    pub scientific_name: String,
    pub lower_threshold_c: f64,
    pub degree_days: f64,
    pub threshold_citation: Citation,
    pub degree_days_citation: Citation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropSummary {
    pub key: String,
    pub name: L10n,
    pub scientific_name: String,
    pub family: String,
    pub crop_type: CropType,
    pub origin: Origin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Advice {
    pub crop: CropSummary,
    pub previous: Vec<CropSummary>,
    pub mode: AdviceMode,
    /// Pre-diagnosis notice, always shown.
    pub notice: L10n,
    /// Annual: current + 2 alternatives. Semi-perennial: renewal options. Perennial: always empty.
    pub strategies: Vec<Strategy>,
    /// Semi-perennial only: cover between the last harvest and replanting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interim_cover: Option<Block>,
    /// Perennial only: practices that keep the main crop.
    pub practices: Vec<Practice>,
    pub soil: Block,
    pub pest_care: Block,
    pub nutrition: Block,
    /// Pests whose generations the caller may compute (published threshold and degree-days both present).
    pub pest_thresholds: Vec<PestThreshold>,
    pub gaps: Vec<L10n>,
}

pub fn summary(c: &CropRecord) -> CropSummary {
    CropSummary {
        key: c.key.clone(),
        name: c.name(),
        scientific_name: c.scientific_name.clone(),
        family: c.family.clone(),
        crop_type: c.crop_type,
        origin: c.origin,
        label: c.label.clone(),
    }
}

/// Builds the advice for the current crop given the previous crops (oldest first) and field signals.
pub fn advice(current: &CropRecord, previous: &[CropRecord], signals: &FieldSignals) -> Advice {
    let mode = match current.crop_type {
        CropType::Annual => AdviceMode::Rotation,
        CropType::SemiPerennial => AdviceMode::Renewal,
        CropType::Perennial => AdviceMode::PerennialSystem,
    };
    let (strategies, interim_cover, practices) = match mode {
        AdviceMode::Rotation => (rotation::annual(current, previous, signals), None, vec![]),
        AdviceMode::Renewal => (rotation::renewal(current, previous, signals), Some(rotation::interim_cover(current)), vec![]),
        AdviceMode::PerennialSystem => (vec![], None, perennial::practices(current, signals)),
    };
    let mut gaps = vec![];
    if current.origin == Origin::AiResearched {
        gaps.push(L10n::new("Crop data were researched by AI from open sources: verify with an agronomist.",
                            "Los datos del cultivo fueron investigados por IA en fuentes abiertas: verifíquelos con un agrónomo."));
    }
    for (path, t) in current.trait_leaves() {
        if let Trait::Gap { gap } = t {
            if !path.starts_with("pests") {
                gaps.push(L10n::new(format!("{}: {path} not available ({gap}).", current.name_en()),
                                    format!("{}: {path} no disponible ({gap}).", current.name_es())));
            }
        }
    }
    for (name, missing) in [
        ("heat-stress days / días de estrés por calor", signals.heat_stress_days.is_none()),
        ("water balance / balance hídrico", signals.water_balance_mm.is_none()),
        ("pest generations (no published degree-day threshold for this crop's pests) / generaciones de plagas (sin umbral publicado de grados-día para las plagas de este cultivo)", signals.pest_generation_delta.is_none()),
        ("bare-soil periods / periodos de suelo desnudo", signals.bare_soil.is_none()),
    ] {
        if missing {
            gaps.push(L10n::new(format!("NASA signal missing: {}.", name.split(" / ").next().unwrap_or(name)),
                                format!("Falta la señal NASA: {}.", name.split(" / ").nth(1).unwrap_or(name))));
        }
    }
    Advice {
        crop: summary(current),
        previous: previous.iter().map(summary).collect(),
        mode,
        notice: L10n::new(
            "Pre-diagnosis, not a diagnosis: review these options with your agronomist or extension engineer before acting.",
            "Prediagnóstico, no diagnóstico: revise estas opciones con su agrónomo o ingeniero de extensión antes de actuar."),
        strategies,
        interim_cover,
        practices,
        soil: care::soil(current, signals),
        pest_care: care::pest_care(current, signals),
        nutrition: care::nutrition(current, previous),
        pest_thresholds: pest_thresholds(current),
        gaps,
    }
}

/// The farmer's own proposed crop as a fourth strategy, scored with the same engine ("What if I grow …?").
pub fn evaluate(current: &CropRecord, previous: &[CropRecord], candidate: &CropRecord, signals: &FieldSignals) -> Strategy {
    rotation::proposal(current, previous, candidate, signals)
}

pub fn pest_thresholds(c: &CropRecord) -> Vec<PestThreshold> {
    c.traits.pests.iter().filter_map(|p| {
        let (Trait::Value { citation: tc, .. }, Trait::Value { citation: dc, note, .. }) = (&p.lower_threshold, &p.degree_days_per_generation) else { return None };
        Some(PestThreshold {
            name: p.name.clone(),
            scientific_name: p.scientific_name.clone(),
            lower_threshold_c: p.lower_threshold.num()?,
            degree_days: p.degree_days_per_generation.num()?,
            threshold_citation: tc.clone(),
            degree_days_citation: dc.clone(),
            note: note.clone(),
        })
    }).collect()
}

/// Evidence line from a cited trait ("Pea Kc mid: 1.15 (FAO-56)"); None for a gap.
pub(crate) fn trait_evidence(crop: &CropRecord, label_en: &str, label_es: &str, t: &Trait) -> Option<Evidence> {
    let v = t.display()?;
    Some(Evidence {
        kind: EvidenceKind::Trait,
        text: L10n::new(format!("{} {label_en}: {v}", crop.name_en()), format!("{} {label_es}: {v}", crop.name_es())),
        citation: t.citation().cloned(),
    })
}

/// A climate signal from NASA POWER: regional context (~50 km grid), never a field measurement.
pub(crate) fn signal(en: String, es: String) -> Evidence {
    Evidence { kind: EvidenceKind::Signal, text: L10n::new(en + " (NASA POWER, regional context ~50 km)", es + " (NASA POWER, contexto regional ~50 km)"), citation: None }
}

/// A satellite signal measured over the field outline (Sentinel-2 / Landsat, field-level).
pub(crate) fn signal_field(en: String, es: String) -> Evidence {
    Evidence { kind: EvidenceKind::Signal, text: L10n::new(en + " (Sentinel-2 / Landsat, field-level)", es + " (Sentinel-2 / Landsat, nivel de campo)"), citation: None }
}

pub(crate) fn guidance(en: &str, es: &str, c: Citation) -> Evidence {
    Evidence { kind: EvidenceKind::Guidance, text: L10n::new(en, es), citation: Some(c) }
}

pub(crate) fn clamp(x: i32) -> u8 { x.clamp(1, 5) as u8 }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn s() -> FieldSignals {
        FieldSignals { heat_stress_days: Some(14.0), water_balance_mm: Some(-120.0), pest_generation_delta: Some(1.2), pest_name: Some(L10n::new("Pea aphid", "Pulgón de la arveja")), bare_soil: Some(true) }
    }

    fn all_text(a: &Advice) -> Vec<&L10n> {
        let mut v: Vec<&L10n> = vec![&a.notice];
        for st in &a.strategies {
            v.extend([&st.title, &st.why]);
            v.extend(st.sequence.iter().chain(&st.gaps).chain(&st.risks));
            v.extend(st.evidence.iter().map(|e| &e.text));
        }
        for p in &a.practices {
            v.extend([&p.title, &p.why]);
            v.extend(p.actions.iter().chain(&p.gaps).chain(&p.risks));
            v.extend(p.evidence.iter().map(|e| &e.text));
        }
        for b in [&a.soil, &a.pest_care, &a.nutrition].into_iter().chain(a.interim_cover.as_ref()) {
            v.push(&b.title);
            v.extend(b.items.iter().map(|i| &i.text));
        }
        v.extend(&a.gaps);
        v
    }

    #[test]
    fn annual_gets_three_scored_strategies_current_first() {
        let a = advice(seed::get("pea").unwrap(), &[seed::get("maize").unwrap().clone()], &s());
        assert_eq!(a.mode, AdviceMode::Rotation);
        assert_eq!(a.strategies.len(), 3);
        assert!(a.strategies[0].is_current);
        assert!(a.strategies[1..].iter().all(|x| !x.is_current));
        for st in &a.strategies {
            for sc in [st.scores.soil, st.scores.water, st.scores.pest_break, st.scores.income, st.scores.risk] {
                assert!((1..=5).contains(&sc));
            }
            assert!(!st.evidence.is_empty() && !st.gaps.is_empty() && !st.risks.is_empty());
        }
        // A legume is followed by a different family.
        assert!(!a.strategies[1].sequence.last().unwrap().en.to_lowercase().contains("pea"));
        assert!(a.interim_cover.is_none() && a.practices.is_empty());
    }

    #[test]
    fn non_legume_annual_gets_a_legume_option() {
        let a = advice(seed::get("maize").unwrap(), &[], &FieldSignals::default());
        let next = &a.strategies[1].sequence.last().unwrap().en.to_lowercase();
        assert!(next.contains("pea") || next.contains("bean"), "{next}");
        assert!(a.gaps.iter().any(|g| g.en.contains("water balance")));
    }

    #[test]
    fn perennials_never_rotate_the_main_crop() {
        for k in ["grape", "mango", "avocado", "mandarin", "asparagus"] {
            let c = seed::get(k).unwrap();
            let a = advice(c, &[], &s());
            assert_eq!(a.mode, AdviceMode::PerennialSystem, "{k}");
            assert!(a.strategies.is_empty(), "{k}: perennial got rotation strategies");
            let ids: Vec<&str> = a.practices.iter().map(|p| p.id.as_str()).collect();
            for id in ["inter-row-cover", "intercropping", "residue-management", "biological-control"] {
                assert!(ids.contains(&id), "{k} missing {id}");
            }
            for t in all_text(&a) {
                let en = t.en.to_lowercase();
                assert!(!en.contains("rotate the main crop") || en.contains("never") || en.contains("not "), "{k}: {en}");
                assert!(!en.contains("replace the") , "{k}: {en}");
            }
        }
    }

    #[test]
    fn semi_perennials_get_renewal_and_interim_cover() {
        for k in ["pineapple", "strawberry", "alfalfa"] {
            let a = advice(seed::get(k).unwrap(), &[], &s());
            assert_eq!(a.mode, AdviceMode::Renewal);
            assert_eq!(a.strategies.len(), 3);
            assert!(a.interim_cover.is_some());
        }
    }

    #[test]
    fn every_text_is_bilingual_and_every_citation_has_a_url() {
        for c in seed::all() {
            let a = advice(c, &[seed::get("potato").unwrap().clone()], &s());
            for t in all_text(&a) {
                assert!(!t.en.trim().is_empty() && !t.es.trim().is_empty(), "{}: {:?}", c.key, t);
            }
            let mut cits: Vec<&Citation> = vec![];
            for st in &a.strategies { cits.extend(st.evidence.iter().filter_map(|e| e.citation.as_ref())); }
            for p in &a.practices { cits.extend(p.evidence.iter().filter_map(|e| e.citation.as_ref())); }
            for b in [&a.soil, &a.pest_care, &a.nutrition] { cits.extend(b.items.iter().flat_map(|i| &i.citations)); }
            for c2 in cits { assert!(c2.url.starts_with("http"), "{}", c2.title); }
        }
        for r in refs::all() { assert!(r.url.starts_with("https://")); }
    }

    #[test]
    fn pest_thresholds_only_where_published() {
        let pea = seed::get("pea").unwrap();
        for p in pest_thresholds(pea) {
            assert!(p.lower_threshold_c.is_finite() && p.degree_days > 0.0);
        }
        let n_full = pea.traits.pests.iter().filter(|p| p.lower_threshold.num().is_some() && p.degree_days_per_generation.num().is_some()).count();
        assert_eq!(pest_thresholds(pea).len(), n_full);
    }

    #[test]
    fn proposals_use_the_same_engine() {
        let pea = crate::seed::get("pea").unwrap();
        let lettuce = crate::seed::get("lettuce").unwrap().clone();
        // A perennial is a plantation, never "rotate the main crop".
        let avo = evaluate(pea, &[lettuce.clone()], crate::seed::get("avocado").unwrap(), &s());
        assert!(avo.title.en.contains("perennial") && avo.id == "proposal");
        assert!(!avo.why.en.to_lowercase().contains("rotate"));
        assert!(avo.scores.water <= 2, "a perennial is irrigated all year");
        // Same botanical family as the current crop: no pest-cycle break.
        let bean = evaluate(pea, &[lettuce.clone()], crate::seed::get("common_bean").unwrap(), &s());
        assert_eq!(bean.scores.pest_break, 1);
        // Another family breaks the cycle.
        let maize = evaluate(pea, &[lettuce], crate::seed::get("maize").unwrap(), &s());
        assert!(maize.scores.pest_break >= 4);
    }
}
