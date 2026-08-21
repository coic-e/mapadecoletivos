use actix_web::{test, web, App};

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

// Os testes de entrega de imagem saíram junto com o servidor de arquivos
// local: as imagens agora vivem no bucket, e as garantias que eles cobriam
// — não listar o diretório e não escapar dele — passaram a ser da política do
// bucket, que libera só GetObject. Isso se verifica contra o MinIO, não aqui.
