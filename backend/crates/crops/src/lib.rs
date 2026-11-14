//! Open crop knowledge: any crop name the farmer types resolves to a cited crop record (verified seed,
//! cached AI research, or new AI research), and the advice engine turns it into crop-type-aware guidance.
//! Routes under `/api/crops`: `GET /suggest?q=`, `GET /resolve?q=`, `POST /advice`, `GET /` (seed list).
pub mod advice;
pub mod model;
pub mod normalize;
pub mod research;
pub mod seed;
pub mod store;

pub use advice::{advice, Advice, FieldSignals};
pub use model::{CropRecord, CropType, Origin};

use axum::{extract::{Query, State}, routing::{get, post}, Json, Router};
use common::{ApiError, ApiResult, AppState};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/suggest", get(suggest))
        .route("/resolve", get(resolve_route))
        .route("/advice", post(advice_route))
}

/// One research job at a time, so concurrent requests for the same unknown crop trigger one call only.
static RESEARCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn check_query(q: &str) -> ApiResult<&str> {
    let q = q.trim();
    if q.is_empty() || q.chars().count() > 80 || !q.chars().any(char::is_alphabetic) {
        return Err(ApiError::BadRequest("type a crop name (1-80 characters)".into()));
    }
    Ok(q)
}

/// Resolves free text to a crop record: seed first, then cached AI research, else researches it once with Claude.
pub async fn resolve(state: &AppState, q: &str) -> ApiResult<CropRecord> {
    let q = check_query(q)?;
    if let Some(m) = normalize::best(q, seed::all()) {
        return seed::get(&m.key).cloned().ok_or(ApiError::NotFound);
    }
    let cached = store::researched(&state.db).await;
    if let Some(m) = normalize::best(q, &cached) {
        return cached.into_iter().find(|c| c.key == m.key).ok_or(ApiError::NotFound);
    }
    let _guard = RESEARCH_LOCK.lock().await;
    // Another request may have researched it while we waited.
    let cached = store::researched(&state.db).await;
    if let Some(m) = normalize::best(q, &cached) {
        return cached.into_iter().find(|c| c.key == m.key).ok_or(ApiError::NotFound);
    }
    let key = state.config.secrets.anthropic.as_deref()
        .ok_or_else(|| ApiError::Upstream("crop not in the knowledge base and AI research is not configured".into()))?;
    let rec = research::research(&state.http, key, q).await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    if let Some(s) = seed::get(&rec.key) {
        return Ok(s.clone());
    }
    store::put(&state.db, &rec.key, &rec).await?;
    Ok(store::get(&state.db, &rec.key).await?.unwrap_or(rec))
}

#[derive(Deserialize)]
struct Q {
    #[serde(default)]
    q: String,
}

#[derive(Serialize)]
struct Suggestion {
    key: String,
    name: model::L10n,
    scientific_name: String,
    family: String,
    crop_type: CropType,
    origin: Origin,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    matched: String,
    score: u8,
}

#[derive(Serialize)]
struct SuggestResponse {
    query: String,
    suggestions: Vec<Suggestion>,
    /// Always true: suggestions help typing; any crop name is accepted and researched if unknown.
    accepts_any: bool,
}

async fn suggest(State(state): State<AppState>, Query(Q { q }): Query<Q>) -> ApiResult<Json<SuggestResponse>> {
    let q = q.trim().to_string();
    if q.is_empty() {
        return Ok(Json(SuggestResponse { query: q, suggestions: vec![], accepts_any: true }));
    }
    let cached = store::researched(&state.db).await;
    let all: Vec<&CropRecord> = seed::all().iter().chain(cached.iter()).collect();
    let suggestions = normalize::rank(&q, all.iter().copied()).into_iter().take(8).filter_map(|m| {
        let c = all.iter().find(|c| c.key == m.key)?;
        Some(Suggestion {
            key: c.key.clone(), name: c.name(), scientific_name: c.scientific_name.clone(), family: c.family.clone(),
            crop_type: c.crop_type, origin: c.origin, label: c.label.clone(), matched: m.matched, score: m.score,
        })
    }).collect();
    Ok(Json(SuggestResponse { query: q, suggestions, accepts_any: true }))
}

async fn resolve_route(State(state): State<AppState>, Query(Q { q }): Query<Q>) -> ApiResult<Json<CropRecord>> {
    Ok(Json(resolve(&state, &q).await?))
}

async fn list() -> Json<Vec<advice::CropSummary>> {
    Json(seed::all().iter().map(advice::summary).collect())
}

#[derive(Deserialize)]
struct AdviceRequest {
    current: String,
    #[serde(default)]
    previous: Vec<String>,
    #[serde(default)]
    signals: FieldSignals,
}

async fn advice_route(State(state): State<AppState>, Json(req): Json<AdviceRequest>) -> ApiResult<Json<Advice>> {
    if req.previous.len() > 6 {
        return Err(ApiError::BadRequest("at most 6 previous crops".into()));
    }
    let current = resolve(&state, &req.current).await?;
    let mut previous = Vec::with_capacity(req.previous.len());
    for p in &req.previous {
        previous.push(resolve(&state, p).await?);
    }
    Ok(Json(advice(&current, &previous, &req.signals)))
}

#[cfg(test)]
mod network_tests {
    use super::*;

    /// Researches one unseeded crop with Claude (one real API call). Run with `cargo test -p crops -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore]
    async fn research_pitahaya_with_claude() {
        let cfg = common::config::Config::load().expect("config");
        let key = cfg.secrets.anthropic.clone().expect("anthropic key in env.txt");
        let http = reqwest::Client::builder().user_agent("FieldShift/0.1 (NASA Space Apps 2026)").build().unwrap();
        let rec = research::research(&http, &key, "pitahaya").await.expect("research");
        assert_eq!(rec.origin, Origin::AiResearched);
        assert_eq!(rec.label.as_deref(), Some(research::LABEL));
        for c in rec.citations() { assert!(c.url.starts_with("http"), "{c:?}"); }
        let leaves = rec.trait_leaves();
        let cited = leaves.iter().filter(|(_, t)| t.citation().is_some()).count();
        println!("key={} sci={} family={} type={:?} names={:?}", rec.key, rec.scientific_name, rec.family, rec.crop_type, rec.names);
        println!("cited values {cited}/{} ; pests {} ; beneficials {}", leaves.len(), rec.traits.pests.len(), rec.traits.beneficials.len());
        for (p, t) in &leaves {
            println!("  {p}: {}", t.display().map(|d| format!("{d}  <{}>", t.citation().unwrap().url)).unwrap_or_else(|| format!("{t:?}")));
        }
        for p in &rec.traits.pests { println!("  pest {} ({}) <{}>", p.name.en, p.scientific_name, p.citation.url); }
        for b in &rec.traits.beneficials { println!("  beneficial {} -> {} <{}>", b.name.en, b.controls.en, b.citation.url); }
        let a = advice(&rec, &[], &FieldSignals::default());
        println!("advice mode {:?}, strategies {}, practices {}", a.mode, a.strategies.len(), a.practices.len());
        println!("{}", serde_json::to_string(&rec).unwrap());
    }
}
