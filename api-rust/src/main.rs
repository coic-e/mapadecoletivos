use std::sync::Arc;

use actix_web::{middleware, HttpServer};

use api_rust::app::{build_app, Limiters};
use api_rust::bootstrap::{seed_first_admin, AdminSeed};
use api_rust::migrations;
use api_rust::storage::{SharedStore, Storage};
use api_rust::{config::AppConfig, db::establish_connection_pool};

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
    // Antes do pool e antes de escutar: se o esquema não estiver no lugar, é
    // melhor não subir do que atender requisição com o banco a meio caminho.
    if let Err(e) = migrations::run(&config.database_url) {
        log::error!("{e}");
        std::process::exit(1);
    }

    let pool = establish_connection_pool(&config);

    // Depois das migrações, porque depende da tabela existir. Falha aqui
    // derruba a subida: semente inválida significa painel inacessível, e é
    // melhor saber disso no deploy do que na hora de moderar.
    {
        let mut conn = pool
            .get()
            .expect("não consegui pegar conexão para verificar os moderadores");

        if let Err(e) = seed_first_admin(&mut conn, AdminSeed::from_env()) {
            log::error!("{e}");
            std::process::exit(1);
        }
    }

    // Clone values needed for server closure
    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    // Um cliente só, compartilhado pelos workers: ele mantém pool de conexões
    // por dentro, e criar um por worker desperdiçaria isso.
    let storage: SharedStore = Arc::new(Storage::new(&config.storage));

    let limiters = Limiters::from_config(&config);

    // O Logger é montado aqui, e não dentro do build_app: mantém a assinatura
    // do build_app com um tipo de corpo só, e deixa os testes sem access log.
    HttpServer::new(move || {
        build_app(pool.clone(), storage.clone(), config.clone(), &limiters)
            .wrap(middleware::Logger::default())
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
