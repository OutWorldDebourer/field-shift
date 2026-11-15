//! Phase-D research for a known crop (iteration 4): the weather that favours each of its pests, the critical
//! micronutrients with deficiency symptoms, and biostimulant categories with published evidence for the crop.
//! Same rules as `research.rs`: open sources found with web search, every value with a citation whose URL was
//! actually retrieved, otherwise a gap; no product names or doses. The result is merged into the seed JSON by the
//! `crop_extras` tool and labelled "AI-researched, verify" until a person reviews it.
use crate::model::*;
use crate::research::{extract_json, norm_url, search_conversation};
use serde_json::Value;
use std::collections::HashSet;

const SYSTEM: &str = "You are an agronomy research assistant for Field Shift, a pre-diagnosis tool for smallholder irrigated farms on the coast of Peru (Huaral valley). You research open, citable sources (university extension, FAO, peer-reviewed papers, INIA, SENASA, CIP). Hard rules: every item must carry a citation whose url is a page you found with web_search in this conversation; never invent; if you cannot cite something, leave it out and name it in \"gaps\". Never give commercial product names or application doses. Your final message must be only the JSON object.";

fn prompt(c: &CropRecord) -> String {
    let pests: Vec<String> = c.traits.pests.iter().map(|p| format!("{} ({})", p.name.en, p.scientific_name)).collect();
    format!(r#"Crop: {} ({}, family {}). Its main pests in our knowledge base: {}.

Reply with one JSON object exactly in this shape (CIT = {{"title": "...", "author": "author(s) or organization", "year": 2020 or null, "url": "https://..."}}):
{{
  "pest_conditions": [{{"scientific_name": "as listed above", "favourable": {{"en": "plain words, e.g. hot, dry weather", "es": "..."}}, "temp_min_c": number or null, "temp_max_c": number or null, "humidity": "dry" | "humid" | "any", "season": {{"en": "...", "es": "..."}} or null, "citation": CIT}}],
  "micronutrients": [{{"element": "Zn|B|Fe|Mn|Mo|Cu", "why": {{"en": "its role for this crop, one sentence", "es": "..."}}, "deficiency": {{"en": "visible symptoms", "es": "..."}}, "citation": CIT}}],
  "biostimulants": [{{"category": "amino-acids|seaweed-extract|humic-fulvic|protein-hydrolysate|microbial|other", "name": {{"en": "...", "es": "..."}}, "evidence": {{"en": "what a study found on this crop or a close relative, one sentence", "es": "..."}}, "stress": {{"en": "heat / salinity / drought…", "es": "..."}} or null, "citation": CIT}}],
  "gaps": ["topics you could not cite"]
}}
Guidance: one entry per listed pest where a source describes the weather that favours it (temperature range in °C only if the source gives numbers); the 2-4 micronutrients most often deficient or critical for this crop; 1-3 biostimulant categories with evidence on this crop (field or greenhouse trials). Spanish in "tú" form. Use at most 8 searches."#,
        c.name_en(), c.scientific_name, c.family, if pests.is_empty() { "none recorded".to_string() } else { pests.join("; ") })
}

fn l10n(v: &Value) -> Option<L10n> {
    let en = v["en"].as_str()?.trim(); let es = v["es"].as_str().unwrap_or(en).trim();
    (!en.is_empty()).then(|| L10n::new(en, es))
}

fn cite(v: &Value, seen: &HashSet<String>) -> Option<Citation> {
    let url = v["url"].as_str()?.trim();
    if !url.starts_with("http") || !seen.contains(&norm_url(url)) { return None; }
    Some(Citation { title: v["title"].as_str().unwrap_or("").trim().to_string(), author: v["author"].as_str().unwrap_or("").trim().to_string(),
                    year: v["year"].as_i64().map(|y| y as i32), url: url.to_string() })
}

/// What the research produced, validated: uncited items are dropped and named as gaps. Stored as the sidecar
/// `data/extras/<key>.json` (the curated seed file stays untouched) and applied when the seed loads.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Extras {
    /// (pest scientific name, conditions)
    pub conditions: Vec<(String, PestConditions)>,
    pub micronutrients: Vec<Micronutrient>,
    pub biostimulants: Vec<Biostimulant>,
    pub gaps: Vec<String>,
    /// False until a person reviews it ("AI-researched, verify").
    #[serde(default)]
    pub reviewed: bool,
}

