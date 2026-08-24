use std::env;
use std::fmt;

/// Segredos de exemplo que circulam no repositório e em tutoriais. Se um deles
/// chegar à produção, qualquer pessoa forja um token de moderador.
const WEAK_JWT_SECRETS: &[&str] = &[
    "dev-secret-trocar-em-producao",
    "troque-por-um-segredo-longo-e-aleatorio",
    "secret",
    "changeme",
];

/// Tamanho mínimo do segredo do JWT. HS256 deriva a chave do segredo cru, então
/// um segredo curto é quebrável por força bruta offline a partir de um token.
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug)]
pub enum ConfigError {
    /// Todas as obrigatórias que faltam, de uma vez.
    ///
    /// Vem como lista porque a alternativa é quem está subindo o serviço
    /// descobrir uma variável por deploy: corrige, sobe de novo, descobre a
    /// seguinte. Numa primeira instalação isso são cinco rodadas.
    Missing(Vec<&'static str>),
    Invalid {
        key: &'static str,
        reason: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(keys) => write!(
                f,
                "variáve{} de ambiente não definida{}: {}",
                if keys.len() == 1 { "l" } else { "is" },
                if keys.len() == 1 { "" } else { "s" },
                keys.join(", ")
            ),
            ConfigError::Invalid { key, reason } => write!(f, "{key} inválida: {reason}"),
        }
    }
}

/// Coletor das variáveis obrigatórias que faltam.
#[derive(Default)]
struct Required(Vec<&'static str>);

impl Required {
    /// Lê uma obrigatória. Ausente, anota o nome e devolve None — a leitura
    /// continua para que a próxima ausência também seja anotada.
    fn get(&mut self, key: &'static str) -> Option<String> {
        match env::var(key) {
            Ok(value) => Some(value),
            Err(_) => {
                self.0.push(key);

                None
            }
        }
    }

    fn into_result(self) -> Result<(), ConfigError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Missing(self.0))
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    /// Teto por arquivo enviado.
    pub max_file_size: usize,
    /// Quantos arquivos um cadastro pode mandar de uma vez.
    pub max_files_per_request: usize,
    /// Teto por campo de texto do multipart — sem ele um campo "about" de 2 GB
    /// é lido inteiro para a memória antes de qualquer validação.
    pub max_field_size: usize,
    /// Teto do corpo inteiro da requisição, somando arquivos e campos.
    pub max_request_size: usize,
    pub base_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_hours: i64,
    /// Origens autorizadas a chamar a API pelo navegador.
    pub cors_allowed_origins: Vec<String>,
    /// Só ligue atrás de um proxy que reescreve X-Forwarded-For. Ligado sem
    /// proxy, qualquer cliente forja o próprio IP e escapa do rate limit.
    pub trust_proxy: bool,
    /// Tentativas de login por IP dentro da janela.
    pub login_rate_limit: u32,
    /// Cadastros de coletivo por IP dentro da janela.
    pub submission_rate_limit: u32,
    /// Janela dos dois limites acima, em segundos.
    pub rate_limit_window_secs: u64,
    pub storage: StorageConfig,
}

/// Bucket compatível com S3 onde as imagens ficam. MinIO no ambiente local,
/// S3/R2/Spaces em produção — muda só endpoint e credenciais.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Vazio significa AWS S3 de verdade, que dispensa endpoint explícito.
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    /// Como o navegador chega nas imagens. Separado do endpoint porque em
    /// produção costuma ser um CDN na frente do bucket.
    pub public_base_url: String,
    /// MinIO endereça por caminho (host/bucket/chave); a AWS, por subdomínio.
    pub force_path_style: bool,
}

