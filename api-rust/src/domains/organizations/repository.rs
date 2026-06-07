use diesel::prelude::*;

use crate::db::DbConnection;
use crate::errors::ApiError;
use crate::models::image::{Image, NewImage};
use crate::models::organization::{NewOrganization, Organization};
use crate::schema::{images, organizations};

pub struct OrganizationRepository;

impl OrganizationRepository {
    pub fn create(
        conn: &mut DbConnection,
        new_organization: &NewOrganization,
    ) -> Result<Organization, ApiError> {
        diesel::insert_into(organizations::table)
            .values(new_organization)
            .get_result(conn)
            .map_err(ApiError::from)
    }

    pub fn find_by_id(
        conn: &mut DbConnection,
        organization_id: i32,
    ) -> Result<(Organization, Vec<Image>), ApiError> {
        let organization = organizations::table
            .find(organization_id)
            .first::<Organization>(conn)?;

        let imgs = Image::belonging_to(&organization).load::<Image>(conn)?;

        Ok((organization, imgs))
    }

    pub fn find_all(
        conn: &mut DbConnection,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(Organization, Vec<Image>)>, ApiError> {
        let mut query = organizations::table.into_boxed();

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        let organizations_list = query.load::<Organization>(conn)?;

        let images_list = Image::belonging_to(&organizations_list)
            .load::<Image>(conn)?
            .grouped_by(&organizations_list);

        let result = organizations_list.into_iter().zip(images_list).collect();

        Ok(result)
    }
}

pub struct ImageRepository;

impl ImageRepository {
    pub fn create_many(
        conn: &mut DbConnection,
        new_images: &[NewImage],
    ) -> Result<Vec<Image>, ApiError> {
        diesel::insert_into(images::table)
            .values(new_images)
            .get_results(conn)
            .map_err(ApiError::from)
    }
}
