//! Pedidos de correção.
//!
//! Quem está de fora sugere; quem decide é a moderação. Nada aqui altera uma
//! organização sem passar por um admin — a rota pública só enfileira.
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

use super::auth::ReviewEditRequests;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::rate_limit::{client_key, SubmissionRateLimiter};
use api_types::{EditRequestView, OrganizationView};
use db_types::edit_request::{EditRequestStatus, OrganizationChanges};

use super::actions;

#[derive(Debug, Deserialize)]
pub struct CreateEditRequestPayload {
    pub changes: OrganizationChanges,
    pub message: Option<String>,
    pub requester_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    /// "pending" (padrão), "applied", "rejected" ou "all".
    pub status: Option<String>,
}

/// Com o caminho no atributo de cada handler, a rota pública volta a morar
/// aqui. Antes ela era registrada no módulo de organizações: os escopos
/// disputavam o prefixo `/organizations`, e o primeiro que casasse passava a
/// ser dono da requisição — então um segundo escopo com o mesmo começo de
/// caminho nunca seria consultado. Sem escopo, não há disputa.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(index)
        .service(apply)
        .service(reject);
}

/// Pedido de correção vindo de fora do painel. Entra na fila; não altera
/// nada por conta própria.
#[post("/organizations/{id_or_slug}/edit-requests")]
pub async fn create(
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<CreateEditRequestPayload>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    limiter: web::Data<SubmissionRateLimiter>,
) -> Result<HttpResponse, ApiError> {
    // Rota aberta: sem limite por IP, um script enche a fila da moderação.
    // Conferido antes de qualquer trabalho, para nem gastar conexão com quem
    // já estourou a cota.
    limiter.check(&format!("edit:{}", client_key(&req, &config)))?;

    let identifier = path.into_inner();
    let body = payload.into_inner();

    actions::validate_changes(&body.changes)?;

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let request = web::block(move || {
        actions::submit(
            &mut conn,
            &identifier,
            &body.changes,
            body.message,
            body.requester_email,
        )
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    Ok(HttpResponse::Created().json(EditRequestView::from(&request)))
}

/// A fila de pedidos.
#[get("/admin/edit-requests")]
pub async fn index(
    w: ReviewEditRequests,
    query: web::Query<StatusQuery>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let status = parse_status(query.status.as_deref())?;

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let requests = web::block(move || actions::list(&mut conn, w, status))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let views: Vec<EditRequestView> = requests.iter().map(EditRequestView::from).collect();

    Ok(HttpResponse::Ok().json(views))
}

/// "all" é a única palavra que não é um estado do banco; ausente vale
/// "pending". Parse de query string, então mora na rota.
fn parse_status(requested: Option<&str>) -> Result<Option<String>, ApiError> {
    let requested = requested.unwrap_or(EditRequestStatus::PENDING);

    if requested == "all" {
        return Ok(None);
    }

    if !EditRequestStatus::is_valid(requested) {
        return Err(ApiError::ValidationError(
            vec![(
                "status".to_string(),
                vec![format!("Status inválido: {requested}")],
            )]
            .into_iter()
            .collect(),
        ));
    }

    Ok(Some(requested.to_string()))
}

/// Aplica o pedido no cadastro e o fecha.
#[post("/admin/edit-requests/{id}/apply")]
pub async fn apply(
    w: ReviewEditRequests,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) = web::block(move || actions::apply(&mut conn, w, request_id))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// Recusa o pedido. O cadastro fica como estava.
#[post("/admin/edit-requests/{id}/reject")]
pub async fn reject(
    w: ReviewEditRequests,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let request = web::block(move || actions::reject(&mut conn, w, request_id))
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))??;

    Ok(HttpResponse::Ok().json(EditRequestView::from(&request)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_status_means_the_queue_of_pending_ones() {
        assert_eq!(
            parse_status(None).unwrap().as_deref(),
            Some(EditRequestStatus::PENDING)
        );
    }

    #[test]
    fn all_is_the_word_that_lifts_the_filter() {
        assert!(parse_status(Some("all")).unwrap().is_none());
    }

    #[test]
    fn an_invented_status_is_a_validation_error_not_an_empty_queue() {
        // Sem isto, `?status=aplicado` responderia lista vazia e pareceria que
        // não há pedido nenhum.
        for inventado in ["aplicado", "APPLIED", "approved", ""] {
            assert!(
                matches!(
                    parse_status(Some(inventado)),
                    Err(ApiError::ValidationError(_))
                ),
                "{inventado:?} deveria ser recusado"
            );
        }
    }

    #[test]
    fn every_real_status_passes() {
        for valido in [
            EditRequestStatus::PENDING,
            EditRequestStatus::APPLIED,
            EditRequestStatus::REJECTED,
        ] {
            assert_eq!(parse_status(Some(valido)).unwrap().as_deref(), Some(valido));
        }
    }
}