impl StorageConfig {
    fn from_env(required: &mut Required) -> Result<Self, ConfigError> {
        let bucket = required.get("S3_BUCKET");
        let access_key = required.get("S3_ACCESS_KEY");
        let secret_key = required.get("S3_SECRET_KEY");

        let endpoint = env::var("S3_ENDPOINT").unwrap_or_default();

        // A base pública deriva do bucket, então sem ele não há o que derivar.
        // O nome já foi anotado; aqui só se evita inventar um valor.
        let bucket = bucket.unwrap_or_default();

        // Sem base pública explícita, monta a do MinIO local: endpoint/bucket.
        let public_base_url = env::var("S3_PUBLIC_BASE_URL").unwrap_or_else(|_| {
            if endpoint.is_empty() {
                format!("https://{bucket}.s3.amazonaws.com")
            } else {
                format!("{}/{bucket}", endpoint.trim_end_matches('/'))
            }
        });

        Ok(StorageConfig {
            access_key: access_key.unwrap_or_default(),
            secret_key: secret_key.unwrap_or_default(),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            force_path_style: parse_var("S3_FORCE_PATH_STYLE", !endpoint.is_empty())?,
            endpoint,
            bucket,
            public_base_url,
        })
    }
}

fn parse_var<T: std::str::FromStr>(key: &'static str, default: T) -> Result<T, ConfigError> {
    match env::var(key) {
        Err(_) => Ok(default),
        // Valor presente mas ilegível vira erro em vez de cair no default em
        // silêncio: MAX_FILE_SIZE="10 MB" não pode virar 10 MB por acidente.
        Ok(raw) => raw.parse().map_err(|_| ConfigError::Invalid {
            key,
            reason: format!("não consegui interpretar {raw:?}"),
        }),
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Primeiro junta tudo que falta, depois valida o que veio: quem está
        // subindo o serviço recebe a lista inteira de uma vez.
        let mut required = Required::default();

        let jwt_secret = required.get("JWT_SECRET");
        let database_url = required.get("DATABASE_URL");
        let storage = StorageConfig::from_env(&mut required)?;

        required.into_result()?;

        let jwt_secret = jwt_secret.expect("into_result já teria falhado");
        let database_url = database_url.expect("into_result já teria falhado");

        if jwt_secret.chars().count() < MIN_JWT_SECRET_LEN {
            return Err(ConfigError::Invalid {
                key: "JWT_SECRET",
                reason: format!("precisa de ao menos {MIN_JWT_SECRET_LEN} caracteres"),
            });
        }

        if WEAK_JWT_SECRETS.contains(&jwt_secret.as_str()) {
            return Err(ConfigError::Invalid {
                key: "JWT_SECRET",
                reason: "é um segredo de exemplo, público no repositório".to_string(),
            });
        }

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
            .split(',')
            .map(|origin| origin.trim().to_string())
            .filter(|origin| !origin.is_empty())
            .collect::<Vec<_>>();

        if cors_allowed_origins.iter().any(|origin| origin == "*") {
            return Err(ConfigError::Invalid {
                key: "CORS_ALLOWED_ORIGINS",
                reason: "\"*\" abriria a API para qualquer site; liste as origens".to_string(),
            });
        }

        Ok(AppConfig {
            database_url,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: parse_var("SERVER_PORT", 8080u16)?,
            max_file_size: parse_var("MAX_FILE_SIZE", 5 * 1024 * 1024)?,
            max_files_per_request: parse_var("MAX_FILES_PER_REQUEST", 6usize)?,
            max_field_size: parse_var("MAX_FIELD_SIZE", 16 * 1024)?,
            max_request_size: parse_var("MAX_REQUEST_SIZE", 32 * 1024 * 1024)?,
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            storage,
            jwt_secret,
            jwt_ttl_hours: parse_var("JWT_TTL_HOURS", 8i64)?,
            cors_allowed_origins,
            trust_proxy: matches!(
                env::var("TRUST_PROXY").unwrap_or_default().as_str(),
                "1" | "true" | "TRUE"
            ),
            login_rate_limit: parse_var("LOGIN_RATE_LIMIT", 5u32)?,
            submission_rate_limit: parse_var("SUBMISSION_RATE_LIMIT", 10u32)?,
            rate_limit_window_secs: parse_var("RATE_LIMIT_WINDOW_SECS", 300u64)?,
        })
    }
}

