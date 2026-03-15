use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]

pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Database error:{0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Redis error:{0}")]
    RedisError(#[from] redis::RedisError),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type) = match self {
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Unauthorized(_) => {
                (actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED")
            }
            AppError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "CONFLICT"),
            AppError::InternalServerError
            | AppError::InternalError(_)
            | AppError::DatabaseError(_)
            | AppError::RedisError(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
            ),
        };
        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}
