//! Login dos moderadores. Não há rota de cadastro: contas nascem pelo binário
//! create_admin, rodado por quem tem acesso ao servidor.
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::auth::AdminIdentity;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::rate_limit::{client_key, LoginRateLimiter};
use api_types::AdminView;

use super::actions;

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(me);
}

#[post("/auth/login")]
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

    let LoginPayload { email, password } = payload.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let config_para_token = config.clone();

    let (admin, token) =
        web::block(move || actions::authenticate(&mut conn, &email, &password, &config_para_token))
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))??;

    // Login certo devolve a cota: quem só errou a senha uma vez não fica preso
    // na janela junto com quem estava tentando força bruta.
    limiter.reset(&rate_key);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "admin": AdminView::from(&admin),
    })))
}

/// O painel usa para saber se o token guardado ainda vale.
///
/// A conta já foi buscada no banco pelo extractor, então não há segunda ida.
#[get("/auth/me")]
pub async fn me(identity: AdminIdentity) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(AdminView {
        id: identity.id,
        name: identity.name,
        email: identity.email,
    }))
}
