//! Peças compartilhadas pelos testes de integração.
//!
//! Cada arquivo em `tests/` é um crate separado, então isto entra por
//! `mod common;` — e o compilador reclama do que um deles não usa. Daí o
//! `#![allow(dead_code)]`.
#![allow(dead_code)]

use std::sync::Mutex;

use api_rust::config::AppConfig;

/// O ambiente é global ao processo e os casos rodam em paralelo.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Configuração completa, montada pelo mesmo caminho do servidor de verdade.
///
/// Passa por `AppConfig::from_env` de propósito: assim o teste também confere
/// que o cenário que ele monta é um cenário que a API aceitaria subir.
pub fn config() -> AppConfig {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let pairs = [
        ("JWT_SECRET", "segredo-de-teste-suficientemente-longo-1234"),
        ("DATABASE_URL", "postgres://ninguem@localhost/nao-usado"),
        ("CORS_ALLOWED_ORIGINS", "https://mapa.exemplo.com"),
        ("S3_BUCKET", "rave-map"),
        ("S3_ACCESS_KEY", "chave"),
        ("S3_SECRET_KEY", "segredo"),
        ("S3_ENDPOINT", "http://127.0.0.1:9000"),
    ];

    for (key, value) in pairs {
        std::env::set_var(key, value);
    }

    let config = AppConfig::from_env().expect("o cenário de teste deveria ser válido");

    for (key, _) in pairs {
        std::env::remove_var(key);
    }

    config
}

/// A origem que `config()` autoriza.
pub const ALLOWED_ORIGIN: &str = "https://mapa.exemplo.com";

/// Pool que nunca abre conexão.
///
/// O `establish_connection_pool` de produção conecta na hora e derruba a subida
/// se o banco não responde — o que é o certo lá, e inviável aqui. Este usa
/// `build_unchecked`, que só guarda a configuração, e um timeout curto: se
/// algum caso encostar no banco sem querer, ele falha em um segundo em vez de
/// pendurar a suíte.
pub fn pool_that_never_connects() -> api_rust::db::DbPool {
    use diesel::r2d2::{ConnectionManager, Pool};

    let manager =
        ConnectionManager::<diesel::PgConnection>::new("postgres://ninguem@127.0.0.1:1/nao-usado");

    Pool::builder()
        .max_size(1)
        .min_idle(Some(0))
        .connection_timeout(std::time::Duration::from_secs(1))
        .build_unchecked(manager)
}

/// Store em memória para os testes de HTTP, que não exercitam o bucket.
pub struct NoopStore;

#[async_trait::async_trait]
impl api_rust::storage::ImageStore for NoopStore {
    async fn put_image(
        &self,
        _key: &str,
        _bytes: Vec<u8>,
        _content_type: &'static str,
    ) -> Result<(), api_rust::errors::ApiError> {
        Ok(())
    }

    async fn remove(&self, _keys: &[String]) {}

    async fn check(&self) -> Result<(), api_rust::errors::ApiError> {
        Ok(())
    }

    fn public_url(&self, key: &str) -> String {
        format!("http://fake/{key}")
    }
}

// --------------------------------------------------------------- banco

use std::sync::OnceLock;

use api_rust::db::{DbConnection, DbPool};

/// Pool para os testes que precisam de Postgres de verdade.
///
/// `None` quando `TEST_DATABASE_URL` não está definida — é o caso normal de
/// quem só roda `cargo test` sem subir o compose, e nesse caso os testes de
/// banco se anunciam como pulados em vez de falhar.
///
/// A variável é separada de `DATABASE_URL` de propósito, e apontar as duas
/// para o mesmo lugar é recusado: estes testes apagam tabela para montar o
/// cenário, e ninguém quer descobrir isso pelo banco de desenvolvimento.
pub fn pool() -> Option<&'static DbPool> {
    static POOL: OnceLock<Option<DbPool>> = OnceLock::new();

    POOL.get_or_init(|| {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;

        if let Ok(desenvolvimento) = std::env::var("DATABASE_URL") {
            assert_ne!(
                url, desenvolvimento,
                "TEST_DATABASE_URL não pode ser o banco de desenvolvimento: \
                 os testes apagam tabela"
            );
        }

        api_rust::migrations::run(&url).expect("não consegui migrar o banco de teste");

        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(url);

        Some(
            diesel::r2d2::Pool::builder()
                .max_size(4)
                .build(manager)
                .expect("não consegui abrir o pool de teste"),
        )
    })
    .as_ref()
}

