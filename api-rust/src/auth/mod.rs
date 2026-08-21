//! Autenticação de moderadores: hash de senha, emissão de JWT e o extractor
//! que protege as rotas administrativas.
use std::sync::OnceLock;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::domains::admins::repository::AdminRepository;
use crate::errors::ApiError;

/// Hash descartável, com o formato de um argon2 real, usado para gastar o
/// mesmo tempo de verificação quando o e-mail não existe. Sem ele, o login
/// responde na hora para e-mail inexistente e devagar para e-mail existente —
/// diferença suficiente para enumerar contas de moderador pelo cronômetro.
static DUMMY_HASH: OnceLock<String> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Id do admin.
    pub sub: i32,
    pub email: String,
    /// Emissão, em segundos desde a epoch.
    pub iat: usize,
    /// Expiração, em segundos desde a epoch.
    pub exp: usize,
}

/// Gera o hash argon2 de uma senha. O salt é sorteado por chamada.
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| ApiError::InternalError(format!("Falha ao gerar hash da senha: {e}")))
}

/// Gasta o tempo de uma verificação de senha sem ter um hash real para
/// comparar, para que "e-mail não existe" custe o mesmo que "senha errada".
pub fn spend_verification_time(password: &str) {
    let dummy = DUMMY_HASH
        .get_or_init(|| hash_password("senha-que-nao-abre-nada").unwrap_or_else(|_| String::new()));

    if !dummy.is_empty() {
        let _ = verify_password(password, dummy);
    }
}

/// Confere a senha contra o hash. Hash malformado conta como senha errada.
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    match PasswordHash::new(password_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn create_token(admin_id: i32, email: &str, config: &AppConfig) -> Result<String, ApiError> {
    let expiration = Utc::now() + Duration::hours(config.jwt_ttl_hours);

    let claims = Claims {
        sub: admin_id,
        email: email.to_string(),
        iat: Utc::now().timestamp().max(0) as usize,
        exp: expiration.timestamp().max(0) as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalError(format!("Falha ao emitir token: {e}")))
}

pub fn decode_token(token: &str, config: &AppConfig) -> Result<Claims, ApiError> {
    // Algoritmo fixo. Aceitar o algoritmo que o próprio token declara é como um
    // atacante troca HS256 por "none" ou por uma confusão de chave. Exigir exp
    // fecha a outra porta: token sem validade nunca expira.
    //
    // sub fica de fora da lista obrigatória porque o nosso é numérico e a
    // checagem de claim padrão do jsonwebtoken só enxerga sub textual — a
    // presença dele já é garantida por ser campo não-opcional de Claims.
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp"]);
    validation.leeway = 5;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::Unauthorized("Sessão inválida ou expirada".to_string()))
}

/// Admin autenticado. Basta pedir este tipo no handler para a rota exigir um
/// Bearer token válido — sem ele, a requisição morre com 401 antes do handler.
#[derive(Debug, Clone)]
pub struct AdminIdentity {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl FromRequest for AdminIdentity {
    type Error = ApiError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let claims = admin_id_from_token(req);
        let pool = req.app_data::<web::Data<DbPool>>().cloned();

        Box::pin(async move {
            let admin_id = claims?;

            let pool = pool.ok_or_else(|| {
                ApiError::InternalError("pool de conexões indisponível".to_string())
            })?;

            let mut conn = pool
                .get()
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            // A conta é conferida no banco a cada requisição, não só na
            // assinatura do token. Sem isso, revogar um moderador não teria
            // efeito nenhum até o token expirar sozinho — quem foi removido
            // continuaria aprovando e rejeitando cadastros por horas.
            let admin = web::block(move || AdminRepository::find_by_id(&mut conn, admin_id))
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;

            match admin {
                Ok(admin) => Ok(AdminIdentity {
                    id: admin.id,
                    name: admin.name,
                    email: admin.email,
                }),
                Err(ApiError::NotFound) => Err(ApiError::Unauthorized(
                    "Sessão inválida ou expirada".to_string(),
                )),
                Err(e) => Err(e),
            }
        })
    }
}

/// Confere o Bearer token e devolve o id do admin que ele afirma ser.
fn admin_id_from_token(req: &HttpRequest) -> Result<i32, ApiError> {
    let config = req
        .app_data::<web::Data<AppConfig>>()
        .ok_or_else(|| ApiError::InternalError("Configuração indisponível".to_string()))?;

    let header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Token ausente".to_string()))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Formato de token inválido".to_string()))?;

    let claims = decode_token(token.trim(), config)?;

    Ok(claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_round_trip() {
        let hash = hash_password("senha-secreta").expect("deveria gerar hash");

        assert!(verify_password("senha-secreta", &hash));
        assert!(!verify_password("senha-errada", &hash));
    }

    #[test]
    fn hash_is_salted_per_call() {
        let first = hash_password("mesma-senha").unwrap();
        let second = hash_password("mesma-senha").unwrap();

        assert_ne!(first, second, "salt deveria variar entre chamadas");
    }

    #[test]
    fn malformed_hash_is_not_a_valid_password() {
        assert!(!verify_password("qualquer", "nao-e-um-hash"));
    }

    fn config_with_secret(secret: &str) -> AppConfig {
        AppConfig {
            storage: crate::config::StorageConfig {
                endpoint: "http://localhost:9000".to_string(),
                region: "us-east-1".to_string(),
                bucket: "rave-map".to_string(),
                access_key: "test".to_string(),
                secret_key: "test".to_string(),
                public_base_url: "http://localhost:9000/rave-map".to_string(),
                force_path_style: true,
            },
            database_url: String::new(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            max_file_size: 1024,
            max_files_per_request: 1,
            max_field_size: 1024,
            max_request_size: 1024,
            base_url: String::new(),
            jwt_secret: secret.to_string(),
            jwt_ttl_hours: 1,
            cors_allowed_origins: Vec::new(),
            trust_proxy: false,
            login_rate_limit: 5,
            submission_rate_limit: 5,
            rate_limit_window_secs: 60,
        }
    }

    #[test]
    fn round_trips_a_token() {
        let config = config_with_secret("um-segredo-de-teste-com-tamanho-suficiente");
        let token = create_token(7, "mod@exemplo.com", &config).unwrap();

        let claims = decode_token(&token, &config).unwrap();

        assert_eq!(claims.sub, 7);
        assert_eq!(claims.email, "mod@exemplo.com");
    }

    #[test]
    fn token_signed_with_another_secret_is_rejected() {
        let mine = config_with_secret("um-segredo-de-teste-com-tamanho-suficiente");
        let theirs = config_with_secret("outro-segredo-de-teste-com-tamanho-ok");

        let token = create_token(7, "mod@exemplo.com", &theirs).unwrap();

        assert!(decode_token(&token, &mine).is_err());
    }

    #[test]
    fn unsigned_alg_none_token_is_rejected() {
        // O ataque clássico: trocar o alg do cabeçalho por "none" e mandar a
        // assinatura vazia.
        let config = config_with_secret("um-segredo-de-teste-com-tamanho-suficiente");
        let forged = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0                      .eyJzdWIiOjEsImVtYWlsIjoiYUBiLmMiLCJpYXQiOjAsImV4cCI6OTk5OTk5OTk5OX0.";

        assert!(decode_token(forged, &config).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let config = config_with_secret("um-segredo-de-teste-com-tamanho-suficiente");
        let claims = Claims {
            sub: 1,
            email: "mod@exemplo.com".to_string(),
            iat: 0,
            exp: 1,
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .unwrap();

        assert!(decode_token(&token, &config).is_err());
    }
}
