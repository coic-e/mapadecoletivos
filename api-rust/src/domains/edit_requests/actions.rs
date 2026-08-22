//! Regras dos pedidos de correção, sem nada de HTTP.
//!
//! O que mora aqui é o que precisa valer independentemente de por onde a
//! chamada entrou: que pedido vazio não ocupa a fila, que lista fechada
//! continua fechada, que só cadastro público aceita sugestão, e que aplicar é
//! tudo-ou-nada. Antes isto vivia dentro dos `web::block` das rotas, onde
//! nenhum teste alcançava sem subir um servidor.
use validator::Validate;

use crate::db::DbConnection;
use crate::domains::organizations::repository::OrganizationRepository;
use crate::errors::ApiError;
use db_types::edit_request::{EditRequest, EditRequestStatus, NewEditRequest, OrganizationChanges};
use db_types::image::Image;
use db_types::organization::Organization;

use super::auth::ReviewEditRequests;
use super::repository::EditRequestRepository;

/// Um erro de validação sobre um campo só, no formato que a API devolve.
fn invalido(campo: &str, motivo: String) -> ApiError {
    ApiError::ValidationError(
        vec![(campo.to_string(), vec![motivo])]
            .into_iter()
            .collect(),
    )
}

/// Confere o que um pedido propõe.
///
/// Roda duas vezes na vida de um pedido — quando entra e quando é aplicado — e
/// é de propósito: o pedido pode ficar na fila enquanto as regras mudam, e é
/// na aplicação que ele vira dado público.
pub fn validate_changes(changes: &OrganizationChanges) -> Result<(), ApiError> {
    if changes.is_empty() {
        return Err(invalido(
            "changes",
            "Informe ao menos um campo para corrigir".to_string(),
        ));
    }

    changes.validate().map_err(ApiError::from)?;

    if let Err(motivo) = changes.validate_closed_lists() {
        return Err(invalido("changes", motivo));
    }

    Ok(())
}

/// Enfileira um pedido para um cadastro público.
///
/// Aceita id ou slug porque o site passou a usar slug e links antigos com id
/// continuam por aí.
pub fn submit(
    conn: &mut DbConnection,
    identifier: &str,
    changes: &OrganizationChanges,
    message: Option<String>,
    requester_email: Option<String>,
) -> Result<EditRequest, ApiError> {
    // Só cadastro público aceita sugestão: pendente e rejeitado não existem
    // para quem está de fora, e aceitar pedido para eles confirmaria que
    // existem.
    let (organization, _) = match identifier.parse::<i32>() {
        Ok(id) => OrganizationRepository::find_approved_by_id(conn, id),
        Err(_) => OrganizationRepository::find_approved_by_slug(conn, identifier),
    }?;

    let changes = serde_json::to_value(changes)
        .map_err(|e| ApiError::InternalError(format!("Falha ao serializar pedido: {e}")))?;

    EditRequestRepository::create(
        conn,
        &NewEditRequest {
            organization_id: organization.id,
            changes,
            message,
            requester_email,
        },
    )
}

/// A fila. `status` em None traz todos os estados.
pub fn list(
    conn: &mut DbConnection,
    _w: ReviewEditRequests,
    status: Option<String>,
) -> Result<Vec<EditRequest>, ApiError> {
    EditRequestRepository::find_all_with_status(conn, status.as_deref())
}

/// Aplica o pedido no cadastro e o fecha, numa transação só.
pub fn apply(
    conn: &mut DbConnection,
    w: ReviewEditRequests,
    request_id: i32,
) -> Result<(Organization, Vec<Image>), ApiError> {
    let request = EditRequestRepository::find_by_id(conn, request_id)?;

    if request.status != EditRequestStatus::PENDING {
        return Err(invalido(
            "status",
            "Este pedido já foi revisado".to_string(),
        ));
    }

    let changes: OrganizationChanges = serde_json::from_value(request.changes.clone())
        .map_err(|e| ApiError::InternalError(format!("Pedido ilegível: {e}")))?;

    // Revalidado agora, e não só na entrada: entre uma coisa e outra as listas
    // fechadas podem ter mudado, e é aqui que ele vira dado público.
    validate_changes(&changes)?;

    EditRequestRepository::apply(conn, &request, &changes, w.admin_id())?;

    OrganizationRepository::find_by_id(conn, w.organizations(), request.organization_id)
}

/// Recusa o pedido. O cadastro fica como estava.
pub fn reject(
    conn: &mut DbConnection,
    w: ReviewEditRequests,
    request_id: i32,
) -> Result<EditRequest, ApiError> {
    EditRequestRepository::set_status(conn, request_id, EditRequestStatus::REJECTED, w.admin_id())
}
