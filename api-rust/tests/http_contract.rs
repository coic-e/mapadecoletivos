//! O que a API promete para todo cliente, independentemente do banco.
//!
//! Nada aqui toca em Postgres ou bucket: são as garantias que valem antes de
//! qualquer handler rodar — quem entra sem token, quais sites podem chamar a
//! API pelo navegador e o que todo navegador é obrigado a assumir sobre as
//! respostas.
//!
//! Os testes de entrega de imagem saíram junto com o servidor de arquivos
//! local: as imagens agora vivem no bucket, e as garantias que eles cobriam
//! — não listar o diretório e não escapar dele — passaram a ser da política do
//! bucket, que libera só GetObject. Isso se verifica contra o RustFS, não aqui.
mod common;

use actix_web::http::{header, StatusCode};
use actix_web::{test, web, App};
use api_rust::app::{build_app, Limiters};
use api_rust::auth::{create_token, AdminIdentity, Claims};
use jsonwebtoken::{EncodingKey, Header};
use std::sync::Arc;

use common::{config, pool_that_never_connects, NoopStore, ALLOWED_ORIGIN};

/// O app de produção, montado pelo mesmo `build_app` que o servidor usa.
///
/// O pool aponta para um banco que não existe: nada que este arquivo exercita
/// chega a abrir conexão — tudo acontece antes, no middleware ou no extractor.
/// Um caso que precisasse do banco falharia aqui, e é assim que se descobre
/// que ele foi parar no arquivo errado.
macro_rules! app {
    () => {{
        let config = config();
        let limiters = Limiters::from_config(&config);
        let pool = pool_that_never_connects();

        test::init_service(build_app(pool, Arc::new(NoopStore), config, &limiters)).await
    }};
}

// ---------------------------------------------------------------- saúde

#[actix_web::test]
async fn health_answers_without_touching_any_dependency() {
    // É o que alimenta a política de reinício: derrubar o container porque o
    // Postgres piscou não conserta nada e ainda tira a API do ar junto.
    let app = app!();

    let resp = test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
}

// ------------------------------------------------------- rotas fechadas

/// Toda rota que só um moderador pode alcançar.
const ADMIN_ROUTES: &[(&str, &str)] = &[
    ("GET", "/auth/me"),
    ("GET", "/admin/organizations"),
    ("GET", "/admin/organizations/1"),
    ("PATCH", "/admin/organizations/1"),
    ("POST", "/admin/organizations/1/approve"),
    ("POST", "/admin/organizations/1/reject"),
    ("GET", "/admin/edit-requests"),
    ("POST", "/admin/edit-requests/1/apply"),
    ("POST", "/admin/edit-requests/1/reject"),
];

fn request(method: &str, uri: &str) -> test::TestRequest {
    match method {
        "GET" => test::TestRequest::get(),
        "POST" => test::TestRequest::post(),
        "PATCH" => test::TestRequest::patch(),
        "PUT" => test::TestRequest::put(),
        "DELETE" => test::TestRequest::delete(),
        outro => panic!("método não previsto no teste: {outro}"),
    }
    .uri(uri)
}

#[actix_web::test]
async fn every_admin_route_refuses_a_request_without_a_token() {
    let app = app!();

    for (method, uri) in ADMIN_ROUTES {
        let resp = test::call_service(&app, request(method, uri).to_request()).await;

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} deveria exigir moderador"
        );
    }
}

#[actix_web::test]
async fn a_forged_or_stale_token_gets_nowhere() {
    let app = app!();
    let config = config();

    let outro_segredo = api_rust::config::AppConfig {
        jwt_secret: "um-outro-segredo-longo-o-suficiente-1234".to_string(),
        ..config.clone()
    };

    let expirado = Claims {
        sub: 1,
        email: "mod@exemplo.com".to_string(),
        iat: 0,
        exp: 1,
    };

    let tokens = [
        (
            "assinado com outro segredo",
            create_token(1, "mod@exemplo.com", &outro_segredo).unwrap(),
        ),
        (
            "expirado",
            jsonwebtoken::encode(
                &Header::default(),
                &expirado,
                &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
            )
            .unwrap(),
        ),
        (
            "alg none",
            "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOjEsImVtYWlsIjoiYUBiLmMiLCJpYXQiOjAsImV4cCI6OTk5OTk5OTk5OX0."
                .to_string(),
        ),
        ("texto qualquer", "nao-e-um-token".to_string()),
    ];

    for (descricao, token) in tokens {
        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/auth/me")
                .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
                .to_request(),
        )
        .await;

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "token {descricao} deveria ser recusado"
        );
    }
}

