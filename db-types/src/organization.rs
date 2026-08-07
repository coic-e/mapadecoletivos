use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::schema::organizations;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
    #[serde(rename = "type")]
    pub type_: String,
    pub city: String,
    pub uf: String,
    pub email: String,
    pub social: String,
    pub about: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Validate, Deserialize)]
#[diesel(table_name = organizations)]
pub struct NewOrganization {
    #[validate(length(min = 1, message = "Nome é obrigatório"))]
    pub name: String,

    pub latitude: BigDecimal,

    pub longitude: BigDecimal,

    #[validate(length(min = 1, message = "Tipo é obrigatório"))]
    #[serde(rename = "type")]
    pub type_: String,

    #[validate(length(min = 1, message = "Cidade é obrigatório"))]
    pub city: String,

    #[validate(length(min = 1, message = "UF é obrigatório"))]
    pub uf: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 1, message = "Social é obrigatório"))]
    pub social: String,

    #[validate(length(min = 1, max = 300, message = "Sobre é obrigatório"))]
    pub about: String,
}
