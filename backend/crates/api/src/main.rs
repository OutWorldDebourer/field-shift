//! Field Shift HTTP API: loads config, connects to MariaDB, runs migrations and mounts every service router under /api.
use axum::{http::{HeaderValue, Method}, routing::get, Json, Router};
use common::{config::Config, AppState};
use sqlx::mysql::MySqlPoolOptions;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod health;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()))
        .init();
    let config = Config::load()?;
    tracing::info!("{config:?}");
    let db = MySqlPoolOptions::new().max_connections(10).connect(config.database_url()).await?;
    sqlx::migrate!("../../migrations").run(&db).await?;
    let bind = config.bind.clone();
    let state = AppState::new(db, config);
    let app = app(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("listening on http://{bind}");
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}

pub fn app(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health::health))
        .route("/version", get(|| async { Json(serde_json::json!({ "name": "field-shift", "version": env!("CARGO_PKG_VERSION") })) }))
        .nest("/power", power::routes())
        .nest("/imagery", imagery::routes())
        .nest("/earthdata", earthdata::routes())
        .nest("/climate", climate::routes())
        .nest("/crops", crops::routes())
        .nest("/analyses", analyses::routes())
        .nest("/consultant", consultant::routes())
        .nest("/export", export::routes());
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|o: &HeaderValue, _| {
            let o = o.as_bytes();
            o.starts_with(b"http://localhost:") || o.starts_with(b"http://127.0.0.1:")
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
    Router::new().nest("/api", api).layer(cors).with_state(state)
}
