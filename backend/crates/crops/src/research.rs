//! Research of an unknown crop with Claude (Messages API + server-side web search), validated so that
//! only values with a URL citation seen in the search results survive; everything else becomes a gap.
//! The API key is read from config and only ever placed in the `x-api-key` header (never logged).
use crate::model::*;
use crate::normalize::slug;
use anyhow::{anyhow, bail, Context};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;

pub const MODEL: &str = "claude-opus-5-5";
const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Output budget per research job (JSON record plus thinking); kept modest to bound cost.
const MAX_TOKENS: u32 = 12_000;
/// Web searches allowed per research job.
const MAX_SEARCHES: u32 = 8;
/// Continuations allowed if the server-side tool loop pauses (`pause_turn`).
const MAX_CONTINUATIONS: usize = 2;
pub const LABEL: &str = "AI-researched, verify";

const SYSTEM: &str = "You are an agronomy research assistant for Field Shift, a pre-diagnosis tool for smallholder irrigated farms on the coast of Peru (Huaral valley). You research one crop at a time from open, citable sources (FAO-56 and FAO Ecocrop, university extension, peer-reviewed papers, Peruvian sources such as INIA, SENASA and UNALM theses where available). Hard rules: every value must come with a citation whose url is a page you found with web_search in this conversation; never invent or estimate a value; if you cannot cite a value, return {\"gap\": \"short reason\"} for it. Quote values exactly as the source gives them (ranges stay ranges), in metric units (°C, mm, kg N/ha, °C·day). Your final message must be only the JSON object, with no prose around it.";

