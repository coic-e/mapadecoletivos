use actix_web::http::header;
use actix_web::{error::ResponseError, HttpResponse};
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationErrors;

/// Erros internos ficam no log do servidor, nunca na resposta: a mensagem do
/// Diesel carrega nome de tabela, de coluna e às vezes a string de conexão, e
/// a do io::Error carrega caminho do sistema de arquivos. Isso é mapa grátis
/// para quem está sondando a API.
const GENERIC_INTERNAL_MESSAGE: &str = "Erro interno. Tente de novo mais tarde.";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation Fails")]
    ValidationError(HashMap<String, Vec<String>>),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("File upload error: {0}")]
    FileUploadError(String),

    /// Limite de requisições estourado; o número é o retry-after em segundos.
    #[error("Too many requests")]
    TooManyRequests(u64),

    #[error("Payload too large")]
    PayloadTooLarge(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "Error",
                    "message": "Validation Fails",
                    "errors": errors
                }))
            }
            // Detalhe só no log.
            ApiError::DatabaseError(detail) => {
                log::error!("erro de banco: {detail}");

                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "Error",
                    "message": GENERIC_INTERNAL_MESSAGE
                }))
            }
            ApiError::NotFound => HttpResponse::NotFound().json(serde_json::json!({
                "status": "Error",
                "message": "Resource not found"
            })),
            ApiError::Unauthorized(msg) => HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "Error",
                "message": msg
            })),
            ApiError::InternalError(detail) => {
                log::error!("erro interno: {detail}");

                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "Error",
                    "message": GENERIC_INTERNAL_MESSAGE
                }))
            }
            // Mensagem de upload é escrita por nós, sobre o arquivo que a
            // pessoa mandou, então pode voltar: ajuda a corrigir o envio.
            ApiError::FileUploadError(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "status": "Error",
                "message": msg
            })),
            ApiError::TooManyRequests(retry_after) => HttpResponse::TooManyRequests()
                .insert_header((header::RETRY_AFTER, retry_after.to_string()))
                .json(serde_json::json!({
                    "status": "Error",
                    "message": "Muitas requisições. Espere um pouco e tente de novo."
                })),
            ApiError::PayloadTooLarge(msg) => {
                HttpResponse::PayloadTooLarge().json(serde_json::json!({
                    "status": "Error",
                    "message": msg
                }))
            }
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
