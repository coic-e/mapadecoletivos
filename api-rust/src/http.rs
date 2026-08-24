//! As peças do servidor HTTP que valem para toda requisição.
//!
//! Ficam aqui, e não soltas dentro do `main`, porque são regras de segurança:
//! quais sites podem chamar a API pelo navegador, e o que todo navegador é
//! obrigado a assumir sobre as respostas. Montadas dentro do closure do
//! `HttpServer` elas não teriam como ser exercitadas por teste nenhum.
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::middleware::DefaultHeaders;
use actix_web::web;

use crate::config::AppConfig;
use crate::domains;
use crate::ops;

/// Origens autorizadas a chamar a API pelo navegador.
///
/// Só as configuradas, e só o que a aplicação usa de fato. O permissivo de
/// antes deixava qualquer site na internet ler as respostas da API pelo
/// navegador de quem visitasse.
pub fn cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PATCH", "OPTIONS"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(3600);

    for origin in &config.cors_allowed_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}

/// Cabeçalhos que vão em toda resposta.
pub fn security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
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
        ))
}

/// Todas as rotas da API, num lugar só, para que o servidor e os testes montem
/// exatamente o mesmo conjunto.
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(domains::organizations::configure)
        .configure(domains::admins::configure)
        .configure(domains::edit_requests::configure)
        .configure(ops::configure);
}
