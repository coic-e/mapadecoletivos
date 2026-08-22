//! O acesso do moderador, da senha ao token, contra um banco de verdade.
//!
//! É o único ponto da API com poder de publicar, e o único que guarda senha.
//! Os casos aqui cobrem as duas pontas: quem entra, e o que acontece com quem
//! deixou de ser moderador mas ainda tem um token válido no bolso.
//!
//! Rodam só com `TEST_DATABASE_URL` definida.
mod common;

use actix_web::http::{header, StatusCode};
use actix_web::test;
use api_rust::app::{build_app, Limiters};
use api_rust::auth::{create_token, hash_password, verify_password};
use api_rust::bootstrap::{seed_first_admin, AdminSeed};
use api_rust::config::AppConfig;
use api_rust::domains::admins::repository::AdminRepository;
use api_rust::errors::ApiError;
use db_types::admin::NewAdmin;
use std::sync::Arc;

use common::{clear, config, pool, with_database, NoopStore};

const SENHA: &str = "uma-senha-de-teste-comprida";

fn novo_admin(email: &str) -> NewAdmin {
    NewAdmin {
        name: "Moderação".to_string(),
        email: email.to_string(),
        password_hash: hash_password(SENHA).unwrap(),
    }
}

// ------------------------------------------------------------ repositório

#[actix_web::test]
async fn an_email_that_does_not_exist_is_not_an_error() {
    // Precisa ser Option, e não erro: no login, "não existe" e "senha errada"
    // têm que virar a mesma resposta.
    with_database("an_email_that_does_not_exist_is_not_an_error", |conn| {
        clear(conn);

        assert!(AdminRepository::find_by_email(conn, "ninguem@exemplo.com")
            .unwrap()
            .is_none());
    });
}

#[actix_web::test]
async fn the_stored_password_is_a_hash_that_only_matches_the_original() {
    with_database(
        "the_stored_password_is_a_hash_that_only_matches_the_original",
        |conn| {
            clear(conn);

            let criado = AdminRepository::create(conn, &novo_admin("mod@exemplo.com")).unwrap();

            assert!(
                !criado.password_hash.contains(SENHA),
                "a senha não pode estar legível no banco"
            );
            assert!(criado.password_hash.starts_with("$argon2"));

            let lido = AdminRepository::find_by_email(conn, "mod@exemplo.com")
                .unwrap()
                .expect("deveria achar");

            assert!(verify_password(SENHA, &lido.password_hash));
            assert!(!verify_password("outra-senha", &lido.password_hash));
        },
    );
}

#[actix_web::test]
async fn a_deleted_moderator_is_a_404_by_id() {
    with_database("a_deleted_moderator_is_a_404_by_id", |conn| {
        clear(conn);

        assert!(matches!(
            AdminRepository::find_by_id(conn, 999_999),
            Err(ApiError::NotFound)
        ));
    });
}

// -------------------------------------------------------------- semente

#[actix_web::test]
async fn the_seed_creates_the_first_moderator_and_only_the_first() {
    with_database(
        "the_seed_creates_the_first_moderator_and_only_the_first",
        |conn| {
            clear(conn);

            let semente = || AdminSeed {
                name: "Primeira".to_string(),
                email: "primeira@exemplo.com".to_string(),
                password: "uma-senha-bem-comprida".to_string(),
            };

            seed_first_admin(conn, Some(semente())).expect("deveria criar");

            let criado = AdminRepository::find_by_email(conn, "primeira@exemplo.com")
                .unwrap()
                .expect("o primeiro moderador deveria existir");

            assert!(verify_password(
                "uma-senha-bem-comprida",
                &criado.password_hash
            ));

            // Segunda subida com a mesma variável no ambiente: não pode
            // reaplicar a senha. Se aplicasse, quem tivesse acesso ao painel
            // assumiria a conta editando uma variável e reiniciando o serviço.
            seed_first_admin(
                conn,
                Some(AdminSeed {
                    name: "Sequestro".to_string(),
                    email: "primeira@exemplo.com".to_string(),
                    password: "senha-trocada-pelo-atacante".to_string(),
                }),
            )
            .expect("deveria ignorar em silêncio");

            let depois = AdminRepository::find_by_id(conn, criado.id).unwrap();

            assert_eq!(depois.name, "Primeira", "a conta não pode ser reescrita");
            assert!(
                verify_password("uma-senha-bem-comprida", &depois.password_hash),
                "a senha original continua valendo"
            );
        },
    );
}

