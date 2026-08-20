use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::schema::organizations;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
    #[serde(rename = "type")]
    pub type_: String,
    pub city: String,
    pub uf: String,
    pub email: String,
    pub about: String,
    pub created_at: NaiveDateTime,
    /// "pending" | "approved" | "rejected". Ver ModerationStatus.
    pub status: String,
    pub rejection_reason: Option<String>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub reviewed_by: Option<i32>,
    pub genres: Vec<String>,
    pub address: Option<String>,
    pub instagram: Option<String>,
    pub soundcloud: Option<String>,
    pub bandcamp: Option<String>,
    pub youtube: Option<String>,
    pub spotify: Option<String>,
    pub website: Option<String>,
    pub is_active: bool,
    pub frequency: Option<String>,
}

/// Estados de moderação. O banco garante o conjunto com um CHECK; aqui os
/// valores viram constantes para não espalhar string solta pelo código.
pub struct ModerationStatus;

impl ModerationStatus {
    pub const PENDING: &'static str = "pending";
    pub const APPROVED: &'static str = "approved";
    pub const REJECTED: &'static str = "rejected";

    pub fn is_valid(value: &str) -> bool {
        matches!(value, Self::PENDING | Self::APPROVED | Self::REJECTED)
    }
}

/// Tipos de coletivo aceitos. Campo fechado: em aberto, ele virava texto livre
/// que o site imprime na página de todo mundo.
/// Espelha TYPE_OPTIONS do formulário em app/src/pages/CreateOrganization.tsx.
/// Mudou lá, muda aqui — o select do navegador é sugestão, não barreira.
pub const ORGANIZATION_TYPES: &[&str] = &[
    "Festa",
    "Festival",
    "Label",
    "Radio",
    "Podcast",
    "Coletivo",
    "Nucleo",
    "Club",
    "Bar",
    "Produtora",
    "outro",
];

/// Gêneros aceitos. Lista fechada porque é ela que sustenta o filtro do mapa:
/// texto livre viraria "techno", "Techno" e "tekno" como três coisas.
pub const MUSIC_GENRES: &[&str] = &[
    "Techno",
    "Hard Techno",
    "Minimal",
    "Acid",
    "House",
    "Deep House",
    "Tech House",
    "Afro House",
    "Disco",
    "Electro",
    "Trance",
    "Psytrance",
    "Drum & Bass",
    "Jungle",
    "Dubstep",
    "Bass Music",
    "Breakbeat",
    "Hardcore",
    "Funk",
    "Ambient",
    "Experimental",
    "Outro",
];

/// Com que frequência o rolê acontece.
pub const FREQUENCIES: &[&str] = &["Semanal", "Quinzenal", "Mensal", "Sazonal", "Pontual"];

fn validate_genres(values: &[String]) -> Result<(), ValidationError> {
    if values.is_empty() {
        return Err(ValidationError::new("genero_obrigatorio"));
    }

    if values.len() > MUSIC_GENRES.len() {
        return Err(ValidationError::new("generos_demais"));
    }

    if values
        .iter()
        .any(|value| !MUSIC_GENRES.contains(&value.as_str()))
    {
        return Err(ValidationError::new("genero_invalido"));
    }

    Ok(())
}

fn validate_frequency(value: &str) -> Result<(), ValidationError> {
    if FREQUENCIES.contains(&value) {
        return Ok(());
    }

    Err(ValidationError::new("frequencia_invalida"))
}

/// Pelo menos um canal de contato precisa existir, senão o cadastro não serve
/// para o que o site existe: achar o rolê.
fn validate_has_link(organization: &NewOrganization) -> Result<(), ValidationError> {
    let has_link = [
        &organization.instagram,
        &organization.soundcloud,
        &organization.bandcamp,
        &organization.youtube,
        &organization.spotify,
        &organization.website,
    ]
    .iter()
    .any(|link| link.as_ref().is_some_and(|value| !value.trim().is_empty()));

    if has_link {
        return Ok(());
    }

    Err(ValidationError::new("link_obrigatorio"))
}

fn validate_organization_type(value: &str) -> Result<(), ValidationError> {
    if ORGANIZATION_TYPES.contains(&value) {
        return Ok(());
    }

    Err(ValidationError::new("tipo_invalido"))
}

/// Latitude/longitude precisam ser coordenadas de verdade. Fora de faixa, o
/// marcador some do mapa ou o front quebra ao desenhar.
fn validate_coordinate(value: &BigDecimal, limit: i32) -> Result<(), ValidationError> {
    let limit = BigDecimal::from(limit);

    if *value >= -limit.clone() && *value <= limit {
        return Ok(());
    }

    Err(ValidationError::new("fora_de_faixa"))
}

fn validate_uf(value: &str) -> Result<(), ValidationError> {
    if value.len() == 2 && value.chars().all(|c| c.is_ascii_alphabetic()) {
        return Ok(());
    }

    Err(ValidationError::new("uf_invalida"))
}

fn validate_latitude(value: &BigDecimal) -> Result<(), ValidationError> {
    validate_coordinate(value, 90)
}

fn validate_longitude(value: &BigDecimal) -> Result<(), ValidationError> {
    validate_coordinate(value, 180)
}

