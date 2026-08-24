//! Armazenamento das imagens num bucket compatível com S3.
//!
//! O disco do container não guarda mais nada: ele é efêmero, some a cada
//! deploy e não é compartilhado entre réplicas. MinIO cobre o ambiente local e
//! S3, R2 ou Spaces cobrem produção — o mesmo código atende os dois, mudando
//! só endpoint e credenciais.
use async_trait::async_trait;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};

use crate::config::StorageConfig;
use crate::errors::ApiError;

/// O que o resto da API precisa de um lugar para guardar imagem.
///
/// Existe para que o caminho de upload possa ser exercitado sem bucket nenhum.
/// Antes disto, testar `process_multipart` significava apontar um cliente S3
/// para um endereço morto e torcer para o caso de teste morrer antes de tocar
/// na rede — o que deixava sem cobertura justamente os caminhos que sobem
/// arquivo, como o teto de imagens por cadastro.
#[async_trait]
pub trait ImageStore: Send + Sync {
    async fn put_image(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &'static str,
    ) -> Result<(), ApiError>;

    /// Remove objetos de um cadastro que não vingou. Não devolve erro: o que
    /// motivou a limpeza é o que importa para quem chamou, e objeto órfão é
    /// problema de faxina, não de correção.
    async fn remove(&self, keys: &[String]);

    /// Confere que o destino responde e que as credenciais valem.
    async fn check(&self) -> Result<(), ApiError>;

    /// URL pública de um objeto, que é o que vai para o navegador.
    fn public_url(&self, key: &str) -> String;
}

/// O ponteiro que circula pelo app. `Arc` porque o actix clona o `App` por
/// worker e o trait object precisa ser compartilhado, não duplicado.
pub type SharedStore = std::sync::Arc<dyn ImageStore>;

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
}

/// Sobe imagem para um bucket compatível com S3.
///
/// O content-type vem da detecção por magic bytes, nunca do que o cliente
/// declarou: é o que impede um arquivo qualquer de ser servido como documento
/// a partir do domínio do bucket.
#[async_trait]
impl ImageStore for Storage {
    async fn put_image(
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

    async fn remove(&self, keys: &[String]) {
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

    /// HeadBucket é a operação mais barata que exercita autenticação e
    /// existência do bucket de uma vez — não lista nada nem transfere objeto.
    async fn check(&self) -> Result<(), ApiError> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| ApiError::InternalError(format!("bucket inacessível: {e}")))?;

        Ok(())
    }

    fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.public_base_url, key)
    }
}

/// Store de mentira para os testes: guarda em memória o que subiria.
///
/// Sem ele, exercitar o caminho de upload exigia um bucket de verdade — e os
/// casos que só acontecem *depois* de um arquivo subir ficavam sem cobertura.
#[cfg(test)]
pub mod fake {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct FakeStore {
        objetos: Mutex<Vec<(String, &'static str, usize)>>,
        /// Quando ligado, todo `put_image` falha — é como se testa que um envio
        /// interrompido no meio não deixa objeto órfão.
        pub falha_ao_subir: bool,
    }

    impl FakeStore {
        pub fn new() -> Self {
            FakeStore::default()
        }

        pub fn que_falha() -> Self {
            FakeStore {
                falha_ao_subir: true,
                ..FakeStore::default()
            }
        }

        /// As chaves que continuam guardadas, em ordem de chegada.
        pub fn chaves(&self) -> Vec<String> {
            self.objetos
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .iter()
                .map(|(chave, _, _)| chave.clone())
                .collect()
        }

        /// O content-type gravado num objeto, para conferir que veio da
        /// detecção por conteúdo e não do que o cliente declarou.
        pub fn content_type(&self, chave: &str) -> Option<&'static str> {
            self.objetos
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .iter()
                .find(|(k, _, _)| k == chave)
                .map(|(_, tipo, _)| *tipo)
        }
    }

    #[async_trait]
    impl ImageStore for FakeStore {
        async fn put_image(
            &self,
            key: &str,
            bytes: Vec<u8>,
            content_type: &'static str,
        ) -> Result<(), ApiError> {
            if self.falha_ao_subir {
                return Err(ApiError::InternalError("bucket fora do ar".to_string()));
            }

            self.objetos
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push((key.to_string(), content_type, bytes.len()));

            Ok(())
        }

        async fn remove(&self, keys: &[String]) {
            self.objetos
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .retain(|(chave, _, _)| !keys.contains(chave));
        }

        async fn check(&self) -> Result<(), ApiError> {
            Ok(())
        }

        fn public_url(&self, key: &str) -> String {
            format!("http://fake/{key}")
        }
    }
}