#[actix_web::test]
async fn a_short_seed_password_stops_the_boot() {
    // Falhar alto é melhor do que subir com um moderador de senha fraca.
    with_database("a_short_seed_password_stops_the_boot", |conn| {
        clear(conn);

        let erro = seed_first_admin(
            conn,
            Some(AdminSeed {
                name: "Curta".to_string(),
                email: "curta@exemplo.com".to_string(),
                password: "123".to_string(),
            }),
        )
        .expect_err("senha curta deveria derrubar a subida");

        assert!(erro.contains("12"), "a mensagem diz o mínimo: {erro}");
        assert!(
            AdminRepository::find_by_email(conn, "curta@exemplo.com")
                .unwrap()
                .is_none(),
            "nada foi criado"
        );
    });
}

#[actix_web::test]
async fn no_seed_and_no_moderators_is_not_an_error() {
    // É só um aviso: a API sobe e atende o público; só o painel fica fechado.
    with_database("no_seed_and_no_moderators_is_not_an_error", |conn| {
        clear(conn);

        assert!(seed_first_admin(conn, None).is_ok());
    });
}

// ---------------------------------------------------------------- login

/// A API inteira, com pool de verdade, montada pelo `build_app` de produção.
macro_rules! api {
    ($config:expr) => {{
        let limiters = Limiters::from_config(&$config);

        test::init_service(build_app(
            pool().unwrap().clone(),
            Arc::new(NoopStore),
            $config.clone(),
            &limiters,
        ))
        .await
    }};
}

/// Cria um moderador fora da transação de teste e o apaga no fim.
///
/// O login passa pelo `web::block`, que roda noutra thread e pega outra
/// conexão do pool: ele não enxergaria uma conta criada dentro de uma
/// transação aberta aqui.
struct ModeradorTemporario {
    id: i32,
    email: String,
    /// Segura a trava do banco pela vida do caso: esta conta é confirmada, e
    /// os testes que contam a tabela inteira não podem enxergá-la.
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl ModeradorTemporario {
    fn novo() -> Option<Self> {
        let pool = pool()?;
        let _guard = common::db_guard();
        let mut conn = pool.get().unwrap();

        let email = format!(
            "login-{}@exemplo.com",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let admin = AdminRepository::create(&mut conn, &novo_admin(&email)).unwrap();

        Some(ModeradorTemporario {
            id: admin.id,
            email,
            _guard,
        })
    }
}

impl Drop for ModeradorTemporario {
    fn drop(&mut self) {
        use db_types::schema::admins;
        use diesel::prelude::*;

        if let Some(pool) = pool() {
            let mut conn = pool.get().unwrap();
            let _ = diesel::delete(admins::table.find(self.id)).execute(&mut conn);
        }
    }
}

/// Pula o caso quando não há banco configurado.
macro_rules! moderador {
    ($nome:literal) => {
        match ModeradorTemporario::novo() {
            Some(moderador) => moderador,
            None => {
                eprintln!("pulando {}: defina TEST_DATABASE_URL", $nome);

                return;
            }
        }
    };
}

/// Genérico no corpo porque o `build_app` embrulha a resposta em `EitherBody`
/// — é o CORS decidindo entre a resposta do handler e a dele.
async fn login<B>(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    email: &str,
    senha: &str,
) -> actix_web::dev::ServiceResponse<B> {
    test::call_service(
        app,
        test::TestRequest::post()
            .uri("/auth/login")
            .set_json(serde_json::json!({ "email": email, "password": senha }))
            .to_request(),
    )
    .await
}

#[actix_web::test]
async fn the_right_password_opens_a_session() {
    let moderador = moderador!("the_right_password_opens_a_session");
    let config = config();
    let app = api!(config);

    let resp = login(&app, &moderador.email, SENHA).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["token"].as_str().expect("deveria vir um token");

    assert_eq!(body["admin"]["email"], moderador.email);
    assert!(
        body["admin"].get("password_hash").is_none(),
        "o hash da senha nunca sai da API: {body}"
    );

    // E o token abre a porta que ele deveria abrir.
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/auth/me")
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);

    let eu: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(eu["email"], moderador.email);
    assert!(eu.get("password_hash").is_none());
}

#[actix_web::test]
async fn the_email_is_matched_without_case_or_padding() {
    let moderador = moderador!("the_email_is_matched_without_case_or_padding");
    let config = config();
    let app = api!(config);

    let resp = login(
        &app,
        &format!("  {}  ", moderador.email.to_uppercase()),
        SENHA,
    )
    .await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "quem digita com maiúscula é a mesma pessoa"
    );
}

