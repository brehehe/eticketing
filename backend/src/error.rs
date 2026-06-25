use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error")]
    Validation(Vec<(String, String)>),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, errors) = match &self {
            AppError::NotFound(msg)     => (StatusCode::NOT_FOUND, msg.clone(), None::<serde_json::Value>),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), None::<serde_json::Value>),
            AppError::Forbidden(msg)    => (StatusCode::FORBIDDEN, msg.clone(), None::<serde_json::Value>),
            AppError::BadRequest(msg)   => (StatusCode::BAD_REQUEST, msg.clone(), None::<serde_json::Value>),
            AppError::Conflict(msg)     => (StatusCode::CONFLICT, msg.clone(), None::<serde_json::Value>),
            AppError::Validation(errs)  => {
                let map: serde_json::Value = errs.iter()
                    .fold(serde_json::Map::new(), |mut m, (k, v)| {
                        m.insert(k.clone(), json!([v]));
                        m
                    })
                    .into();
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
                    "success": false,
                    "message": "Validation failed",
                    "errors": map,
                }))).into_response();
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into(), None)
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into(), None)
            }
        };

        let body = json!({
            "success": false,
            "message": message,
            "errors": errors,
        });
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
