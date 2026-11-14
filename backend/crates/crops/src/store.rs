//! `crop_traits` table: AI-researched records cached forever so each unknown crop is researched once.
use crate::model::CropRecord;
use sqlx::MySqlPool;

/// All cached AI-researched records (small table; read whole). DB errors give an empty list.
pub async fn researched(db: &MySqlPool) -> Vec<CropRecord> {
    let rows: Result<Vec<(String,)>, _> =
        sqlx::query_as("SELECT CAST(record AS CHAR) FROM crop_traits WHERE origin = 'ai-researched'")
            .fetch_all(db)
            .await;
    match rows {
        Ok(rows) => rows.into_iter().filter_map(|(r,)| serde_json::from_str(&r).ok()).collect(),
        Err(e) => {
            tracing::warn!("crop_traits read failed: {e}");
            vec![]
        }
    }
}

pub async fn get(db: &MySqlPool, key: &str) -> anyhow::Result<Option<CropRecord>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT CAST(record AS CHAR) FROM crop_traits WHERE crop_key = ?")
        .bind(key)
        .fetch_optional(db)
        .await?;
    Ok(row.and_then(|(r,)| serde_json::from_str(&r).ok()))
}

/// Stores a researched record under `key` (first write wins).
pub async fn put(db: &MySqlPool, key: &str, rec: &CropRecord) -> anyhow::Result<()> {
    sqlx::query("INSERT IGNORE INTO crop_traits (crop_key, record, origin) VALUES (?, ?, 'ai-researched')")
        .bind(key)
        .bind(serde_json::to_string(rec)?)
        .execute(db)
        .await?;
    Ok(())
}
