//! Request context shared by the SMAP / IMERG readers: HTTP client, bearer token and optional source cache.
use common::AppState;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::MySqlPool;
use std::future::Future;

#[derive(Clone)]
pub struct Ctx {
    pub http: reqwest::Client,
    pub token: String,
    /// None in tests: values are fetched without caching.
    pub db: Option<MySqlPool>,
}

impl Ctx {
    pub fn from_state(state: &AppState) -> anyhow::Result<Self> {
        Ok(Self { http: state.http.clone(), token: crate::client::token(state)?, db: Some(state.db.clone()) })
    }

    /// `common::cache::get_or_fetch` when a database is attached, a plain fetch otherwise.
    pub async fn cached<T, F, Fut>(&self, source: &str, key: &str, ttl_hours: i64, fetch: F) -> anyhow::Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<T>>,
    {
        match &self.db {
            Some(db) => common::cache::get_or_fetch(db, source, key, ttl_hours, fetch).await,
            None => fetch().await,
        }
    }
}
