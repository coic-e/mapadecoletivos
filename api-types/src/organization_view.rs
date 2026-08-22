use crate::image_view::ImageView;
use bigdecimal::ToPrimitive;
use chrono::NaiveDateTime;
use db_types::image::Image;
use db_types::organization::Organization;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OrganizationView {
    pub id: i32,
    pub slug: String,
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
            slug: organization.slug.clone(),
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

#[cfg(test)]
pub(crate) mod fixtures {
    use bigdecimal::BigDecimal;
    use chrono::NaiveDate;
    use db_types::image::Image;
    use db_types::organization::Organization;
    use std::str::FromStr;

    pub fn organization() -> Organization {
        Organization {
            id: 7,
            name: "Bunker 034".to_string(),
            latitude: BigDecimal::from_str("-30.0346").unwrap(),
            longitude: BigDecimal::from_str("-51.2177").unwrap(),
            type_: "Coletivo".to_string(),
            city: "Porto Alegre".to_string(),
            uf: "RS".to_string(),
            email: "contato@exemplo.com".to_string(),
            about: "Um coletivo de techno em Porto Alegre".to_string(),
            created_at: NaiveDate::from_ymd_opt(2026, 3, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
            status: "approved".to_string(),
            rejection_reason: None,
            reviewed_at: None,
            reviewed_by: None,
            genres: vec!["Techno".to_string(), "Acid".to_string()],
            address: Some("Rua Qualquer, 34".to_string()),
            instagram: Some("@bunker034".to_string()),
            soundcloud: None,
            bandcamp: None,
            youtube: None,
            spotify: None,
            website: Some("https://bunker034.example".to_string()),
            is_active: true,
            frequency: Some("Mensal".to_string()),
            slug: "bunker-034".to_string(),
        }
    }

    pub fn image(id: i32, path: &str, position: i32) -> Image {
        Image {
            id,
            path: path.to_string(),
            organization_id: 7,
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{image, organization};
    use super::*;

    const BASE: &str = "https://cdn.exemplo.com/rave-map";

    #[test]
    fn carries_every_public_field_across() {
        let org = organization();
        let view = OrganizationView::render(&org, &[], BASE);

        assert_eq!(view.id, org.id);
        assert_eq!(view.slug, org.slug);
        assert_eq!(view.name, org.name);
        assert_eq!(view.type_, org.type_);
        assert_eq!(view.city, org.city);
        assert_eq!(view.uf, org.uf);
        assert_eq!(view.about, org.about);
        assert_eq!(view.genres, org.genres);
        assert_eq!(view.address, org.address);
        assert_eq!(view.instagram, org.instagram);
        assert_eq!(view.website, org.website);
        assert_eq!(view.frequency, org.frequency);
        assert!(view.is_active);
        assert_eq!(view.status, "approved");
    }

    #[test]
    fn coordinates_become_numbers_the_map_can_draw() {
        let view = OrganizationView::render(&organization(), &[], BASE);

        assert!((view.latitude - -30.0346).abs() < 1e-9);
        assert!((view.longitude - -51.2177).abs() < 1e-9);
    }

    #[test]
    fn the_password_of_nobody_and_the_review_trail_stay_out_of_the_json() {
        // A view existe para isto: quem revisou e quando são dado interno.
        let json =
            serde_json::to_value(OrganizationView::render(&organization(), &[], BASE)).unwrap();

        assert!(json.get("reviewed_by").is_none());
        assert!(json.get("reviewed_at").is_none());
        assert_eq!(json["type"], "Coletivo", "o campo sai como \"type\"");
    }

    #[test]
    fn images_become_public_urls_in_the_order_they_came() {
        let images = vec![image(1, "aaaa.jpg", 0), image(2, "bbbb.png", 1)];

        let view = OrganizationView::render(&organization(), &images, BASE);

        assert_eq!(
            view.images
                .iter()
                .map(|i| i.url.as_str())
                .collect::<Vec<_>>(),
            vec![
                "https://cdn.exemplo.com/rave-map/aaaa.jpg",
                "https://cdn.exemplo.com/rave-map/bbbb.png",
            ]
        );
    }

    #[test]
    fn a_registration_without_images_renders_an_empty_list() {
        let view = OrganizationView::render(&organization(), &[], BASE);

        assert!(view.images.is_empty());
    }

    #[test]
    fn renders_a_whole_page_of_registrations() {
        let page = vec![
            (organization(), vec![image(1, "aaaa.jpg", 0)]),
            (organization(), vec![]),
        ];

        let views = OrganizationView::render_many(page, BASE);

        assert_eq!(views.len(), 2);
        assert_eq!(views[0].images.len(), 1);
        assert!(views[1].images.is_empty());
    }
}
