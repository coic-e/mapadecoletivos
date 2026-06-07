use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}
