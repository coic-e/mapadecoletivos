use diesel::prelude::*;

use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::admin::{Admin, NewAdmin};
use db_types::schema::admins;

pub struct AdminRepository;

impl AdminRepository {
    /// Devolve Option em vez de erro: "não existe" e "senha errada" precisam
    /// virar a mesma resposta no login, para não revelar quais e-mails existem.
    pub fn find_by_email(conn: &mut DbConnection, email: &str) -> Result<Option<Admin>, ApiError> {
        admins::table
            .filter(admins::email.eq(email))
            .first::<Admin>(conn)
            .optional()
            .map_err(ApiError::from)
    }

    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<Admin, ApiError> {
        admins::table
            .find(id)
            .first::<Admin>(conn)
            .map_err(ApiError::from)
    }

    /// Quantos moderadores existem. A semente do primeiro depende disto.
    pub fn count(conn: &mut DbConnection) -> Result<i64, ApiError> {
        admins::table.count().get_result(conn).map_err(ApiError::from)
    }

    pub fn create(conn: &mut DbConnection, new_admin: &NewAdmin) -> Result<Admin, ApiError> {
        diesel::insert_into(admins::table)
            .values(new_admin)
            .get_result(conn)
            .map_err(ApiError::from)
    }
}
