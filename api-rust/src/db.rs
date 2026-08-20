use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use std::time::Duration;

use crate::config::AppConfig;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Teto de conexões. Sem ele o r2d2 usa o padrão e, sob carga, a API sozinha
/// pode consumir todos os slots do Postgres.
const MAX_POOL_SIZE: u32 = 10;

/// Quanto esperar por uma conexão livre antes de devolver erro. Sem timeout, o
/// pedido fica pendurado e o worker do actix não volta para a fila.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

pub fn establish_connection_pool(config: &AppConfig) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(config.database_url.clone());

    r2d2::Pool::builder()
        .max_size(MAX_POOL_SIZE)
        .connection_timeout(CONNECTION_TIMEOUT)
        .build(manager)
        // A URL nunca entra na mensagem: ela carrega a senha do banco.
        .unwrap_or_else(|e| panic!("falha ao criar o pool de conexões: {e}"))
}
