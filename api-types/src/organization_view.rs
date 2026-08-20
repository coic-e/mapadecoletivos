use crate::image_view::ImageView;
use bigdecimal::ToPrimitive;
use chrono::NaiveDateTime;
use db_types::image::Image;
use db_types::organization::Organization;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OrganizationView {
    pub id: i32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "type")]
    pub type_: String,
    pub city: String,
    pub uf: String,
    pub email: String,
    pub about: String,
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
    pub images: Vec<ImageView>,
    /// Sempre "approved" nas rotas públicas; a fila de moderação usa os outros.
    pub status: String,
    pub created_at: NaiveDateTime,
    pub rejection_reason: Option<String>,
}

impl OrganizationView {
    pub fn render(organization: &Organization, images: &[Image], base_url: &str) -> Self {
        OrganizationView {
            id: organization.id,
            name: organization.name.clone(),
            latitude: organization.latitude.to_f64().unwrap_or_default(),
            longitude: organization.longitude.to_f64().unwrap_or_default(),
            type_: organization.type_.clone(),
            city: organization.city.clone(),
            uf: organization.uf.clone(),
            email: organization.email.clone(),
            about: organization.about.clone(),
            genres: organization.genres.clone(),
            address: organization.address.clone(),
            instagram: organization.instagram.clone(),
            soundcloud: organization.soundcloud.clone(),
            bandcamp: organization.bandcamp.clone(),
            youtube: organization.youtube.clone(),
            spotify: organization.spotify.clone(),
            website: organization.website.clone(),
            is_active: organization.is_active,
            frequency: organization.frequency.clone(),
            images: ImageView::render_many(images, base_url),
            status: organization.status.clone(),
            created_at: organization.created_at,
            rejection_reason: organization.rejection_reason.clone(),
        }
    }

    pub fn render_many(
        organizations_with_images: Vec<(Organization, Vec<Image>)>,
        base_url: &str,
    ) -> Vec<Self> {
        organizations_with_images
            .iter()
            .map(|(organization, images)| Self::render(organization, images, base_url))
            .collect()
    }
}
