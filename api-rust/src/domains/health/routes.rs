//! Sinais de saúde da API.
//!
//! São dois, de propósito, porque respondem perguntas diferentes:
//!
//! - `/health` diz que o processo está de pé. Não toca em banco nem bucket, e
//!   é o que deve alimentar política de reinício: derrubar o container porque
//!   o Postgres piscou não conserta nada e ainda tira a API do ar junto.
//! - `/health/ready` diz que a API consegue atender de verdade. É o que um
//!   balanceador deve consultar para decidir se manda tráfego.
use std::time::Duration;

use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::sql_query;

use crate::db::DbPool;
use crate::storage::Storage;

/// Teto para cada verificação. Sem isso, dependência travada deixa a sonda
/// pendurada e o orquestrador interpreta como "sem resposta" muito depois.
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(live))
        .route("/health/ready", web::get().to(ready));
}

/// GET /health — o processo responde.
pub async fn live() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

/// GET /health/ready — banco e bucket alcançáveis.
///
/// Responde 503 quando alguma dependência falha, com o detalhe de qual: sonda
/// que só diz "não" obriga quem está de plantão a adivinhar.
pub async fn ready(pool: web::Data<DbPool>, storage: web::Data<Storage>) -> HttpResponse {
    let database = check_database(&pool).await;
    let bucket = check_storage(&storage).await;

    let healthy = database.is_ok() && bucket.is_ok();

    let body = serde_json::json!({
        "status": if healthy { "ok" } else { "unavailable" },
        "database": match &database {
            Ok(()) => serde_json::json!("ok"),
            Err(e) => serde_json::json!({ "error": e }),
        },
        "storage": match &bucket {
            Ok(()) => serde_json::json!("ok"),
            Err(e) => serde_json::json!({ "error": e }),
        },
    });

    if healthy {
        HttpResponse::Ok().json(body)
    } else {
        HttpResponse::ServiceUnavailable().json(body)
    }
}

async fn check_database(pool: &DbPool) -> Result<(), String> {
    let pool = pool.clone();

    let consulta = web::block(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;

        sql_query("SELECT 1")
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok::<(), String>(())
    });

    match tokio::time::timeout(CHECK_TIMEOUT, consulta).await {
        Ok(Ok(resultado)) => resultado,
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("tempo esgotado".to_string()),
    }
}

async fn check_storage(storage: &Storage) -> Result<(), String> {
    match tokio::time::timeout(CHECK_TIMEOUT, storage.check()).await {
        Ok(resultado) => resultado.map_err(|e| e.to_string()),
        Err(_) => Err("tempo esgotado".to_string()),
    }
}
