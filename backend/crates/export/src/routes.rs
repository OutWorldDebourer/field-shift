//! Routes under /api/export: technical sheet (HTML, print to PDF), CSV, share links (read-only view by token).
use crate::{csv, sheet};
use analyses::store;
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use common::{ApiResult, AppState};
use serde::Deserialize;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/sheet", get(sheet_html))
        .route("/{id}/series.csv", get(series_csv))
        .route("/{id}/share", post(share))
        .route("/{id}/package.zip", get(package_zip))
        .route("/shared/{token}", get(shared))
        .route("/shared/{token}/sheet", get(shared_sheet))
}

#[derive(Deserialize)]
struct LangQ { lang: Option<String> }

async fn sheet_html(State(s): State<AppState>, Path(id): Path<String>, Query(q): Query<LangQ>) -> ApiResult<Html<String>> {
    let st = store::load(&s.db, &id).await?;
    let lang = q.lang.unwrap_or_else(|| st.lang.clone());
    Ok(Html(sheet::render(&st, &lang, &format!("/api/export/{id}/series.csv"))))
}

async fn series_csv(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    let ev = store::latest_evidence(&s.db, &id).await?.unwrap_or(serde_json::Value::Null);
    Ok((
        [(header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"field-shift-evidence.csv\"".to_string())],
        csv::render(&ev),
    ))
}

async fn share(State(s): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    let token = store::share_token(&s.db, &id).await?;
    Ok(Json(serde_json::json!({ "token": token, "url": format!("/app.html?share={token}"), "sheet": format!("/api/export/shared/{token}/sheet") })))
}

/// Read-only analysis for a share link: same state, without photos of the conversation (they stay private).
async fn shared(State(s): State<AppState>, Path(token): Path<String>) -> ApiResult<Json<analyses::AnalysisState>> {
    let id = store::by_share_token(&s.db, &token).await?;
    let mut st = store::load(&s.db, &id).await?;
    for m in &mut st.messages { m.photos.clear(); }
    Ok(Json(st))
}

async fn shared_sheet(State(s): State<AppState>, Path(token): Path<String>, Query(q): Query<LangQ>) -> ApiResult<Html<String>> {
    let id = store::by_share_token(&s.db, &token).await?;
    let st = store::load(&s.db, &id).await?;
    let lang = q.lang.unwrap_or_else(|| st.lang.clone());
    Ok(Html(sheet::render(&st, &lang, &format!("/api/export/{id}/series.csv"))))
}

async fn package_zip(State(s): State<AppState>, Path(id): Path<String>, Query(q): Query<LangQ>) -> ApiResult<impl IntoResponse> {
    let lang = match q.lang { Some(l) => l, None => store::load(&s.db, &id).await?.lang };
    let bytes = crate::package::build(&s, &id, &lang).await?;
    Ok(([(header::CONTENT_TYPE, "application/zip".to_string()),
         (header::CONTENT_DISPOSITION, "attachment; filename=\"field-shift-package.zip\"".to_string())], bytes))
}
