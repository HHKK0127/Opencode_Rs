use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;
use std::fmt;
use crate::storage::StorageError;

#[derive(Debug)]
pub enum AppError {
    Database(String),
    Unauthorized,
    BadRequest(String),
    Conflict(String),
    NotFound,
    PayloadTooLarge,
    Internal,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::BadRequest(e) => write!(f, "Bad request: {}", e),
            AppError::Conflict(e) => write!(f, "Conflict: {}", e),
            AppError::NotFound => write!(f, "Not found"),
            AppError::PayloadTooLarge => write!(f, "Payload too large"),
            AppError::Internal => write!(f, "Internal server error"),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string(),
            "code": self.status_code().as_u16()
        }))
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<StorageError> for AppError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound(_) => AppError::NotFound,
            StorageError::InvalidConfig(msg) => AppError::BadRequest(msg),
            _ => AppError::Internal,
        }
    }
}