/// Configuração completa, com valores plausíveis, para os testes do crate.
///
/// Existe para que um campo novo em AppConfig apareça num lugar só, e não em
/// cada módulo que precisa de uma configuração para exercitar outra coisa.
#[cfg(test)]
impl AppConfig {
    pub(crate) fn sample() -> Self {
        AppConfig {
            database_url: String::new(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            max_file_size: 5 * 1024 * 1024,
            max_files_per_request: 6,
            max_field_size: 16 * 1024,
            max_request_size: 32 * 1024 * 1024,
            base_url: "http://localhost:8080".to_string(),
            jwt_secret: "segredo-de-teste-suficientemente-longo-1234".to_string(),
            jwt_ttl_hours: 8,
            cors_allowed_origins: vec!["http://localhost:5173".to_string()],
            trust_proxy: false,
            login_rate_limit: 5,
            submission_rate_limit: 10,
            rate_limit_window_secs: 300,
            storage: StorageConfig {
                endpoint: "http://127.0.0.1:9000".to_string(),
                region: "us-east-1".to_string(),
                bucket: "rave-map".to_string(),
                access_key: "chave".to_string(),
                secret_key: "segredo".to_string(),
                public_base_url: "http://127.0.0.1:9000/rave-map".to_string(),
                force_path_style: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Todas as variáveis que `from_env` enxerga. O teste limpa a lista
    /// inteira antes de montar o cenário para que o ambiente de quem roda
    /// `cargo test` não vaze para dentro do caso.
    const ALL_KEYS: &[&str] = &[
        "JWT_SECRET",
        "JWT_TTL_HOURS",
        "DATABASE_URL",
        "SERVER_HOST",
        "SERVER_PORT",
        "MAX_FILE_SIZE",
        "MAX_FILES_PER_REQUEST",
        "MAX_FIELD_SIZE",
        "MAX_REQUEST_SIZE",
        "BASE_URL",
        "CORS_ALLOWED_ORIGINS",
        "TRUST_PROXY",
        "LOGIN_RATE_LIMIT",
        "SUBMISSION_RATE_LIMIT",
        "RATE_LIMIT_WINDOW_SECS",
        "S3_BUCKET",
        "S3_ENDPOINT",
        "S3_PUBLIC_BASE_URL",
        "S3_ACCESS_KEY",
        "S3_SECRET_KEY",
        "S3_REGION",
        "S3_FORCE_PATH_STYLE",
    ];

    /// O ambiente é global ao processo e o `cargo test` roda os casos em
    /// paralelo: sem este lock, um teste apagaria a variável que o outro
    /// acabou de escrever.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const STRONG_SECRET: &str = "segredo-de-teste-suficientemente-longo-1234";

    /// Cenário mínimo para `from_env` chegar ao fim.
    fn minimal() -> Vec<(&'static str, &'static str)> {
        vec![
            ("JWT_SECRET", STRONG_SECRET),
            ("DATABASE_URL", "postgres://user:senha@localhost/rave_map"),
            ("S3_BUCKET", "rave-map"),
            ("S3_ACCESS_KEY", "chave"),
            ("S3_SECRET_KEY", "segredo"),
        ]
    }

    fn with_env<T>(pairs: &[(&str, &str)], f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        for key in ALL_KEYS {
            env::remove_var(key);
        }

        for (key, value) in pairs {
            env::set_var(key, value);
        }

        let result = f();

        for key in ALL_KEYS {
            env::remove_var(key);
        }

        result
    }

    /// Monta o cenário mínimo com um ajuste em cima.
    fn from_env_with(extra: &[(&str, &str)]) -> Result<AppConfig, ConfigError> {
        let mut pairs = minimal();

        for (key, value) in extra {
            pairs.retain(|(existing, _)| existing != key);
            pairs.push((key, value));
        }

        with_env(&pairs, AppConfig::from_env)
    }

    fn invalid_key(result: Result<AppConfig, ConfigError>) -> &'static str {
        match result {
            Err(ConfigError::Invalid { key, .. }) => key,
            other => panic!("esperava ConfigError::Invalid, veio {other:?}"),
        }
    }

    fn missing_keys(result: Result<AppConfig, ConfigError>) -> Vec<&'static str> {
        match result {
            Err(ConfigError::Missing(keys)) => keys,
            other => panic!("esperava ConfigError::Missing, veio {other:?}"),
        }
    }

    #[test]
    fn accepts_the_minimal_environment() {
        let config = from_env_with(&[]).expect("o cenário mínimo deveria bastar");

        assert_eq!(config.server_port, 8080);
        assert_eq!(config.jwt_ttl_hours, 8);
        assert!(!config.trust_proxy, "proxy só é confiado quando pedido");
        assert_eq!(config.cors_allowed_origins, vec!["http://localhost:5173"]);
    }

    #[test]
    fn refuses_to_start_without_the_required_variables() {
        for chave in [
            "JWT_SECRET",
            "DATABASE_URL",
            "S3_BUCKET",
            "S3_ACCESS_KEY",
            "S3_SECRET_KEY",
        ] {
            let sem_ela: Vec<_> = minimal()
                .into_iter()
                .filter(|(key, _)| *key != chave)
                .collect();

            assert_eq!(
                missing_keys(with_env(&sem_ela, AppConfig::from_env)),
                vec![chave],
                "faltando {chave}"
            );
        }
    }

    #[test]
    fn reports_every_missing_variable_at_once() {
        // Uma por vez significa quem instala descobrir uma variável por
        // deploy: corrige, sobe de novo, descobre a seguinte.
        let faltando = missing_keys(with_env(&[], AppConfig::from_env));

        assert_eq!(
            faltando,
            vec![
                "JWT_SECRET",
                "DATABASE_URL",
                "S3_BUCKET",
                "S3_ACCESS_KEY",
                "S3_SECRET_KEY",
            ]
        );
    }

    #[test]
    fn the_missing_message_names_all_of_them() {
        let erro = with_env(&[], AppConfig::from_env).unwrap_err();
        let mensagem = erro.to_string();

        for chave in ["JWT_SECRET", "DATABASE_URL", "S3_BUCKET"] {
            assert!(
                mensagem.contains(chave),
                "{chave} ficou de fora: {mensagem}"
            );
        }
    }

    #[test]
    fn a_missing_variable_never_becomes_an_empty_default() {
        // O coletor segue lendo depois de anotar uma ausência, e o risco disso
        // é um valor vazio escapar como se fosse configuração válida.
        let sem_bucket: Vec<_> = minimal()
            .into_iter()
            .filter(|(key, _)| *key != "S3_BUCKET")
            .collect();

        assert!(
            with_env(&sem_bucket, AppConfig::from_env).is_err(),
            "bucket vazio não pode virar configuração aceita"
        );
    }

    #[test]
    fn refuses_a_short_jwt_secret() {
        // Um segredo curto de HS256 cai por força bruta offline a partir de um
        // único token emitido.
        assert_eq!(
            invalid_key(from_env_with(&[("JWT_SECRET", "curto-demais")])),
            "JWT_SECRET"
        );
    }

    #[test]
    fn refuses_the_example_secrets_that_are_public_in_the_repository() {
        for weak in WEAK_JWT_SECRETS {
            assert_eq!(
                invalid_key(from_env_with(&[("JWT_SECRET", weak)])),
                "JWT_SECRET",
                "{weak:?} está no repositório e não pode virar chave de produção"
            );
        }
    }

    #[test]
    fn refuses_a_wildcard_cors_origin() {
        assert_eq!(
            invalid_key(from_env_with(&[("CORS_ALLOWED_ORIGINS", "*")])),
            "CORS_ALLOWED_ORIGINS"
        );

        assert_eq!(
            invalid_key(from_env_with(&[(
                "CORS_ALLOWED_ORIGINS",
                "https://mapa.exemplo.com, *"
            )])),
            "CORS_ALLOWED_ORIGINS",
            "o curinga no meio da lista vale o mesmo que sozinho"
        );
    }

    #[test]
    fn splits_and_trims_the_cors_origin_list() {
        let config = from_env_with(&[(
            "CORS_ALLOWED_ORIGINS",
            " https://mapa.exemplo.com , http://localhost:5173 ,, ",
        )])
        .unwrap();

        assert_eq!(
            config.cors_allowed_origins,
            vec!["https://mapa.exemplo.com", "http://localhost:5173"],
            "entrada vazia entre vírgulas não vira origem"
        );
    }

    #[test]
    fn an_unreadable_number_is_an_error_not_a_silent_default() {
        // O caso que motivou o parse_var: MAX_FILE_SIZE="10 MB" caindo no
        // default sem ninguém perceber.
        assert_eq!(
            invalid_key(from_env_with(&[("MAX_FILE_SIZE", "10 MB")])),
            "MAX_FILE_SIZE"
        );

        assert_eq!(
            invalid_key(from_env_with(&[("SERVER_PORT", "oitenta")])),
            "SERVER_PORT"
        );

        assert_eq!(
            invalid_key(from_env_with(&[("SERVER_PORT", "70000")])),
            "SERVER_PORT",
            "porta fora da faixa de u16 não pode virar 8080 em silêncio"
        );
    }

    #[test]
    fn trust_proxy_only_on_an_explicit_yes() {
        for ligado in ["1", "true", "TRUE"] {
            assert!(
                from_env_with(&[("TRUST_PROXY", ligado)])
                    .unwrap()
                    .trust_proxy,
                "{ligado:?} deveria ligar"
            );
        }

        // Qualquer outra coisa fica desligado: ligado sem proxy na frente,
        // qualquer cliente forja o próprio IP e escapa do rate limit.
        for desligado in ["", "0", "false", "sim", "yes"] {
            assert!(
                !from_env_with(&[("TRUST_PROXY", desligado)])
                    .unwrap()
                    .trust_proxy,
                "{desligado:?} não deveria ligar"
            );
        }
    }

    #[test]
    fn storage_public_url_defaults_to_the_local_bucket() {
        let config = from_env_with(&[("S3_ENDPOINT", "http://localhost:9000/")]).unwrap();

        assert_eq!(
            config.storage.public_base_url,
            "http://localhost:9000/rave-map"
        );
        assert!(
            config.storage.force_path_style,
            "endpoint próprio endereça por caminho"
        );
    }

    #[test]
    fn storage_public_url_defaults_to_aws_without_an_endpoint() {
        let config = from_env_with(&[]).unwrap();

        assert_eq!(
            config.storage.public_base_url,
            "https://rave-map.s3.amazonaws.com"
        );
        assert!(
            !config.storage.force_path_style,
            "a AWS endereça por subdomínio"
        );
    }

    #[test]
    fn an_explicit_public_url_wins_over_the_derived_one() {
        // É o caso de produção: CDN na frente do bucket.
        let config = from_env_with(&[
            ("S3_ENDPOINT", "http://rustfs:9000"),
            ("S3_PUBLIC_BASE_URL", "https://cdn.exemplo.com"),
        ])
        .unwrap();

        assert_eq!(config.storage.public_base_url, "https://cdn.exemplo.com");
    }

    #[test]
    fn the_error_message_never_carries_the_secret() {
        // A mensagem vai para o stderr da subida, que costuma acabar em
        // agregador de log de terceiro.
        let erro = from_env_with(&[("JWT_SECRET", "curto")]).unwrap_err();

        assert!(
            !erro.to_string().contains("curto"),
            "a mensagem não pode repetir o segredo: {erro}"
        );
    }
}
