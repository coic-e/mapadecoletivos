//! Login dos moderadores. Não há rota de cadastro: contas nascem pelo binário
//! create_admin, rodado por quem tem acesso ao servidor.
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::auth::{create_token, spend_verification_time, verify_password, AdminIdentity};
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::rate_limit::{client_key, LoginRateLimiter};
use api_types::AdminView;

use super::repository::AdminRepository;

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(me)),
    );
}

/// POST /auth/login
pub async fn login(
    req: HttpRequest,
    payload: web::Json<LoginPayload>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    limiter: web::Data<LoginRateLimiter>,
) -> Result<HttpResponse, ApiError> {
    // Sem limite, uma senha de moderador cai por força bruta na velocidade da
    // rede. A chave é o IP: limitar por e-mail deixaria um atacante travar a
    // conta de um moderador de propósito.
    let rate_key = format!("login:{}", client_key(&req, &config));
    limiter.check(&rate_key)?;

    let email = payload.email.trim().to_lowercase();
    let password = payload.password.clone();

    if email.len() > 320 || password.len() > 512 {
        return Err(ApiError::Unauthorized(
            "E-mail ou senha inválidos".to_string(),
        ));
    }

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let admin = web::block(move || AdminRepository::find_by_email(&mut conn, &email))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;

    // Mesma mensagem para e-mail inexistente e senha errada.
    let invalid = || ApiError::Unauthorized("E-mail ou senha inválidos".to_string());

    // E-mail que não existe gasta o mesmo tempo de um argon2 real, senão o
    // cronômetro do cliente denuncia quais contas existem.
    let admin = match admin {
        Some(admin) => admin,
        None => {
            spend_verification_time(&password);

            return Err(invalid());
        }
    };

    if !verify_password(&password, &admin.password_hash) {
        return Err(invalid());
    }

    // Login certo devolve a cota: quem só errou a senha uma vez não fica
    // preso na janela junto com quem estava tentando força bruta.
    limiter.reset(&rate_key);

    let token = create_token(admin.id, &admin.email, &config)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "admin": AdminView::from(&admin),
    })))
}

/// GET /auth/me — o painel usa para saber se o token guardado ainda vale.
///
/// A conta já foi buscada no banco pelo extractor, então não há segunda ida.
pub async fn me(identity: AdminIdentity) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(AdminView {
        id: identity.id,
        name: identity.name,
        email: identity.email,
    }))
}
