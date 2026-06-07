use serde::Serialize;
use crate::models::image::Image;

#[derive(Debug, Clone, Serialize)]
pub struct ImageView {
    pub id: i32,
    pub url: String,
}

impl ImageView {
    pub fn render(image: &Image, base_url: &str) -> Self {
        ImageView {
            id: image.id,
            url: format!("{}/uploads/{}", base_url, image.path),
        }
    }
    
    pub fn render_many(images: &[Image], base_url: &str) -> Vec<Self> {
        images.iter().map(|img| Self::render(img, base_url)).collect()
    }
}