#[actix_web::test]
async fn the_wrong_password_and_an_unknown_email_answer_the_same_thing() {
    // A diferença entre as duas respostas seria a lista de quem modera o site.
    let moderador = moderador!("the_wrong_password_and_an_unknown_email_answer_the_same_thing");
    let config = config();
    let app = api!(config);

    let senha_errada = login(&app, &moderador.email, "senha-errada-mesmo").await;
    let status_senha = senha_errada.status();
    let corpo_senha: serde_json::Value = test::read_body_json(senha_errada).await;

    let inexistente = login(&app, "ninguem-mesmo@exemplo.com", SENHA).await;
    let status_email = inexistente.status();
    let corpo_email: serde_json::Value = test::read_body_json(inexistente).await;

    assert_eq!(status_senha, StatusCode::UNAUTHORIZED);
    assert_eq!(status_email, StatusCode::UNAUTHORIZED);
    assert_eq!(
        corpo_senha, corpo_email,
        "as duas respostas precisam ser indistinguíveis"
    );
}

#[actix_web::test]
async fn an_absurd_email_or_password_is_refused_before_the_database() {
    let moderador = moderador!("an_absurd_email_or_password_is_refused_before_the_database");
    let config = config();
    let app = api!(config);

    let email_gigante = format!("{}@exemplo.com", "a".repeat(400));
    let resp = login(&app, &email_gigante, SENHA).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let resp = login(&app, &moderador.email, &"a".repeat(600)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn brute_force_runs_into_the_rate_limit() {
    // Sem o limite, uma senha de moderador cai na velocidade da rede.
    let moderador = moderador!("brute_force_runs_into_the_rate_limit");
    // Cota apertada para o teste não precisar gastar a cota real.
    let config = AppConfig {
        login_rate_limit: 3,
        ..config()
    };
    let app = api!(config);

    for tentativa in 1..=3 {
        let resp = login(&app, &moderador.email, "chute").await;

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "tentativa {tentativa} ainda está na cota"
        );
    }

    let barrado = login(&app, &moderador.email, "chute").await;

    assert_eq!(barrado.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(
        barrado.headers().get(header::RETRY_AFTER).is_some(),
        "o cliente precisa saber quando tentar de novo"
    );

    // E a senha certa não fura o limite enquanto a janela corre.
    let ainda_barrado = login(&app, &moderador.email, SENHA).await;
    assert_eq!(ainda_barrado.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[actix_web::test]
async fn a_successful_login_gives_the_quota_back() {
    // Quem só errou a senha uma vez não fica preso na janela junto com quem
    // estava tentando força bruta.
    let moderador = moderador!("a_successful_login_gives_the_quota_back");
    let config = AppConfig {
        login_rate_limit: 3,
        ..config()
    };
    let app = api!(config);

    assert_eq!(
        login(&app, &moderador.email, "chute").await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, &moderador.email, SENHA).await.status(),
        StatusCode::OK
    );

    for _ in 0..3 {
        assert_eq!(
            login(&app, &moderador.email, "chute").await.status(),
            StatusCode::UNAUTHORIZED,
            "a cota deveria ter sido devolvida no login certo"
        );
    }
}

#[actix_web::test]
async fn a_token_of_a_moderator_who_no_longer_exists_stops_working() {
    // A conta é conferida no banco a cada requisição, não só na assinatura do
    // token. Sem isso, revogar um moderador não teria efeito nenhum até o
    // token expirar sozinho.
    // O moderador vem antes do app: sem banco configurado, é ele quem pula o
    // caso, e montar o app exigiria um pool que não existe.
    let moderador = moderador!("a_token_of_a_moderator_who_no_longer_exists_stops_working");
    let config = config();
    let app = api!(config);

    let token = {
        let moderador = moderador;
        let token = create_token(moderador.id, &moderador.email, &config).unwrap();

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/auth/me")
                .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
                .to_request(),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::OK, "enquanto a conta existe");

        token
        // Aqui o Drop apaga o moderador.
    };

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
        "o mesmo token, depois da conta sumir, não vale mais"
    );
}
