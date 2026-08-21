use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::organization::{Organization, FREQUENCIES};
use crate::schema::{edit_requests, organizations};

/// Pedido de correção vindo de fora do painel.
///
/// Editar de verdade é só do admin; isto é a porta de entrada de quem enxerga
/// o erro e não tem acesso. O pedido entra numa fila e um moderador aplica ou
/// recusa — o mesmo caminho da moderação de cadastro novo.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(Organization, foreign_key = organization_id))]
#[diesel(table_name = edit_requests)]
pub struct EditRequest {
    pub id: i32,
    pub organization_id: i32,
    /// Só os campos que o pedido quer mudar, no formato de OrganizationChanges.
    pub changes: Value,
    pub message: Option<String>,
    pub requester_email: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub reviewed_at: Option<NaiveDateTime>,
    pub reviewed_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = edit_requests)]
pub struct NewEditRequest {
    pub organization_id: i32,
    pub changes: Value,
    pub message: Option<String>,
    pub requester_email: Option<String>,
}

pub struct EditRequestStatus;

impl EditRequestStatus {
    pub const PENDING: &'static str = "pending";
    pub const APPLIED: &'static str = "applied";
    pub const REJECTED: &'static str = "rejected";

    pub fn is_valid(value: &str) -> bool {
        matches!(value, Self::PENDING | Self::APPLIED | Self::REJECTED)
    }
}

/// O que dá para mudar numa organização depois de criada.
///
/// Serve para dois caminhos: o corpo do PATCH do admin e o conteúdo `changes`
/// de um pedido. Ter um tipo só garante que o que é aceito num caminho é
/// exatamente o que é aceito no outro.
///
/// Campo ausente significa "não mexer" — é assim que o AsChangeset do Diesel
/// trata None. Trocar um link por vazio, portanto, se faz mandando string
/// vazia, não omitindo o campo.
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize, Validate)]
#[diesel(table_name = organizations)]
pub struct OrganizationChanges {
    #[validate(length(min = 1, max = 120, message = "Nome de 1 a 120 caracteres"))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,

    #[validate(length(min = 10, max = 1200, message = "Sobre de 10 a 1200 caracteres"))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub about: Option<String>,

    #[validate(email(message = "Email inválido"), length(max = 254))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub email: Option<String>,

    #[validate(length(min = 1, max = 120, message = "Cidade de 1 a 120 caracteres"))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub city: Option<String>,

    #[validate(length(min = 2, max = 2, message = "UF deve ter 2 letras"))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub uf: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub address: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub instagram: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub soundcloud: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bandcamp: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub youtube: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub spotify: Option<String>,

    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub website: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub genres: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub frequency: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_active: Option<bool>,
}

impl OrganizationChanges {
    /// Nada preenchido é pedido vazio, e pedido vazio não deveria ocupar a fila.
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.about.is_none()
            && self.email.is_none()
            && self.city.is_none()
            && self.uf.is_none()
            && self.address.is_none()
            && self.instagram.is_none()
            && self.soundcloud.is_none()
            && self.bandcamp.is_none()
            && self.youtube.is_none()
            && self.spotify.is_none()
            && self.website.is_none()
            && self.genres.is_none()
            && self.frequency.is_none()
            && self.is_active.is_none()
    }

    /// As listas fechadas continuam fechadas na edição: sem isso, o caminho de
    /// correção viraria a porta dos fundos para gênero e periodicidade
    /// inventados, que o cadastro recusa.
    pub fn validate_closed_lists(&self) -> Result<(), String> {
        if let Some(genres) = &self.genres {
            if genres.is_empty() {
                return Err("Informe ao menos um gênero".to_string());
            }

            if let Some(invalid) = genres
                .iter()
                .find(|genre| !crate::organization::MUSIC_GENRES.contains(&genre.as_str()))
            {
                return Err(format!("Gênero inválido: {invalid}"));
            }
        }

        if let Some(frequency) = &self.frequency {
            if !FREQUENCIES.contains(&frequency.as_str()) {
                return Err(format!("Periodicidade inválida: {frequency}"));
            }
        }

        Ok(())
    }
}
