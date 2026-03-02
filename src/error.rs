use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    InvalidUrl(String),
    CodeNotFound(String),
    CodeAlreadyExists(String),
    UrlExpired(String),
    DatabaseError(String),
    InternalError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            AppError::CodeNotFound(code) => write!(f, "Code not found: {}", code),
            AppError::CodeAlreadyExists(code) => write!(f, "Code already exists: {}", code),
            AppError::UrlExpired(code) => write!(f, "URL expired: {}", code),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::InvalidUrl(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_URL", msg.clone())
            }
            AppError::CodeNotFound(code) => {
                (StatusCode::NOT_FOUND, "CODE_NOT_FOUND",
                 format!("Short code '{}' not found", code))
            }
            AppError::CodeAlreadyExists(code) => {
                (StatusCode::CONFLICT, "CODE_EXISTS",
                 format!("Custom code '{}' already exists", code))
            }
            AppError::UrlExpired(code) => {
                (StatusCode::GONE, "URL_EXPIRED",
                 format!("Short URL '{}' has expired", code))
            }
            AppError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR",
                 "Internal database error".to_string())
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR",
                 "An internal error occurred".to_string())
            }
        };

        let body = json!({
            "error": {
                "code": error_code,
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}


impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}