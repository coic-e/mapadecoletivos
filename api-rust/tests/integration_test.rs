use actix_web::{test, web, App};

use api_rust::handlers::static_files::uploads_service;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().route(
        "/health",
        web::get().to(|| async { actix_web::HttpResponse::Ok().body("OK") }),
    ))
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Prepara um diretório de uploads com uma imagem, como a API o deixaria.
fn uploads_fixture(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rave-map-uploads-{name}"));

    std::fs::create_dir_all(&dir).expect("criar diretório de teste");
    std::fs::write(dir.join("imagem.png"), b"\x89PNG\r\n\x1a\n").expect("gravar imagem");

    dir
}

#[actix_web::test]
async fn serves_uploaded_images() {
    let dir = uploads_fixture("serve");
    let app = test::init_service(App::new().service(uploads_service(dir.to_str().unwrap()))).await;

    let req = test::TestRequest::get()
        .uri("/uploads/imagem.png")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "a imagem precisa continuar sendo servida"
    );

    let headers = resp.headers();
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert!(headers
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("sandbox"));
}

#[actix_web::test]
async fn does_not_list_the_uploads_directory() {
    // A listagem entregava o nome de toda imagem enviada, inclusive as de
    // cadastros pendentes e rejeitados.
    let dir = uploads_fixture("listing");
    let app = test::init_service(App::new().service(uploads_service(dir.to_str().unwrap()))).await;

    for uri in ["/uploads/", "/uploads"] {
        let req = test::TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            !resp.status().is_success(),
            "{uri} não deveria listar o diretório"
        );
    }
}

#[actix_web::test]
async fn does_not_escape_the_uploads_directory() {
    let dir = uploads_fixture("traversal");
    let app = test::init_service(App::new().service(uploads_service(dir.to_str().unwrap()))).await;

    for uri in [
        "/uploads/../../../etc/passwd",
        "/uploads/%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        "/uploads/..%2f..%2fetc%2fpasswd",
    ] {
        let req = test::TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            !resp.status().is_success(),
            "{uri} não deveria sair da pasta de uploads"
        );
    }
}