#[actix_web::test]
async fn the_token_has_to_come_as_a_bearer() {
    let app = app!();
    let config = config();
    let token = create_token(1, "mod@exemplo.com", &config).unwrap();

    for header_value in [
        token.clone(),
        format!("Basic {token}"),
        format!("bearer {token}"),
        String::new(),
    ] {
        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/auth/me")
                .insert_header((header::AUTHORIZATION, header_value.clone()))
                .to_request(),
        )
        .await;

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "cabeçalho {header_value:?} não é um Bearer"
        );
    }
}

#[actix_web::test]
async fn the_refusal_says_nothing_about_why() {
    // "usuário não existe" e "token expirado" são a mesma resposta: a
    // diferença serviria para sondar quais contas existem.
    let app = app!();

    let resp =
        test::call_service(&app, test::TestRequest::get().uri("/auth/me").to_request()).await;

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["status"], "Error");
    assert!(
        !body.to_string().to_lowercase().contains("senha"),
        "a resposta não deveria falar de senha: {body}"
    );
}

#[actix_web::test]
async fn an_identity_is_never_taken_from_a_header_the_client_can_write() {
    // Uma tentativa óbvia: mandar o id do admin num cabeçalho próprio.
    let app = app!();

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/auth/me")
            .insert_header(("X-Admin-Id", "1"))
            .insert_header(("X-User-Id", "1"))
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ------------------------------------------------------ cabeçalhos e CORS

#[actix_web::test]
async fn every_response_carries_the_security_headers() {
    let app = app!();

    let resp = test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;

    let esperados = [
        (header::X_CONTENT_TYPE_OPTIONS.as_str(), "nosniff"),
        (header::X_FRAME_OPTIONS.as_str(), "DENY"),
        (header::REFERRER_POLICY.as_str(), "no-referrer"),
        (
            header::STRICT_TRANSPORT_SECURITY.as_str(),
            "max-age=31536000; includeSubDomains",
        ),
        ("cross-origin-opener-policy", "same-origin"),
        (
            "permissions-policy",
            "geolocation=(), camera=(), microphone=()",
        ),
    ];

    for (nome, valor) in esperados {
        assert_eq!(
            resp.headers().get(nome).and_then(|v| v.to_str().ok()),
            Some(valor),
            "faltou {nome}"
        );
    }
}

#[actix_web::test]
async fn the_security_headers_survive_an_error_response() {
    // Resposta de erro também vai para o navegador, e é a que mais tende a
    // escapar de middleware quando alguém mexe na ordem dos wrappers.
    let app = app!();

    let resp =
        test::call_service(&app, test::TestRequest::get().uri("/auth/me").to_request()).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
}

#[actix_web::test]
async fn the_configured_origin_may_read_the_answers() {
    let app = app!();

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/health")
            .insert_header((header::ORIGIN, ALLOWED_ORIGIN))
            .to_request(),
    )
    .await;

    assert_eq!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v| v.to_str().ok()),
        Some(ALLOWED_ORIGIN)
    );
}

#[actix_web::test]
async fn any_other_site_may_not() {
    // Sem isto, qualquer página na internet lê as respostas da API pelo
    // navegador de quem a visitar — com o token do moderador junto.
    let app = app!();

    for intruso in [
        "https://mapa.exemplo.com.evil.test",
        "http://mapa.exemplo.com",
        "https://evil.test",
        "null",
    ] {
        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/health")
                .insert_header((header::ORIGIN, intruso))
                .to_request(),
        )
        .await;

        assert!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none(),
            "{intruso} não deveria ser autorizado"
        );
    }
}

#[actix_web::test]
async fn the_preflight_allows_what_the_panel_actually_does() {
    // O painel edita cadastro por PATCH. Fora desta lista, o navegador barra a
    // requisição na verificação prévia e a edição simplesmente não acontece —
    // sem erro visível na API.
    let app = app!();

    for metodo in ["GET", "POST", "PATCH"] {
        let resp = test::call_service(
            &app,
            test::TestRequest::default()
                .method(actix_web::http::Method::OPTIONS)
                .uri("/admin/organizations/1")
                .insert_header((header::ORIGIN, ALLOWED_ORIGIN))
                .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, metodo))
                .insert_header((header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization"))
                .to_request(),
        )
        .await;

        let permitidos = resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();

        assert!(
            resp.status().is_success(),
            "preflight de {metodo} respondeu {}",
            resp.status()
        );
        assert!(
            permitidos.contains(metodo),
            "{metodo} deveria estar em {permitidos:?}"
        );
    }
}

