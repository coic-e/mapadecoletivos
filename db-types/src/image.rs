use diesel::prelude::*;
use serde::Deserialize;

use crate::organization::Organization;
use crate::schema::images;

/// Sem `Serialize`: o banco guarda a chave do objeto, e o que o navegador
/// precisa é a URL montada — trabalho de `ImageView`.
#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Organization, foreign_key = organization_id))]
#[diesel(table_name = images)]
pub struct Image {
    pub id: i32,
    pub path: String,
    pub organization_id: i32,
    /// Ordem de exibição. A de menor posição é a capa.
    pub position: i32,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = images)]
pub struct NewImage {
    pub path: String,
    pub organization_id: i32,
    pub position: i32,
}
