//! Photo files live on disk under `<root>/backend/uploads/<analysis>/` (git-ignored); the DB keeps references.
use common::{ApiError, ApiResult, AppState};
use std::path::PathBuf;

pub const MAX_PHOTOS_PER_MESSAGE: usize = 10;
pub const MAX_PHOTO_BYTES: usize = 8 * 1024 * 1024;

pub fn uploads_dir(state: &AppState) -> PathBuf { state.config.root.join("backend").join("uploads") }

pub struct StoredPhoto {
    pub id: i64,
    pub url: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

fn ext_for(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

/// Saves one photo and records it; returns its id and public URL.
pub async fn save(state: &AppState, analysis: &str, mime: &str, bytes: Vec<u8>) -> ApiResult<StoredPhoto> {
    let ext = ext_for(mime).ok_or_else(|| ApiError::BadRequest(format!("unsupported image type {mime}")))?;
    if bytes.len() > MAX_PHOTO_BYTES { return Err(ApiError::BadRequest("photo larger than 8 MB".into())); }
    let dir = uploads_dir(state).join(analysis);
    tokio::fs::create_dir_all(&dir).await.map_err(|e| ApiError::Other(e.into()))?;
    let r = sqlx::query("INSERT INTO photos (analysis_id, file_path, mime, bytes) VALUES (?, '', ?, ?)")
        .bind(analysis).bind(mime).bind(bytes.len() as i32).execute(&state.db).await?;
    let id = r.last_insert_id() as i64;
    let rel = format!("{analysis}/{id}.{ext}");
    tokio::fs::write(dir.join(format!("{id}.{ext}")), &bytes).await.map_err(|e| ApiError::Other(e.into()))?;
    sqlx::query("UPDATE photos SET file_path = ? WHERE id = ?").bind(&rel).bind(id).execute(&state.db).await?;
    Ok(StoredPhoto { id, url: format!("/api/analyses/{analysis}/photos/{id}"), mime: mime.into(), bytes })
}

pub async fn read(state: &AppState, analysis: &str, id: i64) -> ApiResult<(String, Vec<u8>)> {
    let row: Option<(String, String)> = sqlx::query_as("SELECT file_path, mime FROM photos WHERE id = ? AND analysis_id = ?")
        .bind(id).bind(analysis).fetch_optional(&state.db).await?;
    let (rel, mime) = row.ok_or(ApiError::NotFound)?;
    let bytes = tokio::fs::read(uploads_dir(state).join(rel)).await.map_err(|_| ApiError::NotFound)?;
    Ok((mime, bytes))
}

pub async fn set_message(state: &AppState, photo_ids: &[i64], message_id: i64) -> ApiResult<()> {
    for id in photo_ids {
        sqlx::query("UPDATE photos SET message_id = ? WHERE id = ?").bind(message_id).bind(id).execute(&state.db).await?;
    }
    Ok(())
}
