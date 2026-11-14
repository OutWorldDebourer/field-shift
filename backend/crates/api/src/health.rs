//! Health endpoint: database reachable, which secrets are configured (names only, never values).
use axum::{extract::State, Json};
use common::AppState;
use serde_json::{json, Value};

pub async fn health(State(s): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&s.db).await.is_ok();
    let sec = &s.config.secrets;
    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "database": db_ok,
        "keys": {
            "nasa_api": sec.nasa_api.is_some(),
            "earthdata": sec.earthdata.is_some(),
            "anthropic": sec.anthropic.is_some(),
        }
    }))
}
