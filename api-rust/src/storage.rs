//! Armazenamento das imagens num bucket compatível com S3.
//!
//! O disco do container não guarda mais nada: ele é efêmero, some a cada
//! deploy e não é compartilhado entre réplicas. MinIO cobre o ambiente local e
//! S3, R2 ou Spaces cobrem produção — o mesmo código atende os dois, mudando
//! só endpoint e credenciais.
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};

use crate::config::StorageConfig;
use crate::errors::ApiError;

#[derive(Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
    /// Base pública das imagens, como o navegador as acessa. Separada do
    /// endpoint porque em produção costuma ser um CDN na frente do bucket.
    public_base_url: String,
}

impl Storage {
    pub fn new(config: &StorageConfig) -> Self {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "rave-map",
        );

        let mut builder = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            // MinIO responde em caminho (host/bucket/chave), não em subdomínio.
            // Na AWS o padrão é subdomínio, então isto vem da configuração.
            .force_path_style(config.force_path_style);

        if !config.endpoint.is_empty() {
            builder = builder.endpoint_url(&config.endpoint);
        }

        Storage {
            client: Client::from_conf(builder.build()),
            bucket: config.bucket.clone(),
            public_base_url: config.public_base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Sobe uma imagem já validada.
    ///
    /// O content-type vem da detecção por magic bytes, nunca do que o cliente
    /// declarou: é o que impede um arquivo qualquer de ser servido como
    /// documento a partir do domínio do bucket.
    pub async fn put_image(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &'static str,
    ) -> Result<(), ApiError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            // O nome do objeto é sorteado e o conteúdo nunca muda, então pode
            // ser cacheado para sempre.
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("Falha ao subir imagem: {e}")))?;

        Ok(())
    }

    /// Remove objetos de um cadastro que não vingou. Falha aqui não derruba a
    /// requisição: o erro que motivou a limpeza é o que importa para quem
    /// chamou, e objeto órfão é problema de faxina, não de correção.
    pub async fn remove(&self, keys: &[String]) {
        for key in keys {
            if let Err(e) = self
                .client
                .delete_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
            {
                log::warn!("não consegui apagar {key} do bucket: {e}");
            }
        }
    }

    /// URL pública de um objeto, que é o que vai para o navegador.
    pub fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.public_base_url, key)
    }
}