fn user_prompt(query: &str) -> String {
    format!(r#"A farmer typed this crop name: "{query}". Identify the crop (it may be Spanish, English or a local name). If it is not a cultivated plant, reply only {{"not_a_crop": true, "reason": "..."}}.

Otherwise reply with one JSON object in exactly this shape. Each TRAIT is either {{"value": <number | true/false | {{"min": a, "max": b}} | "short text">, "unit": "...", "citation": CIT, "note": "optional context"}} or {{"gap": "reason"}}. CIT is {{"title": "...", "author": "author(s) or organization", "year": 2020 or null, "url": "https://..."}}.

{{
  "names": {{"en": ["main English name", "other names"], "es": ["main Spanish name", "Peruvian/regional names"]}},
  "scientific_name": "...",
  "family": "botanical family",
  "crop_type": "annual" | "semi-perennial" | "perennial",
  "traits": {{
    "kc": {{"initial": TRAIT, "mid": TRAIT, "end": TRAIT}},
    "seasonal_water_need": TRAIT,
    "temperature": {{"base": TRAIT, "optimum": TRAIT, "max": TRAIT}},
    "n_fixation": {{"legume": TRAIT, "typical_range": TRAIT}},
    "pests": [{{"name": {{"en": "...", "es": "..."}}, "scientific_name": "...", "citation": CIT, "lower_threshold": TRAIT, "degree_days_per_generation": TRAIT}}],
    "beneficials": [{{"name": {{"en": "...", "es": "..."}}, "scientific_name": "...", "kind": "insect|mite|fungus|bacterium|nematode|virus", "controls": {{"en": "...", "es": "..."}}, "citation": CIT}}],
    "rotation_notes": {{"en": "1-2 factual sentences on botanical-family rotation or, for perennials, inter-row cover", "es": "...", "citation": CIT}} or {{"gap": "reason"}}
  }}
}}

Guidance: Kc from FAO-56 Table 12 or a published crop-coefficient study; temperature thresholds in °C; 2-4 main pests (lower developmental threshold in °C and degree-days per generation ONLY where a paper or extension model publishes them, otherwise gaps); 2-4 biological control agents (e.g. Trichoderma, Beauveria bassiana, Bacillus thuringiensis, Chrysoperla, parasitoid wasps) that are documented for this crop or its pests. Use at most {MAX_SEARCHES} searches; prefer sources covering several traits."#)
}

/// One research conversation with server-side web search: returns the final text and every URL the model actually
/// retrieved (citations are validated against these). Usage goes to `api_usage` (kind "research") when `db` is given.
pub(crate) async fn search_conversation(http: &reqwest::Client, api_key: &str, system: &str, prompt: String, tag: &str, max_tokens: u32, db: Option<&sqlx::MySqlPool>)
    -> anyhow::Result<(String, HashSet<String>)> {
    let tools = json!([{ "type": "web_search_20260209", "name": "web_search", "max_uses": MAX_SEARCHES }]);
    let mut messages = vec![json!({ "role": "user", "content": prompt })];
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut final_text = String::new();
    for attempt in 0..=MAX_CONTINUATIONS {
        let body = json!({
            "model": MODEL,
            "max_tokens": max_tokens,
            "system": system,
            "output_config": { "effort": "medium" },
            "fallbacks": "default",
            "tools": tools,
            "messages": messages,
        });
        let resp = http.post(API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .header("anthropic-beta", FALLBACK_BETA)
            .header("content-type", "application/json")
            .timeout(Duration::from_secs(420))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("anthropic request failed: {}", e.without_url()))?;
        let status = resp.status();
        let msg: Value = resp.json().await.map_err(|e| anyhow!("anthropic response unreadable: {}", e.without_url()))?;
        if let Some(db) = db {
            let call = common::usage::Call { analysis_id: None, kind: "research", turn_key: Some(tag), round: attempt as u32 + 1, model: MODEL };
            common::usage::record(db, &call, common::usage::Usage::of(&msg)).await;
        }
        if !status.is_success() {
            let m = msg.pointer("/error/message").and_then(Value::as_str).unwrap_or("unknown error");
            bail!("anthropic API {status}: {m}");
        }
        let content = msg["content"].as_array().cloned().unwrap_or_default();
        collect_urls(&content, &mut seen_urls);
        final_text = final_text_of(&content);
        match msg["stop_reason"].as_str() {
            Some("pause_turn") if attempt < MAX_CONTINUATIONS => {
                messages.truncate(1);
                messages.push(json!({ "role": "assistant", "content": content }));
                continue;
            }
            Some("refusal") => bail!("the research request was declined by the model"),
            _ => break,
        }
    }
    Ok((final_text, seen_urls))
}

/// Researches `query` with Claude and returns a validated record (origin ai-researched, labelled "AI-researched, verify").
/// `db`, when given, receives the token usage of every call (`api_usage`, kind "research").
pub async fn research(http: &reqwest::Client, api_key: &str, query: &str, db: Option<&sqlx::MySqlPool>) -> anyhow::Result<CropRecord> {
    let (final_text, seen_urls) = search_conversation(http, api_key, SYSTEM, user_prompt(query), query, MAX_TOKENS, db).await?;
    let v = extract_json(&final_text).context("research reply held no JSON object")?;
    if v.get("not_a_crop").and_then(Value::as_bool) == Some(true) {
        bail!("not recognized as a crop: {}", v.get("reason").and_then(Value::as_str).unwrap_or(""));
    }
    let (rec, dropped) = validate(&v, query, &seen_urls)?;
    tracing::info!("researched crop '{}' ({} values dropped to gaps)", rec.key, dropped.len());
    Ok(rec)
}

/// URLs the model actually retrieved: anything under a tool result block, plus text citations.
fn collect_urls(content: &[Value], seen: &mut HashSet<String>) {
    fn walk(v: &Value, seen: &mut HashSet<String>) {
        match v {
            Value::Object(o) => {
                if let Some(u) = o.get("url").and_then(Value::as_str) { seen.insert(norm_url(u)); }
                o.values().for_each(|x| walk(x, seen));
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, seen)),
            _ => {}
        }
    }
    for b in content {
        if b["type"].as_str().is_some_and(|t| t.ends_with("tool_result")) { walk(&b["content"], seen); }
        if let Some(cits) = b["citations"].as_array() { cits.iter().for_each(|c| walk(c, seen)); }
    }
}

/// Text blocks after the last tool result (the model's final answer).
fn final_text_of(content: &[Value]) -> String {
    let start = content.iter().rposition(|b| {
        b["type"].as_str().is_some_and(|t| t == "server_tool_use" || t.ends_with("tool_result"))
    }).map_or(0, |i| i + 1);
    content[start..].iter()
        .filter(|b| b["type"] == "text")
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn extract_json(text: &str) -> Option<Value> {
    let (a, b) = (text.find('{')?, text.rfind('}')?);
    serde_json::from_str(text.get(a..=b)?).ok()
}

pub(crate) fn norm_url(u: &str) -> String {
    let u = u.trim().to_lowercase();
    let u = u.split('#').next().unwrap_or("");
    let u = u.trim_start_matches("https://").trim_start_matches("http://").trim_start_matches("www.");
    u.trim_end_matches('/').to_string()
}

struct Ctx<'a> {
    seen: &'a HashSet<String>,
    dropped: Vec<String>,
}

