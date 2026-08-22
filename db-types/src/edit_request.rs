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

    /// Mesma regra do cadastro: sem isto, a correção seria a porta dos fundos
    /// para um `javascript:` no lugar do site.
    #[validate(
        url(message = "Site inválido"),
        custom(
            function = "crate::organization::validate_website",
            message = "O site precisa começar com http:// ou https://"
        ),
        length(max = 200)
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn changes_from(json: serde_json::Value) -> OrganizationChanges {
        serde_json::from_value(json).expect("o pedido deveria ser legível")
    }

    #[test]
    fn an_untouched_request_is_empty() {
        assert!(OrganizationChanges::default().is_empty());
        assert!(changes_from(serde_json::json!({})).is_empty());
    }

    #[test]
    fn every_field_counts_as_a_change() {
        // Se um campo novo entrar na struct e ninguém lembrar do is_empty, o
        // pedido que só mexe nele passaria por "vazio" e seria recusado.
        for campo in [
            serde_json::json!({ "name": "Bunker" }),
            serde_json::json!({ "about": "um texto qualquer aqui" }),
            serde_json::json!({ "email": "a@b.com" }),
            serde_json::json!({ "city": "Porto Alegre" }),
            serde_json::json!({ "uf": "RS" }),
            serde_json::json!({ "address": "Rua X, 1" }),
            serde_json::json!({ "instagram": "@x" }),
            serde_json::json!({ "soundcloud": "x" }),
            serde_json::json!({ "bandcamp": "x" }),
            serde_json::json!({ "youtube": "x" }),
            serde_json::json!({ "spotify": "x" }),
            serde_json::json!({ "website": "https://x.com" }),
            serde_json::json!({ "genres": ["Techno"] }),
            serde_json::json!({ "frequency": "Mensal" }),
            serde_json::json!({ "is_active": false }),
        ] {
            assert!(
                !changes_from(campo.clone()).is_empty(),
                "{campo} deveria contar como mudança"
            );
        }
    }

    #[test]
    fn clearing_a_link_is_an_empty_string_not_an_omission() {
        // Campo ausente vira None e o AsChangeset não mexe na coluna. Quem
        // quer apagar um link manda string vazia.
        let omitido = changes_from(serde_json::json!({ "name": "Bunker" }));
        assert!(omitido.instagram.is_none());

        let apagado = changes_from(serde_json::json!({ "instagram": "" }));
        assert_eq!(apagado.instagram.as_deref(), Some(""));
        assert!(!apagado.is_empty());
    }

    #[test]
    fn fields_outside_the_struct_are_dropped_not_applied() {
        // Isto é o que impede um pedido de correção de se auto-aprovar ou de
        // sequestrar a URL de outro cadastro: status, slug e id não existem
        // aqui, então o serde os descarta e sobra um pedido vazio.
        let changes = changes_from(serde_json::json!({
            "status": "approved",
            "slug": "cadastro-famoso",
            "id": 1,
            "reviewed_by": 1,
            "created_at": "2026-01-01T00:00:00",
        }));

        assert!(
            changes.is_empty(),
            "nada disso deveria virar mudança aplicável"
        );
    }

    #[test]
    fn the_closed_lists_stay_closed_on_the_edit_path() {
        // Sem isto, a correção viraria a porta dos fundos para gênero e
        // periodicidade que o cadastro recusa.
        let invalido = changes_from(serde_json::json!({ "genres": ["Sertanejo"] }));
        assert!(invalido.validate_closed_lists().is_err());

        let vazio = changes_from(serde_json::json!({ "genres": [] }));
        assert!(vazio.validate_closed_lists().is_err());

        let frequencia = changes_from(serde_json::json!({ "frequency": "Quando dá" }));
        assert!(frequencia.validate_closed_lists().is_err());

        let ok = changes_from(serde_json::json!({
            "genres": ["Techno", "Acid"],
            "frequency": "Mensal",
        }));
        assert!(ok.validate_closed_lists().is_ok());
    }

    #[test]
    fn a_request_that_touches_no_list_passes_the_list_check() {
        assert!(changes_from(serde_json::json!({ "city": "Recife" }))
            .validate_closed_lists()
            .is_ok());
    }

    #[test]
    fn field_lengths_and_formats_are_validated() {
        let nome_longo = changes_from(serde_json::json!({ "name": "a".repeat(121) }));
        assert!(nome_longo.validate().is_err());

        let nome_vazio = changes_from(serde_json::json!({ "name": "" }));
        assert!(nome_vazio.validate().is_err());

        let email = changes_from(serde_json::json!({ "email": "nao-e-email" }));
        assert!(email.validate().is_err());

        let uf = changes_from(serde_json::json!({ "uf": "República" }));
        assert!(uf.validate().is_err());

        let sobre_curto = changes_from(serde_json::json!({ "about": "curto" }));
        assert!(sobre_curto.validate().is_err());

        for hostil in [
            "javascript:alert(1)",
            "data:text/html,<script>",
            "coletivo.com.br",
        ] {
            let site = changes_from(serde_json::json!({ "website": hostil }));

            assert!(
                site.validate().is_err(),
                "{hostil:?} não deveria entrar como site pela correção"
            );
        }

        let ok = changes_from(serde_json::json!({
            "name": "Bunker 034",
            "email": "contato@exemplo.com",
            "uf": "RS",
            "about": "Uma descrição com tamanho suficiente para passar",
            "website": "https://coletivo.com.br",
        }));
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn survives_the_round_trip_through_the_changes_column() {
        // O pedido é gravado como jsonb e relido na hora de aplicar; o que
        // sobreviver a essa viagem é o que vira dado público.
        let original = changes_from(serde_json::json!({
            "name": "Bunker 034",
            "genres": ["Techno"],
            "is_active": false,
        }));

        let gravado = serde_json::to_value(&original).unwrap();
        let relido: OrganizationChanges = serde_json::from_value(gravado.clone()).unwrap();

        assert_eq!(relido.name.as_deref(), Some("Bunker 034"));
        assert_eq!(relido.genres.as_deref(), Some(&["Techno".to_string()][..]));
        assert_eq!(relido.is_active, Some(false));
        assert!(
            gravado.get("city").is_none(),
            "campo não mexido não deveria aparecer no jsonb: {gravado}"
        );
    }

    #[test]
    fn knows_its_own_statuses() {
        assert!(EditRequestStatus::is_valid(EditRequestStatus::PENDING));
        assert!(EditRequestStatus::is_valid(EditRequestStatus::APPLIED));
        assert!(EditRequestStatus::is_valid(EditRequestStatus::REJECTED));

        assert!(!EditRequestStatus::is_valid("approved"));
        assert!(!EditRequestStatus::is_valid("PENDING"));
        assert!(!EditRequestStatus::is_valid(""));
    }
}