#[actix_web::test]
async fn the_preflight_refuses_what_the_api_does_not_do() {
    let app = app!();

    for metodo in ["DELETE", "PUT"] {
        let resp = test::call_service(
            &app,
            test::TestRequest::default()
                .method(actix_web::http::Method::OPTIONS)
                .uri("/admin/organizations/1")
                .insert_header((header::ORIGIN, ALLOWED_ORIGIN))
                .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, metodo))
                .to_request(),
        )
        .await;

        assert!(
            !resp.status().is_success(),
            "{metodo} não é método desta API"
        );
    }
}

// ---------------------------------------------------------------- rotas

#[actix_web::test]
async fn an_unknown_route_is_a_404_not_a_crash() {
    let app = app!();

    for uri in [
        "/nao-existe",
        "/organizations/../../etc/passwd",
        "/uploads/qualquer.jpg",
        "/admin",
    ] {
        let resp = test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;

        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{uri} deveria ser 404"
        );
    }
}

/// Todo caminho que a API atende, com o método que o atende.
///
/// A lista existe porque o caminho de cada handler vive num atributo em cima
/// dele: um erro de digitação ali não quebra a compilação, só faz a rota sumir
/// — e o front recebe 404 numa tela que funcionava.
const EVERY_ROUTE: &[(&str, &str)] = &[
    ("GET", "/health"),
    ("GET", "/health/ready"),
    ("POST", "/auth/login"),
    ("GET", "/auth/me"),
    ("GET", "/organizations"),
    ("POST", "/organizations"),
    ("GET", "/organizations/bunker-034"),
    ("POST", "/organizations/bunker-034/edit-requests"),
    ("GET", "/admin/organizations"),
    ("GET", "/admin/organizations/1"),
    ("PATCH", "/admin/organizations/1"),
    ("POST", "/admin/organizations/1/approve"),
    ("POST", "/admin/organizations/1/reject"),
    ("GET", "/admin/edit-requests"),
    ("POST", "/admin/edit-requests/1/apply"),
    ("POST", "/admin/edit-requests/1/reject"),
];

#[actix_web::test]
async fn every_route_is_registered_at_the_path_it_advertises() {
    // 404 aqui significa "não existe rota nesse caminho". Qualquer outra
    // resposta — 401 por falta de token, 400 por corpo inválido, 500 porque o
    // pool deste arquivo não conecta — prova que o caminho casou.
    let app = app!();

    for (method, uri) in EVERY_ROUTE {
        let resp = test::call_service(&app, request(method, uri).to_request()).await;

        assert_ne!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{method} {uri} não está registrado"
        );
    }
}

#[actix_web::test]
async fn a_route_answers_only_the_method_it_advertises() {
    // Sem isto, trocar #[post] por #[get] num handler passaria: o caminho
    // continua registrado, e o teste acima seguiria verde.
    let app = app!();

    let trocas = [
        ("GET", "/auth/login"),
        ("POST", "/auth/me"),
        ("GET", "/admin/organizations/1/approve"),
        ("GET", "/admin/edit-requests/1/apply"),
        ("POST", "/admin/organizations/1"),
        ("PATCH", "/organizations"),
    ];

    for (method, uri) in trocas {
        let resp = test::call_service(&app, request(method, uri).to_request()).await;

        assert!(
            matches!(
                resp.status(),
                StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED
            ),
            "{method} {uri} deveria ser recusado, veio {}",
            resp.status()
        );
    }
}

#[actix_web::test]
async fn the_extractor_type_is_what_closes_the_route() {
    // Guarda de regressão do desenho: se alguém trocar `AdminIdentity` por um
    // parâmetro opcional num handler, este arquivo inteiro continua passando —
    // menos isto, que monta uma rota nova com o extractor e confere que ele
    // sozinho já barra.
    let app = test::init_service(App::new().app_data(web::Data::new(config())).route(
        "/rota-de-teste",
        web::get().to(|_: AdminIdentity| async { actix_web::HttpResponse::Ok().finish() }),
    ))
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/rota-de-teste").to_request(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