impl Ctx<'_> {
    /// A citation is kept only with an http(s) URL that was among the retrieved pages (when any were retrieved).
    fn citation(&mut self, v: &Value, path: &str) -> Option<Citation> {
        let url = v.get("url").and_then(Value::as_str).unwrap_or("").trim();
        let ok_url = url.starts_with("https://") || url.starts_with("http://");
        let seen = self.seen.is_empty() || self.seen.contains(&norm_url(url));
        if !ok_url || !seen {
            self.dropped.push(path.to_string());
            return None;
        }
        Some(Citation {
            title: str_of(v.get("title")).unwrap_or_else(|| url.to_string()),
            author: str_of(v.get("author")).unwrap_or_default(),
            year: v.get("year").and_then(Value::as_i64).map(|y| y as i32),
            url: url.to_string(),
        })
    }

    fn trait_(&mut self, v: Option<&Value>, path: &str) -> Trait {
        let Some(v) = v.filter(|v| v.is_object()) else { return Trait::gap("not returned by research") };
        if let Some(g) = v.get("gap") {
            return Trait::gap(str_of(Some(g)).unwrap_or_else(|| "no cited value found".into()));
        }
        let Some(value) = v.get("value").and_then(val_of) else { return Trait::gap("no usable value returned") };
        let Some(citation) = v.get("citation").and_then(|c| self.citation(c, path)) else {
            return Trait::gap("dropped: value returned without a verifiable URL citation");
        };
        Trait::Value { value, unit: str_of(v.get("unit")).unwrap_or_default(), citation, note: str_of(v.get("note")) }
    }
}

fn str_of(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

fn val_of(v: &Value) -> Option<Val> {
    match v {
        Value::Bool(b) => Some(Val::Bool(*b)),
        Value::Number(n) => n.as_f64().map(Val::Num),
        Value::String(s) if !s.trim().is_empty() => Some(Val::Text(s.trim().into())),
        Value::Object(o) => Some(Val::Range { min: o.get("min")?.as_f64()?, max: o.get("max")?.as_f64()? }),
        _ => None,
    }
}

fn l10n(v: Option<&Value>) -> Option<L10n> {
    let v = v?;
    if let Some(s) = v.as_str() { return Some(L10n::new(s, s)); }
    let en = str_of(v.get("en"));
    let es = str_of(v.get("es"));
    match (en, es) {
        (Some(en), Some(es)) => Some(L10n { en, es }),
        (Some(x), None) | (None, Some(x)) => Some(L10n::new(x.clone(), x)),
        _ => None,
    }
}

fn names(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array).map(|a| {
        a.iter().filter_map(|x| str_of(Some(x))).map(|s| s.to_lowercase()).collect()
    }).unwrap_or_default()
}

