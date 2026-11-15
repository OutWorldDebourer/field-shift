//! Shared plumbing for every Field Shift service: configuration, app state, errors, geometry and the source cache.
pub mod cache;
pub mod text;
pub mod config;
pub mod error;
pub mod geo;
pub mod state;
pub mod usage;

pub use error::{ApiError, ApiResult};
pub use state::AppState;
