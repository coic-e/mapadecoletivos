use crate::models::collective::Collective;
use crate::models::image::Image;
use crate::views::image_view::ImageView;
use bigdecimal::BigDecimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CollectiveView {
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

impl CollectiveView {
    pub fn render(collective: &Collective, images: &[Image], base_url: &str) -> Self {
        CollectiveView {
            id: collective.id,
            name: collective.name.clone(),
            latitude: collective.latitude.clone(),
            longitude: collective.longitude.clone(),
            type_: collective.type_.clone(),
            city: collective.city.clone(),
            uf: collective.uf.clone(),
            email: collective.email.clone(),
            social: collective.social.clone(),
            about: collective.about.clone(),
            images: ImageView::render_many(images, base_url),
        }
    }

    pub fn render_many(
        collectives_with_images: Vec<(Collective, Vec<Image>)>,
        base_url: &str,
    ) -> Vec<Self> {
        collectives_with_images
            .iter()
            .map(|(collective, images)| Self::render(collective, images, base_url))
            .collect()
    }
}
