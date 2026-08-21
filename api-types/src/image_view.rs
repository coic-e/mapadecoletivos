use db_types::image::Image;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ImageView {
    pub id: i32,
    pub url: String,
}

impl ImageView {
    /// `base_url` é a base pública do bucket — em produção, provavelmente um
    /// CDN na frente dele. O banco guarda só a chave do objeto, então trocar de
    /// provedor ou pôr um CDN na frente não exige migração de dados.
    pub fn render(image: &Image, base_url: &str) -> Self {
        ImageView {
            id: image.id,
            url: format!("{}/{}", base_url.trim_end_matches('/'), image.path),
        }
    }

    pub fn render_many(images: &[Image], base_url: &str) -> Vec<Self> {
        images
            .iter()
            .map(|img| Self::render(img, base_url))
            .collect()
    }
}
