//! Entrega das imagens enviadas pelos cadastros.
//!
//! Fica isolado num módulo porque as regras aqui são de segurança, não de
//! conveniência, e precisam valer igual em produção e nos testes.
use actix_files as fs;
use actix_web::dev::HttpServiceFactory;
use actix_web::http::header;
use actix_web::{middleware, web};

/// Monta /uploads em modo somente-leitura e sem listagem.
///
/// - sem `show_files_listing`: a listagem de antes entregava o nome de toda
///   imagem já enviada, inclusive de cadastros pendentes e rejeitados, que o
///   site não mostra a ninguém;
/// - `sandbox` + `nosniff`: mesmo que algum arquivo escape da validação de
///   conteúdo, o navegador não o executa como documento;
/// - `Cross-Origin-Resource-Policy: cross-origin` porque o site roda em outro
///   domínio e precisa conseguir exibir as imagens.
pub fn uploads_service(upload_dir: &str) -> impl HttpServiceFactory {
    web::scope("/uploads")
        .wrap(
            middleware::DefaultHeaders::new()
                .add((
                    header::CONTENT_SECURITY_POLICY,
                    "sandbox; default-src 'none'",
                ))
                .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                .add(("Cross-Origin-Resource-Policy", "cross-origin"))
                .add((header::CACHE_CONTROL, "public, max-age=31536000, immutable")),
        )
        .service(
            fs::Files::new("", upload_dir)
                .prefer_utf8(true)
                .disable_content_disposition(),
        )
}
