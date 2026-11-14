//! One error type for HTTP handlers: JSON body `{ "error": "..." }`, never leaking secrets or URLs with tokens.
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("too many requests, try again in a minute")]
    RateLimited,
    #[error("upstream source failed: {0}")]
    Upstream(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self { ApiError::Other(e.into()) }
}
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self { ApiError::Upstream(e.without_url().to_string()) }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Upstream(_) => StatusCode::BAD_GATEWAY,
            ApiError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if code.is_server_error() { tracing::error!("{self}"); }
        (code, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}
