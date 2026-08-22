//! Migrações aplicadas na subida da API.
//!
//! Ficam embutidas no binário, e não copiadas soltas para a imagem: assim o
//! executável e o esquema que ele espera viajam juntos, e não existe um deploy
//! em que o código é novo e o banco é velho porque alguém esqueceu de rodar um
//! comando depois.
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Aplica o que estiver pendente.
///
/// Roda antes de o servidor aceitar conexão: subir atendendo requisição com o
/// banco a meio caminho seria pior do que não subir.
///
/// Usa conexão própria, fora do pool: é uma operação única, de inicialização, e
/// não deve consumir uma conexão que o pool vai precisar depois.
pub fn run(database_url: &str) -> Result<(), String> {
    let mut conn = PgConnection::establish(database_url)
        .map_err(|e| format!("não consegui conectar no banco para migrar: {e}"))?;

    let aplicadas = conn
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("falha ao aplicar migrações: {e}"))?;

    if aplicadas.is_empty() {
        log::info!("Banco já está atualizado");
    } else {
        for migracao in &aplicadas {
            log::info!("Migração aplicada: {migracao}");
        }
    }

    Ok(())
}
