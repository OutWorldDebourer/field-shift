//! Per-visitor and global caps on live model turns. Visitors are identified by a salted hash of their IP, never the IP.
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

pub const PER_VISITOR_PER_HOUR: i64 = 30;
pub const GLOBAL_PER_DAY: i64 = 400;

pub fn visitor_id(ip: &str) -> String {
    let mut h = Sha256::new();
    h.update(b"field-shift-visitor:");
    h.update(ip.as_bytes());
    h.finalize().iter().take(12).map(|b| format!("{b:02x}")).collect()
}

/// Counts one live turn; returns false when a cap is reached (the caller then answers in demo mode).
pub async fn allow(db: &MySqlPool, visitor: &str) -> anyhow::Result<bool> {
    let buckets = [
        (format!("v:{visitor}:{}", chrono::Utc::now().format("%Y%m%d%H")), PER_VISITOR_PER_HOUR, 1),
        (format!("g:{}", chrono::Utc::now().format("%Y%m%d")), GLOBAL_PER_DAY, 24),
    ];
    for (bucket, _, hours) in &buckets {
        sqlx::query("INSERT INTO rate_limits (bucket, hits, window_end) VALUES (?, 0, NOW() + INTERVAL ? HOUR) ON DUPLICATE KEY UPDATE bucket = bucket")
            .bind(bucket).bind(hours).execute(db).await?;
    }
    for (bucket, cap, _) in &buckets {
        let hits: i64 = sqlx::query_scalar("SELECT hits FROM rate_limits WHERE bucket = ?").bind(bucket).fetch_one(db).await?;
        if hits >= *cap { return Ok(false); }
    }
    for (bucket, _, _) in &buckets {
        sqlx::query("UPDATE rate_limits SET hits = hits + 1 WHERE bucket = ?").bind(bucket).execute(db).await?;
    }
    sqlx::query("DELETE FROM rate_limits WHERE window_end < NOW()").execute(db).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn visitor_id_hides_ip() {
        let v = super::visitor_id("203.0.113.9");
        assert_eq!(v.len(), 24);
        assert!(!v.contains("203"));
    }
}
