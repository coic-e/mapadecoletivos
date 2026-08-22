use chrono::Utc;
use diesel::prelude::*;

use crate::db::DbConnection;
use crate::errors::ApiError;
use db_types::image::{Image, NewImage};
use db_types::organization::{ModerationStatus, NewOrganization, Organization};
use db_types::schema::{images, organizations};

pub struct OrganizationRepository;

impl OrganizationRepository {
    pub fn create(
        conn: &mut DbConnection,
        new_organization: &NewOrganization,
    ) -> Result<Organization, ApiError> {
        // status não vai no insert: o banco aplica o default 'pending'.
        diesel::insert_into(organizations::table)
            .values(new_organization)
            .get_result(conn)
            .map_err(ApiError::from)
    }

    /// Acha um slug livre a partir do slug base, acrescentando sufixo numérico
    /// enquanto houver colisão: bunker-034, bunker-034-2, bunker-034-3...
    pub fn find_free_slug(conn: &mut DbConnection, base: &str) -> Result<String, ApiError> {
        let mut candidate = base.to_string();
        let mut suffix = 1;

        loop {
            let taken: i64 = organizations::table
                .filter(organizations::slug.eq(&candidate))
                .count()
                .get_result(conn)?;

            if taken == 0 {
                return Ok(candidate);
            }

            suffix += 1;
            candidate = format!("{base}-{suffix}");
        }
    }

    /// Busca pública por slug. Mesma regra do id: só aprovado existe.
    pub fn find_approved_by_slug(
        conn: &mut DbConnection,
        slug: &str,
    ) -> Result<(Organization, Vec<Image>), ApiError> {
        let organization = organizations::table
            .filter(organizations::slug.eq(slug))
            .filter(organizations::status.eq(ModerationStatus::APPROVED))
            .first::<Organization>(conn)?;

        let imgs = Image::belonging_to(&organization)
            .order(images::position.asc())
            .load::<Image>(conn)?;

        Ok((organization, imgs))
    }

    /// Busca pública: um cadastro só existe para o site depois de aprovado.
    /// Pendente ou rejeitado responde 404, e não "existe mas está escondido".
    pub fn find_approved_by_id(
        conn: &mut DbConnection,
        organization_id: i32,
    ) -> Result<(Organization, Vec<Image>), ApiError> {
        let organization = organizations::table
            .find(organization_id)
            .filter(organizations::status.eq(ModerationStatus::APPROVED))
            .first::<Organization>(conn)?;

        let imgs = Image::belonging_to(&organization)
            .order(images::position.asc())
            .load::<Image>(conn)?;

        Ok((organization, imgs))
    }

    /// Listagem pública do mapa: só aprovados.
    pub fn find_all_approved(
        conn: &mut DbConnection,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(Organization, Vec<Image>)>, ApiError> {
        Self::find_all_with_status(conn, Some(ModerationStatus::APPROVED), limit, offset)
    }

    /// Listagem da moderação. `status` em None traz tudo, de qualquer estado.
    pub fn find_all_with_status(
        conn: &mut DbConnection,
        status: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(Organization, Vec<Image>)>, ApiError> {
        let mut query = organizations::table.into_boxed();

        if let Some(state) = status {
            query = query.filter(organizations::status.eq(state.to_string()));
        }

        // Mais antigos primeiro: quem cadastrou antes espera menos.
        query = query.order(organizations::created_at.asc());

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        if let Some(off) = offset {
            query = query.offset(off);
        }

        let organizations_list = query.load::<Organization>(conn)?;

        let images_list = Image::belonging_to(&organizations_list)
            .order(images::position.asc())
            .load::<Image>(conn)?
            .grouped_by(&organizations_list);

        Ok(organizations_list.into_iter().zip(images_list).collect())
    }

    /// Busca da moderação: enxerga qualquer estado.
    pub fn find_by_id(
        conn: &mut DbConnection,
        organization_id: i32,
    ) -> Result<(Organization, Vec<Image>), ApiError> {
        let organization = organizations::table
            .find(organization_id)
            .first::<Organization>(conn)?;

        let imgs = Image::belonging_to(&organization)
            .order(images::position.asc())
            .load::<Image>(conn)?;

        Ok((organization, imgs))
    }

    /// Apaga as linhas de imagem e devolve as chaves dos objetos.
    ///
    /// Só o banco: quem apaga do bucket é a rota, porque remoção lá é
    /// assíncrona e não pode acontecer dentro da transação — se a transação
    /// desfizesse depois, os arquivos já teriam sumido.
    pub fn delete_images(
        conn: &mut DbConnection,
        organization_id: i32,
    ) -> Result<Vec<String>, ApiError> {
        let keys: Vec<String> = images::table
            .filter(images::organization_id.eq(organization_id))
            .select(images::path)
            .load(conn)?;

        diesel::delete(images::table.filter(images::organization_id.eq(organization_id)))
            .execute(conn)?;

        Ok(keys)
    }

    /// Grava a decisão do moderador, junto de quem decidiu e quando.
    pub fn set_status(
        conn: &mut DbConnection,
        organization_id: i32,
        status: &str,
        reviewed_by: i32,
        rejection_reason: Option<String>,
    ) -> Result<Organization, ApiError> {
        diesel::update(organizations::table.find(organization_id))
            .set((
                organizations::status.eq(status.to_string()),
                organizations::reviewed_at.eq(Some(Utc::now().naive_utc())),
                organizations::reviewed_by.eq(Some(reviewed_by)),
                organizations::rejection_reason.eq(rejection_reason),
            ))
            .get_result(conn)
            .map_err(ApiError::from)
    }
}

pub struct ImageRepository;

impl ImageRepository {
    pub fn create_many(
        conn: &mut DbConnection,
        new_images: &[NewImage],
    ) -> Result<Vec<Image>, ApiError> {
        diesel::insert_into(images::table)
            .values(new_images)
            .get_results(conn)
            .map_err(ApiError::from)
    }
}
