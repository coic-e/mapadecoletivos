//! Pedidos de correção.
//!
//! Quem está de fora sugere; quem decide é a moderação. Nada aqui altera uma
//! organização sem passar por um admin — a rota pública só enfileira.
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use crate::auth::AdminIdentity;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::rate_limit::{client_key, SubmissionRateLimiter};
use api_types::{EditRequestView, OrganizationView};
use db_types::edit_request::{EditRequestStatus, NewEditRequest, OrganizationChanges};

use super::repository::EditRequestRepository;
use crate::domains::organizations::repository::OrganizationRepository;

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

/// Só as rotas administrativas. A pública mora dentro do escopo
/// `/organizations`, em domains::organizations::routes: no actix, o primeiro
/// escopo que casa com o prefixo passa a ser dono da requisição, então um
/// segundo escopo com o mesmo começo de caminho nunca seria consultado.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/edit-requests")
            .route("", web::get().to(index))
            .route("/{id}/apply", web::post().to(apply))
            .route("/{id}/reject", web::post().to(reject)),
    );
}

/// POST /organizations/{id_ou_slug}/edit-requests
pub async fn create(
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<CreateEditRequestPayload>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    limiter: web::Data<SubmissionRateLimiter>,
) -> Result<HttpResponse, ApiError> {
    // Rota aberta: sem limite por IP, um script enche a fila da moderação.
    limiter.check(&format!("edit:{}", client_key(&req, &config)))?;

    let identifier = path.into_inner();
    let body = payload.into_inner();

    if body.changes.is_empty() {
        return Err(ApiError::ValidationError(
            vec![(
                "changes".to_string(),
                vec!["Informe ao menos um campo para corrigir".to_string()],
            )]
            .into_iter()
            .collect(),
        ));
    }

    body.changes.validate().map_err(ApiError::from)?;

    if let Err(reason) = body.changes.validate_closed_lists() {
        return Err(ApiError::ValidationError(
            vec![("changes".to_string(), vec![reason])]
                .into_iter()
                .collect(),
        ));
    }

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let request = web::block(move || {
        // Só cadastro público aceita sugestão: pendente e rejeitado não
        // existem para quem está de fora, e aceitar pedido para eles
        // confirmaria que existem.
        let (organization, _) = match identifier.parse::<i32>() {
            Ok(id) => OrganizationRepository::find_approved_by_id(&mut conn, id),
            Err(_) => OrganizationRepository::find_approved_by_slug(&mut conn, &identifier),
        }?;

        let changes = serde_json::to_value(&body.changes)
            .map_err(|e| ApiError::InternalError(format!("Falha ao serializar pedido: {e}")))?;

        EditRequestRepository::create(
            &mut conn,
            &NewEditRequest {
                organization_id: organization.id,
                changes,
                message: body.message.clone(),
                requester_email: body.requester_email.clone(),
            },
        )
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    Ok(HttpResponse::Created().json(EditRequestView::from(&request)))
}

/// GET /admin/edit-requests?status=pending
pub async fn index(
    _identity: AdminIdentity,
    query: web::Query<StatusQuery>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let requested = query
        .status
        .clone()
        .unwrap_or_else(|| EditRequestStatus::PENDING.to_string());

    let status = if requested == "all" {
        None
    } else {
        if !EditRequestStatus::is_valid(&requested) {
            return Err(ApiError::ValidationError(
                vec![(
                    "status".to_string(),
                    vec![format!("Status inválido: {requested}")],
                )]
                .into_iter()
                .collect(),
            ));
        }

        Some(requested)
    };

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let requests = web::block(move || {
        EditRequestRepository::find_all_with_status(&mut conn, status.as_deref())
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let views: Vec<EditRequestView> = requests.iter().map(EditRequestView::from).collect();

    Ok(HttpResponse::Ok().json(views))
}

/// POST /admin/edit-requests/{id}/apply
pub async fn apply(
    identity: AdminIdentity,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) = web::block(move || {
        let request = EditRequestRepository::find_by_id(&mut conn, request_id)?;

        if request.status != EditRequestStatus::PENDING {
            return Err(ApiError::ValidationError(
                vec![(
                    "status".to_string(),
                    vec!["Este pedido já foi revisado".to_string()],
                )]
                .into_iter()
                .collect(),
            ));
        }

        // Validado de novo na aplicação: o pedido pode ter ficado na fila
        // enquanto as regras mudaram, e é aqui que ele vira dado público.
        let changes: OrganizationChanges = serde_json::from_value(request.changes.clone())
            .map_err(|e| ApiError::InternalError(format!("Pedido ilegível: {e}")))?;

        changes.validate().map_err(ApiError::from)?;

        if let Err(reason) = changes.validate_closed_lists() {
            return Err(ApiError::ValidationError(
                vec![("changes".to_string(), vec![reason])]
                    .into_iter()
                    .collect(),
            ));
        }

        EditRequestRepository::apply(&mut conn, &request, &changes, identity.id)?;

        OrganizationRepository::find_by_id(&mut conn, request.organization_id)
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// POST /admin/edit-requests/{id}/reject
pub async fn reject(
    identity: AdminIdentity,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let request_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let request = web::block(move || {
        EditRequestRepository::set_status(
            &mut conn,
            request_id,
            EditRequestStatus::REJECTED,
            identity.id,
        )
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    Ok(HttpResponse::Ok().json(EditRequestView::from(&request)))
}
