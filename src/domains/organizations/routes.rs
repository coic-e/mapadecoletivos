/// HTTP routes for organizations domain
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::handlers::upload::{cleanup_files, parse_bigdecimal, process_multipart};
use crate::models::organization::NewOrganization;
use crate::views::OrganizationView;

use super::actions;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Configure routes for organizations domain
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/organizations")
            .route("", web::post().to(create))
            .route("", web::get().to(index))
            .route("/{id}", web::get().to(show)),
    );
}

/// POST /organizations - Create a new organization
pub async fn create(
    payload: Multipart,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    // Extract multipart data (files + text fields)
    let multipart_data = process_multipart(payload, &config).await?;

    // Helper to get required field
    let get_field = |name: &str| -> Result<String, ApiError> {
        multipart_data.fields.get(name).cloned().ok_or_else(|| {
            ApiError::ValidationError(
                vec![(name.to_string(), vec![format!("{} é obrigatório", name)])]
                    .into_iter()
                    .collect(),
            )
        })
    };

    // Extract and validate images
    if multipart_data.files.is_empty() {
        // Cleanup any uploaded files
        let filenames: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        cleanup_files(&config.upload_dir, &filenames);

        return Err(ApiError::ValidationError(
            vec![(
                "images".to_string(),
                vec!["Ao menos uma imagem é obrigatória".to_string()],
            )]
            .into_iter()
            .collect(),
        ));
    }

    // Extract text fields
    let name = get_field("name")?;
    let latitude_str = get_field("latitude")?;
    let longitude_str = get_field("longitude")?;
    let type_ = get_field("type")?;
    let city = get_field("city")?;
    let uf = get_field("uf")?;
    let email = get_field("email")?;
    let social = get_field("social")?;
    let about = get_field("about")?;

    // Parse numeric fields
    let latitude = parse_bigdecimal("latitude", &latitude_str)?;
    let longitude = parse_bigdecimal("longitude", &longitude_str)?;

    // Build NewOrganization
    let new_organization = NewOrganization {
        name,
        latitude,
        longitude,
        type_,
        city,
        uf,
        email,
        social,
        about,
    };

    // Validate organization data
    if let Err(validation_errors) = new_organization.validate() {
        // Cleanup uploaded files on validation error
        let filenames: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        cleanup_files(&config.upload_dir, &filenames);

        return Err(ApiError::from(validation_errors));
    }

    // Get database connection
    let mut conn = pool.get().map_err(|e| {
        // Cleanup files on connection error
        let filenames: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        cleanup_files(&config.upload_dir, &filenames);
        ApiError::DatabaseError(e.to_string())
    })?;

    // Clone data for the blocking operation
    let files: Vec<String> = multipart_data
        .files
        .iter()
        .map(|f| f.filename.clone())
        .collect();
    let upload_dir = config.upload_dir.clone();

    // Execute business logic in blocking thread
    let result =
        web::block(move || actions::create_organization(&mut conn, new_organization, files))
            .await
            .map_err(|e| {
                // Cleanup files on blocking error
                let filenames: Vec<String> = multipart_data
                    .files
                    .iter()
                    .map(|f| f.filename.clone())
                    .collect();
                cleanup_files(&upload_dir, &filenames);
                ApiError::InternalError(e.to_string())
            })?;

    // Handle database result
    let (organization, images) = result.inspect_err(|_e| {
        // Cleanup files on database error
        let filenames: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        cleanup_files(&config.upload_dir, &filenames);
    })?;

    // Render view
    let view = OrganizationView::render(&organization, &images, &config.base_url);

    Ok(HttpResponse::Created().json(view))
}

/// GET /organizations/{id} - Get a single organization
pub async fn show(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let organization_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) =
        web::block(move || actions::get_organization_by_id(&mut conn, organization_id))
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// GET /organizations - Get all organizations with pagination
pub async fn index(
    query: web::Query<PaginationQuery>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let limit = query.limit;
    let offset = query.offset;

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let organizations_with_images =
        web::block(move || actions::get_all_organizations(&mut conn, limit, offset))
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let base_url = config.base_url.clone();
    let views = OrganizationView::render_many(organizations_with_images, &base_url);

    Ok(HttpResponse::Ok().json(views))
}
