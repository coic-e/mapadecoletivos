//! Criação do primeiro moderador a partir do ambiente.
//!
//! Existe porque não há rota pública para criar admin e nem todo lugar de
//! deploy dá console para rodar o binário `create_admin`.
//!
//! É semente de primeira subida, não fonte permanente de senha: só age quando
//! a tabela está vazia. Se aplicasse a senha a cada boot, quem tivesse acesso
//! ao painel assumiria a conta editando uma variável e reiniciando o serviço —
//! e a senha do moderador ficaria permanentemente exposta em `docker inspect` e
//! em `/proc/<pid>/environ`.
use diesel::prelude::*;

use crate::auth::hash_password;
use crate::db::DbConnection;
use crate::domains::admins::repository::AdminRepository;
use db_types::admin::NewAdmin;
use db_types::schema::admins;

/// Mesmo piso do binário create_admin: as duas portas precisam exigir o mesmo,
/// senão a mais frouxa vira o caminho preferido.
const MIN_PASSWORD_LEN: usize = 12;

pub struct AdminSeed {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl AdminSeed {
    /// Lê a semente do ambiente. Ausência das variáveis não é erro: é o caso
    /// normal depois da primeira subida.
    pub fn from_env() -> Option<Self> {
        let email = std::env::var("ADMIN_EMAIL").ok()?;
        let password = std::env::var("ADMIN_PASSWORD").ok()?;

        Some(AdminSeed {
            name: std::env::var("ADMIN_NAME").unwrap_or_else(|_| "Moderação".to_string()),
            email: email.trim().to_lowercase(),
            password,
        })
    }
}

/// Cria o primeiro moderador, se ainda não houver nenhum.
///
/// Devolve erro só quando a semente existe e está inválida: nesse caso é melhor
/// falhar alto do que subir sem ninguém conseguir moderar.
pub fn seed_first_admin(conn: &mut DbConnection, seed: Option<AdminSeed>) -> Result<(), String> {
    let existentes: i64 = admins::table
        .count()
        .get_result(conn)
        .map_err(|e| format!("não consegui contar os moderadores: {e}"))?;

    match (existentes, seed) {
        (0, None) => {
            log::warn!(
                "Nenhum moderador cadastrado e ADMIN_EMAIL/ADMIN_PASSWORD não definidos: \
                 o painel de moderação fica inacessível até criar uma conta"
            );

            Ok(())
        }

        (0, Some(seed)) => {
            if seed.password.chars().count() < MIN_PASSWORD_LEN {
                return Err(format!(
                    "ADMIN_PASSWORD precisa de pelo menos {MIN_PASSWORD_LEN} caracteres"
                ));
            }

            let password_hash =
                hash_password(&seed.password).map_err(|e| format!("falha ao gerar hash: {e}"))?;

            let admin = AdminRepository::create(
                conn,
                &NewAdmin {
                    name: seed.name,
                    email: seed.email,
                    password_hash,
                },
            )
            .map_err(|e| format!("falha ao criar o primeiro moderador: {e}"))?;

            log::info!(
                "Primeiro moderador criado: {} <{}>",
                admin.name,
                admin.email
            );
            log::warn!(
                "Remova ADMIN_EMAIL e ADMIN_PASSWORD do ambiente: a conta já existe e a \
                 senha não precisa mais ficar guardada lá"
            );

            Ok(())
        }

        (_, Some(_)) => {
            // A senha no ambiente não vale mais nada aqui, e continuar guardada
            // só aumenta a superfície de vazamento.
            log::warn!(
                "ADMIN_EMAIL/ADMIN_PASSWORD definidos mas já existem moderadores: \
                 ignorados. Remova-os do ambiente"
            );

            Ok(())
        }

        (_, None) => Ok(()),
    }
}
