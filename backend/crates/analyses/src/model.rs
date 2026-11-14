//! Wire types, mirroring `frontend/src/shared/contract.ts`. Section content and evidence stay as JSON values here
//! so this crate does not depend on the services that produce them.
use common::geo::Field;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SECTION_KEYS: [&str; 6] = ["field", "soil", "pests", "nutrition", "rotation", "sheet"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bi {
    pub en: String,
    pub es: String,
}

impl Bi {
    pub fn new(en: impl Into<String>, es: impl Into<String>) -> Self { Self { en: en.into(), es: es.into() } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldOut {
    pub center: common::geo::LatLon,
    pub ring: Vec<[f64; 2]>,
    pub area_ha: f64,
    pub label: String,
}

impl FieldOut {
    pub fn geom(&self) -> Field { Field { center: self.center, ring: self.ring.clone() } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhotoRef {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub role: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub photos: Vec<PhotoRef>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
    pub key: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Bi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisState {
    pub id: String,
    pub lang: String,
    pub location_mode: String,
    pub example_site: Option<String>,
    pub field: FieldOut,
    pub crop_current: Option<Value>,
    pub crop_history: Vec<Value>,
    pub messages: Vec<ChatMessage>,
    pub sections: Vec<Section>,
    pub evidence: Option<Value>,
    pub demo_mode: bool,
    /// Headline + confidence from the rotation section (built by the evidence stage), if any.
    pub prediagnosis: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewAnalysis {
    pub field: Field,
    #[serde(default = "default_mode")]
    pub location_mode: String,
    pub example_site: Option<String>,
    #[serde(default = "default_lang")]
    pub lang: String,
    pub label: Option<String>,
}

fn default_mode() -> String { "map".into() }
fn default_lang() -> String { "en".into() }
