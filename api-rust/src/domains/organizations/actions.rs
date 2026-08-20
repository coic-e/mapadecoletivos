/// Business logic for organizations domain
/// All functions here are pure business logic without HTTP concerns
use diesel::Connection;

use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::image::NewImage;
use db_types::organization::{ModerationStatus, NewOrganization, Organization};

use super::repository::{ImageRepository, OrganizationRepository};

/// Create a new organization with images
pub fn create_organization(
    conn: &mut DbConnection,
    new_org: NewOrganization,
    files: Vec<String>,
) -> Result<(Organization, Vec<db_types::image::Image>), ApiError> {
    conn.transaction::<_, ApiError, _>(|conn| {
        // Create organization
        let organization = OrganizationRepository::create(conn, &new_org)?;

        // Prepare images
        let new_images: Vec<NewImage> = files
            .iter()
            .map(|filename| NewImage {
                path: filename.clone(),
                organization_id: organization.id,
            })
            .collect();

        // Create images
        let images = ImageRepository::create_many(conn, &new_images)?;

        Ok((organization, images))
    })
}

/// Busca pública por id: só devolve cadastro aprovado.
pub fn get_organization_by_id(
    conn: &mut DbConnection,
    id: i32,
) -> Result<(Organization, Vec<db_types::image::Image>), ApiError> {
    OrganizationRepository::find_approved_by_id(conn, id)
}

/// Listagem pública paginada: só cadastros aprovados.
pub fn get_all_organizations(
    conn: &mut DbConnection,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<(Organization, Vec<db_types::image::Image>)>, ApiError> {
    OrganizationRepository::find_all_approved(conn, limit, offset)
}

/// Fila de moderação. `status` em None traz todos os estados.
pub fn get_organizations_for_moderation(
    conn: &mut DbConnection,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<(Organization, Vec<db_types::image::Image>)>, ApiError> {
    OrganizationRepository::find_all_with_status(conn, status.as_deref(), limit, offset)
}

/// Aprova ou rejeita um cadastro, registrando quem decidiu.
pub fn review_organization(
    conn: &mut DbConnection,
    id: i32,
    status: &str,
    reviewed_by: i32,
    rejection_reason: Option<String>,
) -> Result<(Organization, Vec<db_types::image::Image>), ApiError> {
    if !ModerationStatus::is_valid(status) {
        return Err(ApiError::ValidationError(
            vec![(
                "status".to_string(),
                vec![format!("Status inválido: {}", status)],
            )]
            .into_iter()
            .collect(),
        ));
    }

    conn.transaction::<_, ApiError, _>(|conn| {
        // Confere que existe antes de atualizar, para responder 404 em vez de
        // um erro genérico de banco.
        OrganizationRepository::find_by_id(conn, id)?;

        OrganizationRepository::set_status(conn, id, status, reviewed_by, rejection_reason)?;

        OrganizationRepository::find_by_id(conn, id)
    })
}
