use actix_web::{error::ResponseError, HttpResponse};
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation Fails")]
    ValidationError(HashMap<String, Vec<String>>),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found")]
    NotFound,

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("File upload error: {0}")]
    FileUploadError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Validation Fails",
                    "errors": errors
                }))
            }
            ApiError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "Error",
                    "message": format!("Internal server error: {}", msg)
                }))
            }
            ApiError::NotFound => HttpResponse::NotFound().json(serde_json::json!({
                "status": "Error",
                "message": "Resource not found"
            })),
            ApiError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "Error",
                    "message": format!("Internal server error: {}", msg)
                }))
            }
            ApiError::FileUploadError(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "status": "Error",
                "message": format!("File upload error: {}", msg)
            })),
        }
    }
}

// Convert Diesel errors to ApiError
impl From<diesel::result::Error> for ApiError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => ApiError::NotFound,
            _ => ApiError::DatabaseError(err.to_string()),
        }
    }
}

// Convert validation errors to ApiError
impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        let mut error_map: HashMap<String, Vec<String>> = HashMap::new();

        for (field, field_errors) in errors.field_errors() {
            let messages: Vec<String> = field_errors
                .iter()
                .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                .collect();
            error_map.insert(field.to_string(), messages);
        }

        ApiError::ValidationError(error_map)
    }
}

// Convert IO errors to ApiError
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}
