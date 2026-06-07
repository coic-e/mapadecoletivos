use crate::models::organization::Organization;
use crate::models::image::Image;
use crate::views::image_view::ImageView;
use bigdecimal::BigDecimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OrganizationView {
    pub id: i32,
    pub name: String,
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
    #[serde(rename = "type")]
    pub type_: String,
    pub city: String,
    pub uf: String,
    pub email: String,
    pub social: String,
    pub about: String,
    pub images: Vec<ImageView>,
}

impl OrganizationView {
    pub fn render(organization: &Organization, images: &[Image], base_url: &str) -> Self {
        OrganizationView {
            id: organization.id,
            name: organization.name.clone(),
            latitude: organization.latitude.clone(),
            longitude: organization.longitude.clone(),
            type_: organization.type_.clone(),
            city: organization.city.clone(),
            uf: organization.uf.clone(),
            email: organization.email.clone(),
            social: organization.social.clone(),
            about: organization.about.clone(),
            images: ImageView::render_many(images, base_url),
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
