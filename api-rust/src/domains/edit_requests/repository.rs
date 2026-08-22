use chrono::Utc;
use diesel::prelude::*;

use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::edit_request::{EditRequest, EditRequestStatus, NewEditRequest, OrganizationChanges};
use db_types::organization::Organization;
use db_types::schema::{edit_requests, organizations};

pub struct EditRequestRepository;

impl EditRequestRepository {
    pub fn create(
        conn: &mut DbConnection,
        new_request: &NewEditRequest,
    ) -> Result<EditRequest, ApiError> {
        diesel::insert_into(edit_requests::table)
            .values(new_request)
            .get_result(conn)
            .map_err(ApiError::from)
    }

    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<EditRequest, ApiError> {
        edit_requests::table
            .find(id)
            .first::<EditRequest>(conn)
            .map_err(ApiError::from)
    }

    /// Fila de pedidos. `status` em None traz todos.
    pub fn find_all_with_status(
        conn: &mut DbConnection,
        status: Option<&str>,
    ) -> Result<Vec<EditRequest>, ApiError> {
        let mut query = edit_requests::table.into_boxed();

        if let Some(state) = status {
            query = query.filter(edit_requests::status.eq(state.to_string()));
        }

        // Mais antigos primeiro: quem pediu antes espera menos, e o id
        // desempata para que a fila tenha sempre a mesma ordem.
        query
            .order(edit_requests::created_at.asc())
            .then_order_by(edit_requests::id.asc())
            .load::<EditRequest>(conn)
            .map_err(ApiError::from)
    }

    pub fn set_status(
        conn: &mut DbConnection,
        id: i32,
        status: &str,
        reviewed_by: i32,
    ) -> Result<EditRequest, ApiError> {
        diesel::update(edit_requests::table.find(id))
            .set((
                edit_requests::status.eq(status.to_string()),
                edit_requests::reviewed_at.eq(Some(Utc::now().naive_utc())),
                edit_requests::reviewed_by.eq(Some(reviewed_by)),
            ))
            .get_result(conn)
            .map_err(ApiError::from)
    }

    /// Aplica as mudanças na organização. Campo ausente fica como estava.
    pub fn apply_changes(
        conn: &mut DbConnection,
        organization_id: i32,
        changes: &OrganizationChanges,
    ) -> Result<Organization, ApiError> {
        diesel::update(organizations::table.find(organization_id))
            .set(changes)
            .get_result(conn)
            .map_err(ApiError::from)
    }

    /// Aplica o pedido e o marca como aplicado, numa transação só: pedido
    /// marcado sem a mudança correspondente seria pior que não ter aplicado.
    pub fn apply(
        conn: &mut DbConnection,
        request: &EditRequest,
        changes: &OrganizationChanges,
        reviewed_by: i32,
    ) -> Result<Organization, ApiError> {
        conn.transaction::<_, ApiError, _>(|conn| {
            let organization = Self::apply_changes(conn, request.organization_id, changes)?;

            Self::set_status(conn, request.id, EditRequestStatus::APPLIED, reviewed_by)?;

            Ok(organization)
        })
    }
}
