use chrono::NaiveDateTime;
use db_types::edit_request::EditRequest;
use serde::Serialize;
use serde_json::Value;

/// Pedido de correção como o painel enxerga.
#[derive(Debug, Clone, Serialize)]
pub struct EditRequestView {
    pub id: i32,
    pub organization_id: i32,
    pub changes: Value,
    pub message: Option<String>,
    pub requester_email: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
}

impl From<&EditRequest> for EditRequestView {
    fn from(request: &EditRequest) -> Self {
        EditRequestView {
            id: request.id,
            organization_id: request.organization_id,
            changes: request.changes.clone(),
            message: request.message.clone(),
            requester_email: request.requester_email.clone(),
            status: request.status.clone(),
            created_at: request.created_at,
        }
    }
}
