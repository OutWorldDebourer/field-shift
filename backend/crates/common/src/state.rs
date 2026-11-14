//! Shared application state handed to every router.
use crate::config::Config;
use sqlx::MySqlPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub http: reqwest::Client,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: MySqlPool, config: Config) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("FieldShift/0.1 (NASA Space Apps 2026)")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("http client");
        Self { db, http, config: Arc::new(config) }
    }
}
