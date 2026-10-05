//! Unified application error type. Every layer (application services, infrastructure
//! repositories, HTTP handlers) converges on `AppError`, which knows how to render
//! itself as an HTTP response at the edge (`IntoResponse`).

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    /// A real, honest "this feature isn't wired up yet" — e.g. the payment gateway has
    /// no provider configured. Distinct from `Internal`: this isn't a bug, it's a known
    /// and expected state that should be surfaced to the caller as-is, not logged as an
    /// error or masked as a generic 500.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),

    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Maps a Postgres foreign-key-violation error (SQLSTATE 23503) to a friendly
    /// `Conflict` instead of a generic 500 — used by repository `delete()` impls where
    /// the row might still be referenced by other tables (e.g. deleting a user who
    /// authored questions, or a session that already has attempts).
    pub fn from_sqlx_delete(e: sqlx::Error, conflict_message: impl Into<String>) -> AppError {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23503") {
                return AppError::Conflict(conflict_message.into());
            }
        }
        AppError::Database(e)
    }
}
