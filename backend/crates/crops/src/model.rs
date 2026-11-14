//! Crop record types: names, botany, crop type and traits where every value carries its citation or is an explicit gap.
use serde::{Deserialize, Serialize};

/// Where a value comes from. `url` is always an http(s) link for trait values.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Citation {
    pub title: String,
    /// Author(s) or organization.
    pub author: String,
    pub year: Option<i32>,
    pub url: String,
}

/// A trait value: a flag, a number, a range or a short text, always as the source gives it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Val {
    Bool(bool),
    Num(f64),
    Range { min: f64, max: f64 },
    Text(String),
}

/// `{ value, unit, citation, note? }` or `{ gap }`: never a value without a source.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Trait {
    Value {
        value: Val,
        unit: String,
        citation: Citation,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    Gap { gap: String },
}

impl Trait {
    pub fn gap(reason: impl Into<String>) -> Self { Trait::Gap { gap: reason.into() } }
    pub fn citation(&self) -> Option<&Citation> {
        match self { Trait::Value { citation, .. } => Some(citation), Trait::Gap { .. } => None }
    }
    /// A single number: the value itself, or the midpoint of a range.
    pub fn num(&self) -> Option<f64> {
        match self {
            Trait::Value { value: Val::Num(n), .. } => Some(*n),
            Trait::Value { value: Val::Range { min, max }, .. } => Some((min + max) / 2.0),
            _ => None,
        }
    }
    pub fn flag(&self) -> Option<bool> {
        match self { Trait::Value { value: Val::Bool(b), .. } => Some(*b), _ => None }
    }
    /// "1.15 -", "350–500 mm/season", or None for a gap.
    pub fn display(&self) -> Option<String> {
        let Trait::Value { value, unit, .. } = self else { return None };
        let v = match value {
            Val::Bool(b) => if *b { "yes".into() } else { "no".into() },
            Val::Num(n) => fmt_num(*n),
            Val::Range { min, max } => format!("{}–{}", fmt_num(*min), fmt_num(*max)),
            Val::Text(t) => t.clone(),
        };
        let unitless = unit.is_empty() || unit == "-" || matches!(value, Val::Bool(_)) || unit.eq_ignore_ascii_case("dimensionless");
        Some(if unitless { v } else { format!("{v} {unit}") })
    }
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 { format!("{n:.0}") } else { format!("{n}") }
}

/// Text in both product languages.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct L10n {
    pub en: String,
    pub es: String,
}

impl L10n {
    pub fn new(en: impl Into<String>, es: impl Into<String>) -> Self { Self { en: en.into(), es: es.into() } }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Names {
    #[serde(default)]
    pub en: Vec<String>,
    #[serde(default)]
    pub es: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CropType {
    Annual,
    SemiPerennial,
    Perennial,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    #[default]
    Seed,
    AiResearched,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Kc {
    pub initial: Trait,
    pub mid: Trait,
    pub end: Trait,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Temperature {
    pub base: Trait,
    pub optimum: Trait,
    pub max: Trait,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NFixation {
    pub legume: Trait,
    pub typical_range: Trait,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pest {
    pub name: L10n,
    #[serde(default)]
    pub scientific_name: String,
    /// Source saying it is a main pest of this crop.
    pub citation: Citation,
    /// Lower developmental threshold, °C (only where published).
    pub lower_threshold: Trait,
    /// Degree-days (°C·day) per generation or egg-to-adult (see note), only where published.
    pub degree_days_per_generation: Trait,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Beneficial {
    pub name: L10n,
    #[serde(default)]
    pub scientific_name: String,
    /// insect | mite | fungus | bacterium | nematode | virus
    pub kind: String,
    pub controls: L10n,
    pub citation: Citation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RotationNotes {
    Note { en: String, es: String, citation: Citation },
    Gap { gap: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Traits {
    pub kc: Kc,
    pub seasonal_water_need: Trait,
    pub temperature: Temperature,
    pub n_fixation: NFixation,
    #[serde(default)]
    pub pests: Vec<Pest>,
    #[serde(default)]
    pub beneficials: Vec<Beneficial>,
    pub rotation_notes: RotationNotes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CropRecord {
    pub key: String,
    pub names: Names,
    pub scientific_name: String,
    pub family: String,
    pub crop_type: CropType,
    pub traits: Traits,
    #[serde(default)]
    pub origin: Origin,
    /// "AI-researched, verify" on researched records; None for the verified seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Extra spellings farmers typed that resolved to this record.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl CropRecord {
    pub fn name_en(&self) -> &str { self.names.en.first().map(String::as_str).unwrap_or(&self.key) }
    pub fn name_es(&self) -> &str { self.names.es.first().map(String::as_str).unwrap_or(self.name_en()) }
    pub fn name(&self) -> L10n { L10n::new(self.name_en(), self.name_es()) }
    pub fn is_legume(&self) -> bool {
        self.traits.n_fixation.legume.flag().unwrap_or(false) || self.family.eq_ignore_ascii_case("Fabaceae")
    }
    /// Every name a farmer might type: key, common names, scientific name, aliases.
    pub fn all_names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.key.as_str())
            .chain(self.names.en.iter().map(String::as_str))
            .chain(self.names.es.iter().map(String::as_str))
            .chain(self.aliases.iter().map(String::as_str))
            .chain(std::iter::once(self.scientific_name.as_str()))
    }
    /// Every trait leaf with a dotted path, for validation and reporting.
    pub fn trait_leaves(&self) -> Vec<(String, &Trait)> {
        let t = &self.traits;
        let mut v = vec![
            ("kc.initial".into(), &t.kc.initial),
            ("kc.mid".into(), &t.kc.mid),
            ("kc.end".into(), &t.kc.end),
            ("seasonal_water_need".into(), &t.seasonal_water_need),
            ("temperature.base".into(), &t.temperature.base),
            ("temperature.optimum".into(), &t.temperature.optimum),
            ("temperature.max".into(), &t.temperature.max),
            ("n_fixation.legume".into(), &t.n_fixation.legume),
            ("n_fixation.typical_range".into(), &t.n_fixation.typical_range),
        ];
        for p in &t.pests {
            v.push((format!("pests[{}].lower_threshold", p.scientific_name), &p.lower_threshold));
            v.push((format!("pests[{}].degree_days_per_generation", p.scientific_name), &p.degree_days_per_generation));
        }
        v
    }
    /// Every citation in the record (trait values, pests, beneficials, rotation notes).
    pub fn citations(&self) -> Vec<&Citation> {
        let mut c: Vec<&Citation> = self.trait_leaves().into_iter().filter_map(|(_, t)| t.citation()).collect();
        c.extend(self.traits.pests.iter().map(|p| &p.citation));
        c.extend(self.traits.beneficials.iter().map(|b| &b.citation));
        if let RotationNotes::Note { citation, .. } = &self.traits.rotation_notes { c.push(citation); }
        c
    }
}
