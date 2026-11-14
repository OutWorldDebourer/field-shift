//! analyses service (stub, filled in by its milestone).
use axum::Router;
use common::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