/// Toda string tem teto de tamanho. As colunas são VARCHAR sem limite, então
/// sem isso um cadastro pode gravar megabytes por campo — o banco incha e a
/// listagem pública carrega tudo de volta a cada requisição.
#[derive(Debug, Clone, Insertable, Validate, Deserialize)]
#[diesel(table_name = organizations)]
#[validate(schema(function = "validate_has_link", skip_on_field_errors = false))]
pub struct NewOrganization {
    #[validate(length(
        min = 1,
        max = 120,
        message = "Nome é obrigatório (até 120 caracteres)"
    ))]
    pub name: String,

    #[validate(custom(function = "validate_latitude", message = "Latitude inválida"))]
    pub latitude: BigDecimal,

    #[validate(custom(function = "validate_longitude", message = "Longitude inválida"))]
    pub longitude: BigDecimal,

    #[validate(custom(function = "validate_organization_type", message = "Tipo inválido"))]
    #[serde(rename = "type")]
    pub type_: String,

    #[validate(length(
        min = 1,
        max = 120,
        message = "Cidade é obrigatório (até 120 caracteres)"
    ))]
    pub city: String,

    #[validate(custom(function = "validate_uf", message = "UF deve ter 2 letras"))]
    pub uf: String,

    #[validate(
        email(message = "Email inválido"),
        length(max = 254, message = "Email longo demais")
    )]
    pub email: String,

    #[validate(custom(
        function = "validate_genres",
        message = "Informe ao menos um gênero válido"
    ))]
    pub genres: Vec<String>,

    #[validate(length(max = 200, message = "Endereço longo demais"))]
    pub address: Option<String>,

    #[validate(length(max = 200, message = "Link longo demais"))]
    pub instagram: Option<String>,

    #[validate(length(max = 200, message = "Link longo demais"))]
    pub soundcloud: Option<String>,

    #[validate(length(max = 200, message = "Link longo demais"))]
    pub bandcamp: Option<String>,

    #[validate(length(max = 200, message = "Link longo demais"))]
    pub youtube: Option<String>,

    #[validate(length(max = 200, message = "Link longo demais"))]
    pub spotify: Option<String>,

    #[validate(url(message = "Site inválido"), length(max = 200))]
    pub website: Option<String>,

    pub is_active: bool,

    #[validate(custom(function = "validate_frequency", message = "Periodicidade inválida"))]
    pub frequency: Option<String>,

    #[validate(length(
        min = 10,
        max = 1200,
        message = "Sobre é obrigatório (10 a 1200 caracteres)"
    ))]
    pub about: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn valid() -> NewOrganization {
        NewOrganization {
            name: "Coletivo Teste".to_string(),
            latitude: BigDecimal::from_str("-30.03").unwrap(),
            longitude: BigDecimal::from_str("-51.23").unwrap(),
            type_: "Coletivo".to_string(),
            city: "Porto Alegre".to_string(),
            uf: "RS".to_string(),
            email: "contato@exemplo.com".to_string(),
            about: "Um coletivo de techno em Porto Alegre".to_string(),
            genres: vec!["Techno".to_string()],
            address: None,
            instagram: Some("@coletivo".to_string()),
            soundcloud: None,
            bandcamp: None,
            youtube: None,
            spotify: None,
            website: None,
            is_active: true,
            frequency: Some("Mensal".to_string()),
        }
    }

    #[test]
    fn requires_at_least_one_genre() {
        let mut org = valid();
        org.genres = vec![];

        assert!(org.validate().is_err());
    }

    #[test]
    fn rejects_unknown_genre() {
        let mut org = valid();
        org.genres = vec!["Sertanejo".to_string()];

        assert!(org.validate().is_err());
    }

    #[test]
    fn requires_at_least_one_link() {
        let mut org = valid();
        org.instagram = None;

        assert!(
            org.validate().is_err(),
            "sem nenhum link o cadastro não serve para achar o rolê"
        );

        let mut org = valid();
        org.instagram = None;
        org.bandcamp = Some("https://coletivo.bandcamp.com".to_string());

        assert!(org.validate().is_ok(), "um link basta");
    }

    #[test]
    fn rejects_unknown_frequency() {
        let mut org = valid();
        org.frequency = Some("Quando dá".to_string());

        assert!(org.validate().is_err());
    }

    #[test]
    fn accepts_a_well_formed_organization() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn rejects_oversized_text() {
        let mut org = valid();
        org.name = "a".repeat(121);

        assert!(org.validate().is_err());
    }

    #[test]
    fn rejects_coordinates_off_the_globe() {
        let mut org = valid();
        org.latitude = BigDecimal::from_str("910").unwrap();

        assert!(org.validate().is_err());

        let mut org = valid();
        org.longitude = BigDecimal::from_str("-181").unwrap();

        assert!(org.validate().is_err());
    }

    #[test]
    fn rejects_free_text_in_closed_fields() {
        let mut org = valid();
        org.type_ = "<script>alert(1)</script>".to_string();

        assert!(org.validate().is_err());

        let mut org = valid();
        org.uf = "República".to_string();

        assert!(org.validate().is_err());
    }
}
