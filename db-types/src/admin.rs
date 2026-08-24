use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;
use validator::Validate;

use crate::schema::admins;

/// Moderador. Só existe pelo binário create_admin — não há cadastro público.
/// Sem `Serialize`: o hash da senha não depende de ninguém lembrar de um
/// `skip_serializing`. Quem sai da API é `AdminView`.
#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = admins)]
pub struct Admin {
    pub id: i32,
    pub name: String,
    pub email: String,
    /// Hash argon2. Não há caminho daqui para uma resposta: veja AdminView.
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Validate, Deserialize)]
#[diesel(table_name = admins)]
pub struct NewAdmin {
    #[validate(length(min = 1, message = "Nome é obrigatório"))]
    pub name: String,

    #[validate(email(message = "E-mail inválido"))]
    pub email: String,

    pub password_hash: String,
}
