use db_types::admin::Admin;
use serde::Serialize;

/// Admin como o painel enxerga. Existe para o hash da senha nunca sair da API.
#[derive(Debug, Clone, Serialize)]
pub struct AdminView {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl From<&Admin> for AdminView {
    fn from(admin: &Admin) -> Self {
        AdminView {
            id: admin.id,
            name: admin.name.clone(),
            email: admin.email.clone(),
        }
    }
}
