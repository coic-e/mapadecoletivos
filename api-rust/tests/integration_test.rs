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

// Note: These tests require a running database
// To run: DATABASE_URL=postgresql://docker:mapadecoletivos@localhost:5432/mapadecoletivos cargo test

#[cfg(all(test, not(debug_assertions)))]
mod with_database {
    use super::*;
    use api_rust::{config::AppConfig, db::establish_connection_pool, domains};

    #[actix_web::test]
    async fn test_list_organizations() {
        dotenv::dotenv().ok();
        let config = AppConfig::from_env().expect("Failed to load config");
        let pool = establish_connection_pool();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(config.clone()))
                .configure(domains::organizations::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/organizations").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
