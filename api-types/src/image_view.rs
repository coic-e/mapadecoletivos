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

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Image {
        Image {
            id: 3,
            path: "9f8e7d6c.jpg".to_string(),
            organization_id: 7,
            position: 0,
        }
    }

    #[test]
    fn joins_the_bucket_base_with_the_object_key() {
        assert_eq!(
            ImageView::render(&image(), "https://cdn.exemplo.com").url,
            "https://cdn.exemplo.com/9f8e7d6c.jpg"
        );
    }

    #[test]
    fn a_trailing_slash_in_the_base_does_not_double_up() {
        // A base vem de configuração escrita à mão; a barra sobrando é o erro
        // mais fácil de cometer e quebraria toda imagem do site.
        for base in ["https://cdn.exemplo.com/", "https://cdn.exemplo.com///"] {
            assert_eq!(
                ImageView::render(&image(), base).url,
                "https://cdn.exemplo.com/9f8e7d6c.jpg",
                "base {base:?}"
            );
        }
    }

    #[test]
    fn the_database_keeps_the_key_not_the_url() {
        // Trocar de provedor ou pôr um CDN na frente é mudar a base, sem
        // migração de dado.
        let local = ImageView::render(&image(), "http://localhost:9000/rave-map");
        let producao = ImageView::render(&image(), "https://cdn.exemplo.com");

        assert_eq!(local.id, producao.id);
        assert!(local.url.ends_with("/9f8e7d6c.jpg"));
        assert!(producao.url.ends_with("/9f8e7d6c.jpg"));
    }

    #[test]
    fn renders_a_whole_gallery() {
        let images = vec![
            image(),
            Image {
                id: 4,
                path: "b.png".to_string(),
                ..image()
            },
        ];

        let views = ImageView::render_many(&images, "https://cdn.exemplo.com");

        assert_eq!(views.len(), 2);
        assert_eq!(views[1].url, "https://cdn.exemplo.com/b.png");
    }
}
