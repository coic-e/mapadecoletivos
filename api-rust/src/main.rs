use actix_cors::Cors;
use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};

use api_rust::{config::AppConfig, db::establish_connection_pool, domains};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration from environment");

    log::info!(
        "Starting server at {}:{}",
        config.server_host,
        config.server_port
    );
    log::info!("Database: {}", config.database_url);
    log::info!("Base URL: {}", config.base_url);

    // Establish database connection pool
    let pool = establish_connection_pool();

    // Clone values needed for server closure
    let server_host = config.server_host.clone();
    let server_port = config.server_port;
    let upload_dir = config.upload_dir.clone();

    HttpServer::new(move || {
        // Configure CORS - permissive for development
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // Static file serving for uploads
            .service(fs::Files::new("/uploads", &upload_dir).show_files_listing())
            // Domain routes
            .configure(domains::organizations::configure)
            // Health check endpoint
            .route(
                "/health",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            )
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
