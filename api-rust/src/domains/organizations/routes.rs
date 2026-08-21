/// HTTP routes for organizations domain
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use crate::auth::AdminIdentity;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::errors::ApiError;
use crate::handlers::upload::{parse_bigdecimal, process_multipart};
use crate::rate_limit::{client_key, SubmissionRateLimiter};
use crate::storage::Storage;
use api_types::OrganizationView;
use db_types::organization::{slugify, ModerationStatus, NewOrganization};

use super::actions;

/// Teto de itens por página. Sem ele, `?limit=100000000` faz a API carregar a
/// tabela inteira e as imagens de todo mundo em memória — negação de serviço
/// com uma requisição só.
const MAX_PAGE_SIZE: i64 = 100;
const DEFAULT_PAGE_SIZE: i64 = 50;

/// Normaliza limit/offset vindos da query string.
fn paginate(limit: Option<i64>, offset: Option<i64>) -> (Option<i64>, Option<i64>) {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    // Offset negativo é erro de sintaxe no Postgres, e o erro do banco voltava
    // como 500. Aqui vira simplesmente "do começo".
    let offset = offset.unwrap_or(0).max(0);

    (Some(limit), Some(offset))
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ModerationQuery {
    /// "pending" (padrão), "approved", "rejected" ou "all".
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RejectPayload {
    pub reason: Option<String>,
}

/// Configure routes for organizations domain
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/organizations")
            .route("", web::post().to(create))
            .route("", web::get().to(index))
            .route("/{id}", web::get().to(show)),
    );

    // Rotas de moderação. Todo handler aqui pede AdminIdentity, então sem
    // Bearer token válido a requisição morre em 401 antes de tocar no banco.
    cfg.service(
        web::scope("/admin/organizations")
            .route("", web::get().to(moderation_index))
            .route("/{id}", web::get().to(moderation_show))
            .route("/{id}/approve", web::post().to(approve))
            .route("/{id}/reject", web::post().to(reject)),
    );
}

