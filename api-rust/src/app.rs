//! Montagem do app, num lugar só.
//!
//! Existe para que o servidor e os testes montem exatamente a mesma coisa. Com
//! a montagem inline no `main`, cada teste remontava o `App` à mão e ia ficando
//! parecido com produção sem nunca ser igual: um `app_data` esquecido aqui, um
//! middleware a menos ali, e o teste passava a atestar uma aplicação que não
//! existe em lugar nenhum.
use std::sync::Arc;
use std::time::Duration;

use actix_web::body::EitherBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{web, App, Error};

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::handlers::upload::payload_config;
use crate::http;
use crate::rate_limit::{LoginRateLimiter, SubmissionRateLimiter};
use crate::storage::SharedStore;

/// Os limitadores por IP. Nascem fora do closure do `HttpServer` porque cada
/// worker roda o closure uma vez, e um limitador por worker contaria cada IP N
/// vezes antes de barrar.
///
/// `Clone` clona o handle, não o contador: por dentro é `Arc`, então todos os
/// workers somam no mesmo lugar — que é justamente o ponto.
#[derive(Clone)]
pub struct Limiters {
    pub login: web::Data<LoginRateLimiter>,
    pub submission: web::Data<SubmissionRateLimiter>,
}

impl Limiters {
    pub fn from_config(config: &AppConfig) -> Self {
        let janela = Duration::from_secs(config.rate_limit_window_secs);

        Limiters {
            login: web::Data::new(LoginRateLimiter::new(config.login_rate_limit, janela)),
            submission: web::Data::new(SubmissionRateLimiter::new(
                config.submission_rate_limit,
                janela,
            )),
        }
    }

    fn clone_handles(
        &self,
    ) -> (
        web::Data<LoginRateLimiter>,
        web::Data<SubmissionRateLimiter>,
    ) {
        (self.login.clone(), self.submission.clone())
    }
}

pub fn build_app(
    pool: DbPool,
    storage: SharedStore,
    config: AppConfig,
    limiters: &Limiters,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<EitherBody<actix_web::body::BoxBody>>,
        Error = Error,
        InitError = (),
    >,
> {
    let (login, submission) = limiters.clone_handles();

    App::new()
        .app_data(web::Data::new(pool))
        .app_data(web::Data::new(storage))
        .app_data(login)
        .app_data(submission)
        .app_data(payload_config(&config))
        // Corpo JSON pequeno: as rotas que recebem JSON só levam login e
        // motivo de rejeição.
        .app_data(web::JsonConfig::default().limit(16 * 1024))
        .wrap(http::security_headers())
        .wrap(http::cors(&config))
        // Por último no encadeamento, porque tudo acima o consulta.
        .app_data(web::Data::new(config))
        .configure(http::routes)
}

/// Atalho para quem já tem os pedaços soltos.
pub fn shared_store(store: impl crate::storage::ImageStore + 'static) -> SharedStore {
    Arc::new(store)
}
