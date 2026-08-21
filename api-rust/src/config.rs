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
    Missing(&'static str),
    Invalid { key: &'static str, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "variável de ambiente {key} não definida"),
            ConfigError::Invalid { key, reason } => write!(f, "{key} inválida: {reason}"),
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
    fn from_env() -> Result<Self, ConfigError> {
        let bucket = env::var("S3_BUCKET").map_err(|_| ConfigError::Missing("S3_BUCKET"))?;
        let endpoint = env::var("S3_ENDPOINT").unwrap_or_default();

        // Sem base pública explícita, monta a do MinIO local: endpoint/bucket.
        let public_base_url = env::var("S3_PUBLIC_BASE_URL").unwrap_or_else(|_| {
            if endpoint.is_empty() {
                format!("https://{bucket}.s3.amazonaws.com")
            } else {
                format!("{}/{bucket}", endpoint.trim_end_matches('/'))
            }
        });

        Ok(StorageConfig {
            access_key: env::var("S3_ACCESS_KEY")
                .map_err(|_| ConfigError::Missing("S3_ACCESS_KEY"))?,
            secret_key: env::var("S3_SECRET_KEY")
                .map_err(|_| ConfigError::Missing("S3_SECRET_KEY"))?,
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
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

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
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: parse_var("SERVER_PORT", 8080u16)?,
            max_file_size: parse_var("MAX_FILE_SIZE", 5 * 1024 * 1024)?,
            max_files_per_request: parse_var("MAX_FILES_PER_REQUEST", 6usize)?,
            max_field_size: parse_var("MAX_FIELD_SIZE", 16 * 1024)?,
            max_request_size: parse_var("MAX_REQUEST_SIZE", 32 * 1024 * 1024)?,
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            storage: StorageConfig::from_env()?,
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