/// Trava de acesso ao banco.
///
/// Os casos rodam em paralelo dentro do mesmo processo e alguns precisam da
/// tabela num estado conhecido — a semente do primeiro moderador conta a
/// tabela inteira, por exemplo. Uma transação não isola disso: um caso que
/// confirma sua escrita passa a ser visto pelos outros no mesmo instante.
///
/// Todo teste que toca no banco pega esta trava, seja pela `with_database` ou
/// segurando o guarda pela vida do caso.
pub fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    static DB_LOCK: Mutex<()> = Mutex::new(());

    DB_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Roda o corpo do teste numa transação que nunca é confirmada.
///
/// Cada caso monta o cenário que quer e o banco volta ao que era ao final,
/// então a ordem em que os testes rodam não importa.
///
/// Devolve `false` quando não há banco configurado, para o caso se anunciar
/// como pulado.
pub fn with_database(nome: &str, corpo: impl FnOnce(&mut DbConnection)) -> bool {
    use diesel::connection::Connection;

    let Some(pool) = pool() else {
        eprintln!("pulando {nome}: defina TEST_DATABASE_URL para rodar os testes de banco");

        return false;
    };

    let _guard = db_guard();
    let mut conn = pool.get().expect("não consegui pegar conexão do pool");

    conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
        corpo(conn);

        Ok(())
    });

    true
}

/// Esvazia as tabelas, na ordem que as chaves estrangeiras permitem.
///
/// Só faz sentido dentro de `with_database`, onde a transação é desfeita no
/// fim: aqui é montagem de cenário, não faxina de verdade.
pub fn clear(conn: &mut DbConnection) {
    use db_types::schema::{admins, edit_requests, images, organizations};
    use diesel::prelude::*;

    diesel::delete(edit_requests::table).execute(conn).unwrap();
    diesel::delete(images::table).execute(conn).unwrap();
    diesel::delete(organizations::table).execute(conn).unwrap();
    diesel::delete(admins::table).execute(conn).unwrap();
}

/// Um moderador, para as ações que registram quem decidiu.
pub fn admin(conn: &mut DbConnection) -> db_types::admin::Admin {
    use api_rust::domains::admins::repository::AdminRepository;

    AdminRepository::create(
        conn,
        &db_types::admin::NewAdmin {
            name: "Moderação".to_string(),
            email: format!("mod-{}@exemplo.com", nanos()),
            password_hash: api_rust::auth::hash_password("uma-senha-de-teste").unwrap(),
        },
    )
    .expect("deveria criar o moderador")
}

/// Cadastro válido, pronto para ser criado.
pub fn new_organization(name: &str) -> db_types::organization::NewOrganization {
    use bigdecimal::BigDecimal;
    use db_types::organization::{slugify, NewOrganization};
    use std::str::FromStr;

    NewOrganization {
        name: name.to_string(),
        latitude: BigDecimal::from_str("-30.0346").unwrap(),
        longitude: BigDecimal::from_str("-51.2177").unwrap(),
        type_: "Coletivo".to_string(),
        city: "Porto Alegre".to_string(),
        uf: "RS".to_string(),
        email: "contato@exemplo.com".to_string(),
        about: "Um coletivo de techno em Porto Alegre".to_string(),
        genres: vec!["Techno".to_string()],
        address: None,
        instagram: Some("@coletivo".to_string()),
        soundcloud: None,
        bandcamp: None,
        youtube: None,
        spotify: None,
        website: None,
        is_active: true,
        frequency: Some("Mensal".to_string()),
        slug: slugify(name),
    }
}

/// Sufixo para e-mails que precisam ser únicos dentro de um mesmo teste.
fn nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default()
}
