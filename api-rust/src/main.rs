use std::time::Duration;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use api_rust::handlers::upload::payload_config;
use api_rust::rate_limit::{LoginRateLimiter, SubmissionRateLimiter};
use api_rust::storage::Storage;
use api_rust::{config::AppConfig, db::establish_connection_pool, domains};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = match AppConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            // Sem log::error aqui: a config é o que decide o logger, e a
            // mensagem precisa aparecer mesmo com RUST_LOG desligado.
            eprintln!("configuração inválida: {e}");
            std::process::exit(1);
        }
    };

    log::info!(
        "Starting server at {}:{}",
        config.server_host,
        config.server_port
    );
    // DATABASE_URL fica fora do log: ela carrega usuário e senha do banco, e
    // log de aplicação costuma ir parar em serviço de terceiro.
    log::info!("Base URL: {}", config.base_url);
    log::info!(
        "Bucket: {} em {}",
        config.storage.bucket,
        if config.storage.endpoint.is_empty() {
            "AWS S3"
        } else {
            &config.storage.endpoint
        }
    );
    log::info!("Origens CORS: {:?}", config.cors_allowed_origins);

    // Establish database connection pool
    let pool = establish_connection_pool(&config);

    // Clone values needed for server closure
    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    // Um cliente só, compartilhado pelos workers: ele mantém pool de conexões
    // por dentro, e criar um por worker desperdiçaria isso.
    let storage = web::Data::new(Storage::new(&config.storage));

    let rate_window = Duration::from_secs(config.rate_limit_window_secs);
    // Fora do closure: cada worker do actix roda o closure uma vez, e um
    // limitador por worker contaria cada IP N vezes antes de barrar.
    let login_limiter = web::Data::new(LoginRateLimiter::new(config.login_rate_limit, rate_window));
    let submission_limiter = web::Data::new(SubmissionRateLimiter::new(
        config.submission_rate_limit,
        rate_window,
    ));

    HttpServer::new(move || {
        // Só as origens configuradas, e só o que a aplicação usa de fato. O
        // permissivo de antes deixava qualquer site na internet ler as
        // respostas da API pelo navegador de quem visitasse.
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
            .max_age(3600);

        for origin in &config.cors_allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(storage.clone())
            .app_data(login_limiter.clone())
            .app_data(submission_limiter.clone())
            .app_data(payload_config(&config))
            // Corpo JSON pequeno: as rotas que recebem JSON só levam login e
            // motivo de rejeição.
            .app_data(web::JsonConfig::default().limit(16 * 1024))
            .wrap(cors)
            .wrap(
                middleware::DefaultHeaders::new()
                    .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                    .add((header::X_FRAME_OPTIONS, "DENY"))
                    .add((header::REFERRER_POLICY, "no-referrer"))
                    .add((
                        header::STRICT_TRANSPORT_SECURITY,
                        "max-age=31536000; includeSubDomains",
                    ))
                    .add(("Cross-Origin-Opener-Policy", "same-origin"))
                    .add((
                        "Permissions-Policy",
                        "geolocation=(), camera=(), microphone=()",
                    )),
            )
            .wrap(middleware::Logger::default())
            // Domain routes
            .configure(domains::organizations::configure)
            .configure(domains::admins::configure)
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
