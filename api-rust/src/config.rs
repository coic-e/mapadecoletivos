use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub upload_dir: String,
    pub max_file_size: usize,
    pub base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(AppConfig {
            database_url: env::var("DATABASE_URL")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string()),
            max_file_size: env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .unwrap_or(10485760),
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        })
    }
}
