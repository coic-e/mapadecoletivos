/// Business logic for organizations domain
/// All functions here are pure business logic without HTTP concerns
use diesel::Connection;

use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::image::NewImage;
use db_types::organization::{NewOrganization, Organization};

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

/// Get an organization by ID with its images
pub fn get_organization_by_id(
    conn: &mut DbConnection,
    id: i32,
) -> Result<(Organization, Vec<db_types::image::Image>), ApiError> {
    OrganizationRepository::find_by_id(conn, id)
}

/// Get all organizations with pagination
pub fn get_all_organizations(
    conn: &mut DbConnection,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<(Organization, Vec<db_types::image::Image>)>, ApiError> {
    OrganizationRepository::find_all(conn, limit, offset)
}