/// Turns the model's JSON into a record: uncited values become gaps, uncited pests/beneficials are dropped.
/// Returns the record and the list of dropped paths.
pub(crate) fn validate(v: &Value, query: &str, seen: &HashSet<String>) -> anyhow::Result<(CropRecord, Vec<String>)> {
    let mut cx = Ctx { seen, dropped: vec![] };
    let t = &v["traits"];
    let crop_type = match v["crop_type"].as_str().map(|s| s.to_lowercase().replace(['_', ' '], "-")) {
        Some(s) if s == "annual" => CropType::Annual,
        Some(s) if s == "semi-perennial" || s == "semiperennial" => CropType::SemiPerennial,
        Some(s) if s == "perennial" => CropType::Perennial,
        other => bail!("research returned no valid crop_type ({other:?})"),
    };
    let mut en = names(v.pointer("/names/en"));
    let es = names(v.pointer("/names/es"));
    if en.is_empty() && es.is_empty() { en.push(query.trim().to_lowercase()); }
    let key = slug(en.first().or(es.first()).map(String::as_str).unwrap_or(query));

    let mut pests = vec![];
    for (i, p) in t["pests"].as_array().into_iter().flatten().enumerate() {
        let path = format!("pests[{i}]");
        let Some(name) = l10n(p.get("name")) else { continue };
        let Some(citation) = p.get("citation").and_then(|c| cx.citation(c, &path)) else { continue };
        pests.push(Pest {
            name,
            scientific_name: str_of(p.get("scientific_name")).unwrap_or_default(),
            citation,
            lower_threshold: cx.trait_(p.get("lower_threshold"), &format!("{path}.lower_threshold")),
            degree_days_per_generation: cx.trait_(p.get("degree_days_per_generation"), &format!("{path}.degree_days_per_generation")),
            note: str_of(p.get("note")),
            conditions: None,
        });
    }
    let mut beneficials = vec![];
    for (i, b) in t["beneficials"].as_array().into_iter().flatten().enumerate() {
        let path = format!("beneficials[{i}]");
        let (Some(name), Some(controls)) = (l10n(b.get("name")), l10n(b.get("controls"))) else { continue };
        let Some(citation) = b.get("citation").and_then(|c| cx.citation(c, &path)) else { continue };
        beneficials.push(Beneficial {
            name,
            scientific_name: str_of(b.get("scientific_name")).unwrap_or_default(),
            kind: str_of(b.get("kind")).unwrap_or_else(|| "unknown".into()),
            controls,
            citation,
            note: str_of(b.get("note")),
        });
    }
    let rn = &t["rotation_notes"];
    let rotation_notes = match (str_of(rn.get("en")), str_of(rn.get("es")), rn.get("citation")) {
        (Some(en), Some(es), Some(c)) => match cx.citation(c, "rotation_notes") {
            Some(citation) => RotationNotes::Note { en, es, citation },
            None => RotationNotes::Gap { gap: "dropped: rotation note without a verifiable URL citation".into() },
        },
        _ => RotationNotes::Gap { gap: str_of(rn.get("gap")).unwrap_or_else(|| "no cited rotation note found".into()) },
    };
    let traits = Traits {
        kc: Kc {
            initial: cx.trait_(t.pointer("/kc/initial"), "kc.initial"),
            mid: cx.trait_(t.pointer("/kc/mid"), "kc.mid"),
            end: cx.trait_(t.pointer("/kc/end"), "kc.end"),
        },
        seasonal_water_need: cx.trait_(t.get("seasonal_water_need"), "seasonal_water_need"),
        temperature: Temperature {
            base: cx.trait_(t.pointer("/temperature/base"), "temperature.base"),
            optimum: cx.trait_(t.pointer("/temperature/optimum"), "temperature.optimum"),
            max: cx.trait_(t.pointer("/temperature/max"), "temperature.max"),
        },
        n_fixation: NFixation {
            legume: cx.trait_(t.pointer("/n_fixation/legume"), "n_fixation.legume"),
            typical_range: cx.trait_(t.pointer("/n_fixation/typical_range"), "n_fixation.typical_range"),
        },
        pests,
        beneficials,
        rotation_notes,
        micronutrients: vec![], biostimulants: vec![], extras_reviewed: None, extras_gaps: vec![],
    };
    let q = query.trim().to_lowercase();
    let aliases = if en.contains(&q) || es.contains(&q) { vec![] } else { vec![q] };
    let rec = CropRecord {
        key,
        names: Names { en, es },
        scientific_name: str_of(v.get("scientific_name")).unwrap_or_default(),
        family: str_of(v.get("family")).unwrap_or_default(),
        crop_type,
        traits,
        origin: Origin::AiResearched,
        label: Some(LABEL.into()),
        aliases,
    };
    Ok((rec, cx.dropped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncited_values_become_gaps_and_unseen_urls_are_dropped() {
        let v = json!({
            "names": {"en": ["Dragon fruit"], "es": ["pitahaya"]},
            "scientific_name": "Selenicereus undatus", "family": "Cactaceae", "crop_type": "perennial",
            "traits": {
                "kc": {
                    "initial": {"value": 0.5, "unit": "-", "citation": {"title": "A", "author": "B", "year": 2020, "url": "https://example.org/a/"}},
                    "mid": {"value": 0.9, "unit": "-"},
                    "end": {"value": 0.7, "unit": "-", "citation": {"title": "C", "author": "D", "year": 2020, "url": "https://not-searched.org/x"}}
                },
                "seasonal_water_need": {"gap": "none found"},
                "temperature": {}, "n_fixation": {},
                "pests": [{"name": {"en": "x", "es": "y"}, "citation": {"title": "t", "url": "https://nope.org"}}],
                "beneficials": [],
                "rotation_notes": {"gap": "perennial"}
            }
        });
        let seen: HashSet<String> = [norm_url("https://www.example.org/a")].into();
        let (r, dropped) = validate(&v, "Pitahaya", &seen).unwrap();
        assert_eq!(r.key, "dragon_fruit");
        assert_eq!(r.crop_type, CropType::Perennial);
        assert_eq!(r.label.as_deref(), Some(LABEL));
        assert!(matches!(r.traits.kc.initial, Trait::Value { .. }));
        assert!(matches!(r.traits.kc.mid, Trait::Gap { .. }));
        assert!(matches!(r.traits.kc.end, Trait::Gap { .. }));
        assert!(r.traits.pests.is_empty());
        assert_eq!(dropped.len(), 2);
        for c in r.citations() { assert!(c.url.starts_with("http")); }
    }

    #[test]
    fn json_is_found_inside_text() {
        assert_eq!(extract_json("Here:\n```json\n{\"a\": 1}\n```").unwrap()["a"], 1);
    }
}