pub fn validate(v: &Value, seen: &HashSet<String>) -> Extras {
    let mut x = Extras::default();
    for p in v["pest_conditions"].as_array().into_iter().flatten() {
        let (Some(sci), Some(fav)) = (p["scientific_name"].as_str(), l10n(&p["favourable"])) else { continue };
        match cite(&p["citation"], seen) {
            Some(c) => x.conditions.push((sci.trim().to_string(), PestConditions {
                favourable: fav, temp_min_c: p["temp_min_c"].as_f64(), temp_max_c: p["temp_max_c"].as_f64(),
                humidity: match p["humidity"].as_str() { Some(h @ ("dry" | "humid")) => h.into(), _ => "any".into() },
                season: l10n(&p["season"]), citation: c })),
            None => x.gaps.push(format!("conditions for {sci}: citation not verified")),
        }
    }
    for m in v["micronutrients"].as_array().into_iter().flatten() {
        let (Some(el), Some(why), Some(def)) = (m["element"].as_str(), l10n(&m["why"]), l10n(&m["deficiency"])) else { continue };
        match cite(&m["citation"], seen) {
            Some(c) => x.micronutrients.push(Micronutrient { element: el.trim().to_string(), why, deficiency: def, citation: c }),
            None => x.gaps.push(format!("micronutrient {el}: citation not verified")),
        }
    }
    for b in v["biostimulants"].as_array().into_iter().flatten() {
        let (Some(cat), Some(name), Some(ev)) = (b["category"].as_str(), l10n(&b["name"]), l10n(&b["evidence"])) else { continue };
        match cite(&b["citation"], seen) {
            Some(c) => x.biostimulants.push(Biostimulant { category: cat.trim().to_string(), name, evidence: ev, stress: l10n(&b["stress"]), citation: c }),
            None => x.gaps.push(format!("biostimulant {cat}: citation not verified")),
        }
    }
    x.gaps.extend(v["gaps"].as_array().into_iter().flatten().filter_map(|g| g.as_str().map(|s| s.trim().to_string())).filter(|s| !s.is_empty()));
    x
}

/// Researches the phase-D data for `c` (paid: one web-search conversation; usage in `api_usage` when `db` is given).
pub async fn research(http: &reqwest::Client, api_key: &str, c: &CropRecord, db: Option<&sqlx::MySqlPool>) -> anyhow::Result<Extras> {
    let (text, seen) = search_conversation(http, api_key, SYSTEM, prompt(c), &format!("extras:{}", c.key), 12_000, db).await?;
    let v = extract_json(&text).ok_or_else(|| anyhow::anyhow!("no JSON in the research reply"))?;
    Ok(validate(&v, &seen))
}

/// Applies a sidecar to a record (pests matched by scientific name).
pub fn apply(rec: &mut CropRecord, x: &Extras) {
    for (sci, cond) in &x.conditions {
        let key = sci.to_lowercase();
        if let Some(p) = rec.traits.pests.iter_mut().find(|p| {
            let s = p.scientific_name.to_lowercase();
            !s.is_empty() && (s.contains(&key) || key.contains(&s))
        }) { p.conditions = Some(cond.clone()); }
    }
    rec.traits.micronutrients = x.micronutrients.clone();
    rec.traits.biostimulants = x.biostimulants.clone();
    rec.traits.extras_reviewed = Some(x.reviewed);
    rec.traits.extras_gaps = x.gaps.clone();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn uncited_items_become_gaps() {
        let seen: HashSet<String> = [norm_url("https://ok.org/a")].into_iter().collect();
        let v = json!({
            "pest_conditions": [{ "scientific_name": "Tetranychus urticae", "favourable": { "en": "hot, dry weather", "es": "clima cálido y seco" }, "temp_min_c": 25, "temp_max_c": 35, "humidity": "dry",
                                  "citation": { "title": "t", "author": "a", "year": 2020, "url": "https://ok.org/a" } }],
            "micronutrients": [{ "element": "Zn", "why": { "en": "x", "es": "y" }, "deficiency": { "en": "x", "es": "y" }, "citation": { "title": "t", "url": "https://invented.org" } }],
            "biostimulants": [], "gaps": ["seaweed"],
        });
        let x = validate(&v, &seen);
        assert_eq!(x.conditions.len(), 1);
        assert_eq!(x.conditions[0].1.humidity, "dry");
        assert!(x.micronutrients.is_empty());
        assert!(x.gaps.iter().any(|g| g.contains("Zn")) && x.gaps.iter().any(|g| g == "seaweed"));
    }
}
