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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use validator::Validate;

    /// Corpo da resposta como JSON, que é o que o navegador realmente recebe.
    async fn body_of(error: &ApiError) -> serde_json::Value {
        let response = error.error_response();
        let bytes = to_bytes(response.into_body())
            .await
            .expect("o corpo deveria ser legível");

        serde_json::from_slice(&bytes).expect("o corpo deveria ser JSON")
    }

    #[test]
    fn maps_each_error_to_its_status() {
        assert_eq!(
            ApiError::NotFound.error_response().status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::Unauthorized("sem token".to_string())
                .error_response()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::ValidationError(HashMap::new())
                .error_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::FileUploadError("não é imagem".to_string())
                .error_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::PayloadTooLarge("grande demais".to_string())
                .error_response()
                .status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            ApiError::TooManyRequests(30).error_response().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ApiError::DatabaseError("qualquer coisa".to_string())
                .error_response()
                .status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            ApiError::InternalError("qualquer coisa".to_string())
                .error_response()
                .status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn a_database_error_never_reaches_the_client() {
        // A mensagem do Diesel carrega nome de tabela, de coluna e, no erro de
        // conexão, a própria DATABASE_URL — com senha dentro.
        let detalhe = "FATAL: password authentication failed for user \"docker\" \
                       (postgres://docker:senha-secreta@localhost/rave_map)";

        let body = body_of(&ApiError::DatabaseError(detalhe.to_string())).await;
        let texto = body.to_string();

        assert_eq!(body["message"], GENERIC_INTERNAL_MESSAGE);
        assert!(!texto.contains("senha-secreta"), "vazou a senha: {texto}");
        assert!(!texto.contains("rave_map"), "vazou o banco: {texto}");
        assert!(!texto.contains("docker"), "vazou o usuário: {texto}");
    }

    #[actix_web::test]
    async fn an_internal_error_never_reaches_the_client() {
        let body = body_of(&ApiError::InternalError(
            "falha ao abrir /home/deploy/.aws/credentials".to_string(),
        ))
        .await;

        assert_eq!(body["message"], GENERIC_INTERNAL_MESSAGE);
        assert!(
            !body.to_string().contains("/home/deploy"),
            "caminho do servidor é mapa grátis para quem sonda a API"
        );
    }

    #[actix_web::test]
    async fn an_upload_error_does_reach_the_client() {
        // Esta mensagem é escrita por nós, sobre o arquivo que a pessoa
        // mandou: devolver ajuda a corrigir o envio.
        let body = body_of(&ApiError::FileUploadError(
            "Envie imagens JPEG, PNG, GIF ou WebP".to_string(),
        ))
        .await;

        assert_eq!(body["message"], "Envie imagens JPEG, PNG, GIF ou WebP");
    }

    #[test]
    fn rate_limit_answers_with_retry_after() {
        // Sem o cabeçalho, o cliente só pode adivinhar quando tentar de novo.
        let response = ApiError::TooManyRequests(42).error_response();

        assert_eq!(
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some("42")
        );
    }

    #[actix_web::test]
    async fn validation_errors_come_back_field_by_field() {
        let errors: HashMap<String, Vec<String>> = vec![
            ("email".to_string(), vec!["Email inválido".to_string()]),
            ("uf".to_string(), vec!["UF deve ter 2 letras".to_string()]),
        ]
        .into_iter()
        .collect();

        let body = body_of(&ApiError::ValidationError(errors)).await;

        assert_eq!(body["message"], "Validation Fails");
        assert_eq!(body["errors"]["email"][0], "Email inválido");
        assert_eq!(body["errors"]["uf"][0], "UF deve ter 2 letras");
    }

    #[test]
    fn diesel_not_found_becomes_a_404_not_a_500() {
        // O caminho de sempre: cadastro que não existe (ou não está aprovado)
        // precisa responder 404, e não erro interno.
        let error = ApiError::from(diesel::result::Error::NotFound);

        assert!(matches!(error, ApiError::NotFound));
        assert_eq!(error.error_response().status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn other_diesel_errors_become_database_errors() {
        let error = ApiError::from(diesel::result::Error::RollbackTransaction);

        assert!(matches!(error, ApiError::DatabaseError(_)));
        assert_eq!(
            error.error_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn validator_errors_carry_the_message_of_each_field() {
        use db_types::organization::NewOrganization;
        use std::str::FromStr;

        let org = NewOrganization {
            name: String::new(),
            latitude: bigdecimal::BigDecimal::from_str("-30.03").unwrap(),
            longitude: bigdecimal::BigDecimal::from_str("-51.23").unwrap(),
            type_: "Coletivo".to_string(),
            city: "Porto Alegre".to_string(),
            uf: "RS".to_string(),
            email: "nao-e-email".to_string(),
            about: "Um coletivo de techno em Porto Alegre".to_string(),
            genres: vec!["Techno".to_string()],
            address: None,
            instagram: Some("@coletivo".to_string()),
            soundcloud: None,
            bandcamp: None,
            youtube: None,
            spotify: None,
            website: None,
            is_active: true,
            frequency: None,
            slug: "x".to_string(),
        };

        let error = ApiError::from(org.validate().unwrap_err());
        let body = body_of(&error).await;

        assert_eq!(body["errors"]["email"][0], "Email inválido");
        assert!(
            body["errors"]["name"][0]
                .as_str()
                .unwrap()
                .contains("obrigatório"),
            "a mensagem do campo deveria chegar ao formulário"
        );
    }

    #[actix_web::test]
    async fn io_errors_are_internal_and_hide_the_path() {
        let error = ApiError::from(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "/var/lib/rave-map/segredo",
        ));

        assert!(matches!(error, ApiError::InternalError(_)));
        assert_eq!(body_of(&error).await["message"], GENERIC_INTERNAL_MESSAGE);
    }
}
