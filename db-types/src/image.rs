use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::organization::Organization;
use crate::schema::images;

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Organization, foreign_key = organization_id))]
#[diesel(table_name = images)]
pub struct Image {
    pub id: i32,
    pub path: String,
    pub organization_id: i32,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = images)]
pub struct NewImage {
    pub path: String,
    pub organization_id: i32,
}
