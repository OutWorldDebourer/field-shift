//! JSON cache of upstream responses in the `source_cache` table, so the demo keeps working when a source is slow or down.
use serde::{de::DeserializeOwned, Serialize};
use sqlx::MySqlPool;
use std::future::Future;

/// Returns the cached value for `key` if younger than `ttl_hours`; otherwise fetches and stores it.
/// If the fetch fails and any older copy exists, the stale copy is served (and a warning logged).
pub async fn get_or_fetch<T, F, Fut>(db: &MySqlPool, source: &str, key: &str, ttl_hours: i64, fetch: F) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let row: Option<(serde_json::Value, i64)> = sqlx::query_as(
        "SELECT payload, (fetched_at > NOW() - INTERVAL ? HOUR) + 0 FROM source_cache WHERE cache_key = ?",
    )
    .bind(ttl_hours)
    .bind(key)
    .fetch_optional(db)
    .await?;
    if let Some((payload, 1)) = &row {
        if let Ok(v) = serde_json::from_value(payload.clone()) {
            return Ok(v);
        }
    }
    match fetch().await {
        Ok(v) => {
            let payload = serde_json::to_value(&v)?;
            sqlx::query("REPLACE INTO source_cache (cache_key, source, payload, fetched_at) VALUES (?, ?, ?, NOW())")
                .bind(key)
                .bind(source)
                .bind(payload.to_string())
                .execute(db)
                .await?;
            Ok(v)
        }
        Err(e) => match row {
            Some((payload, _)) => {
                tracing::warn!("{source}: serving stale cache after error: {e}");
                Ok(serde_json::from_value(payload)?)
            }
            None => Err(e),
        },
    }
}