/// GET /admin/organizations?status=pending — a fila de revisão.
pub async fn moderation_index(
    _identity: AdminIdentity,
    query: web::Query<ModerationQuery>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let requested = query
        .status
        .clone()
        .unwrap_or_else(|| ModerationStatus::PENDING.to_string());

    // "all" é a única palavra que não é um status do banco.
    let status = if requested == "all" {
        None
    } else {
        if !ModerationStatus::is_valid(&requested) {
            return Err(ApiError::ValidationError(
                vec![(
                    "status".to_string(),
                    vec![format!("Status inválido: {}", requested)],
                )]
                .into_iter()
                .collect(),
            ));
        }

        Some(requested)
    };

    let (limit, offset) = paginate(query.limit, query.offset);

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let organizations_with_images = web::block(move || {
        actions::get_organizations_for_moderation(&mut conn, status, limit, offset)
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let views =
        OrganizationView::render_many(organizations_with_images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(views))
}

/// GET /admin/organizations/{id} — enxerga qualquer estado, não só aprovado.
pub async fn moderation_show(
    _identity: AdminIdentity,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let organization_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) = web::block(move || {
        crate::domains::organizations::repository::OrganizationRepository::find_by_id(
            &mut conn,
            organization_id,
        )
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// POST /admin/organizations/{id}/approve — passa a aparecer no mapa.
pub async fn approve(
    identity: AdminIdentity,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    review(
        identity,
        path,
        pool,
        config,
        ModerationStatus::APPROVED,
        None,
    )
    .await
}

/// POST /admin/organizations/{id}/reject — sai do mapa, com motivo opcional.
pub async fn reject(
    identity: AdminIdentity,
    path: web::Path<i32>,
    payload: Option<web::Json<RejectPayload>>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let reason = payload
        .and_then(|body| body.reason.clone())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    review(
        identity,
        path,
        pool,
        config,
        ModerationStatus::REJECTED,
        reason,
    )
    .await
}

async fn review(
    identity: AdminIdentity,
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    status: &'static str,
    rejection_reason: Option<String>,
) -> Result<HttpResponse, ApiError> {
    let organization_id = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) = web::block(move || {
        actions::review_organization(
            &mut conn,
            organization_id,
            status,
            identity.id,
            rejection_reason,
        )
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// POST /organizations - Create a new organization
pub async fn create(
    req: HttpRequest,
    payload: Multipart,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    limiter: web::Data<SubmissionRateLimiter>,
    storage: web::Data<Storage>,
) -> Result<HttpResponse, ApiError> {
    // O cadastro é aberto e grava imagem no bucket: sem limite por IP, um
    // script enche o bucket e a fila de moderação em minutos. Conferido antes
    // de ler o corpo, para nem gastar banda com quem já estourou a cota.
    limiter.check(&format!("submit:{}", client_key(&req, &config)))?;

    // Extract multipart data (files + text fields)
    let multipart_data = process_multipart(payload, &config, &storage).await?;

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
        let keys: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        storage.remove(&keys).await;

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
    // Calculado antes de `name` ser movido para a struct.
    let slug = slugify(&name);
    let latitude_str = get_field("latitude")?;
    let longitude_str = get_field("longitude")?;
    let type_ = get_field("type")?;
    let city = get_field("city")?;
    let uf = get_field("uf")?;
    let email = get_field("email")?;
    let about = get_field("about")?;

    // Campo ausente e campo em branco valem a mesma coisa aqui: sem valor.
    let optional_field = |name: &str| -> Option<String> {
        multipart_data
            .fields
            .get(name)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    };

    // Os campos do multipart vão para um mapa, então nome repetido se
    // sobrescreveria: os gêneros chegam num campo só, separados por vírgula.
    let genres: Vec<String> = optional_field("genres")
        .map(|value| {
            value
                .split(',')
                .map(|genre| genre.trim().to_string())
                .filter(|genre| !genre.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Ausente significa ativo: é o caso comum.
    let is_active = optional_field("is_active")
        .map(|value| value != "false")
        .unwrap_or(true);

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
        about,
        genres,
        address: optional_field("address"),
        instagram: optional_field("instagram"),
        soundcloud: optional_field("soundcloud"),
        bandcamp: optional_field("bandcamp"),
        youtube: optional_field("youtube"),
        spotify: optional_field("spotify"),
        website: optional_field("website"),
        is_active,
        frequency: optional_field("frequency"),
        // A unicidade é resolvida na transação de criação.
        slug,
    };

    // Validate organization data
    if let Err(validation_errors) = new_organization.validate() {
        // Cleanup uploaded files on validation error
        let keys: Vec<String> = multipart_data
            .files
            .iter()
            .map(|f| f.filename.clone())
            .collect();
        storage.remove(&keys).await;

        return Err(ApiError::from(validation_errors));
    }

    // A limpeza do bucket é assíncrona, então as chaves ficam prontas antes:
    // cada saída por erro daqui para baixo apaga o que já subiu.
    let keys: Vec<String> = multipart_data
        .files
        .iter()
        .map(|f| f.filename.clone())
        .collect();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            storage.remove(&keys).await;
            return Err(ApiError::DatabaseError(e.to_string()));
        }
    };

    let files = keys.clone();

    // Execute business logic in blocking thread
    // Qual das fotos enviadas é a capa, pela ordem de envio.
    let cover_index = multipart_data
        .fields
        .get("cover_index")
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let result = web::block(move || {
        actions::create_organization(&mut conn, new_organization, files, cover_index)
    })
    .await;

    let (organization, images) = match result {
        Ok(Ok(created)) => created,
        Ok(Err(e)) => {
            storage.remove(&keys).await;
            return Err(e);
        }
        Err(e) => {
            storage.remove(&keys).await;
            return Err(ApiError::InternalError(e.to_string()));
        }
    };

    // Render view
    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Created().json(view))
}

/// GET /organizations/{id_ou_slug} - Um cadastro, por id ou por slug
///
/// Aceita os dois porque o site passou a usar slug, mas links antigos com id
/// continuam existindo por aí. Numérico é id; o resto é slug.
pub async fn show(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let identifier = path.into_inner();

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (organization, images) = web::block(move || match identifier.parse::<i32>() {
        Ok(organization_id) => actions::get_organization_by_id(&mut conn, organization_id),
        Err(_) => actions::get_organization_by_slug(&mut conn, &identifier),
    })
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let view = OrganizationView::render(&organization, &images, &config.storage.public_base_url);

    Ok(HttpResponse::Ok().json(view))
}

/// GET /organizations - Get all organizations with pagination
pub async fn index(
    query: web::Query<PaginationQuery>,
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, ApiError> {
    let (limit, offset) = paginate(query.limit, query.offset);

    let mut conn = pool
        .get()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let organizations_with_images =
        web::block(move || actions::get_all_organizations(&mut conn, limit, offset))
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))??;

    let base_url = config.storage.public_base_url.clone();
    let views = OrganizationView::render_many(organizations_with_images, &base_url);

    Ok(HttpResponse::Ok().json(views))
}
