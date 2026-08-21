use actix_multipart::Multipart;
use actix_web::web;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use bigdecimal::BigDecimal;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::str::FromStr;

use crate::config::AppConfig;
use crate::errors::ApiError;
use crate::storage::Storage;

/// Bytes que bastam para reconhecer os formatos aceitos. WebP é o mais longo:
/// "RIFF" + 4 bytes de tamanho + "WEBP".
const MAGIC_PREFIX_LEN: usize = 12;

/// Quantos campos o multipart pode trazer. Sem teto, um corpo com um milhão de
/// campos minúsculos passa por baixo de qualquer limite de tamanho.
const MAX_MULTIPART_FIELDS: usize = 64;

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub filename: String,
}

pub struct MultipartData {
    pub files: Vec<UploadedFile>,
    pub fields: HashMap<String, String>,
}

/// Formato de imagem reconhecido pelo conteúdo do arquivo.
///
/// A extensão vem daqui, nunca do nome que o cliente mandou. É o que separa
/// "upload de imagem" de "upload de qualquer coisa": sem isso, um .html ou um
/// .svg com script entra no diretório e, como /uploads é servido pela própria
/// API, roda como página no domínio dela — XSS armazenado, com o token do
/// moderador ao alcance. SVG fica de fora justamente por ser um documento que
/// executa script, não um bitmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageKind {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl ImageKind {
    fn extension(self) -> &'static str {
        match self {
            ImageKind::Jpeg => "jpg",
            ImageKind::Png => "png",
            ImageKind::Gif => "gif",
            ImageKind::Webp => "webp",
        }
    }

    /// Content-type gravado no objeto. Vem da detecção por conteúdo, nunca do
    /// que o cliente declarou.
    fn content_type(self) -> &'static str {
        match self {
            ImageKind::Jpeg => "image/jpeg",
            ImageKind::Png => "image/png",
            ImageKind::Gif => "image/gif",
            ImageKind::Webp => "image/webp",
        }
    }
}

/// Reconhece o formato pelos bytes iniciais.
fn detect_image_kind(header: &[u8]) -> Option<ImageKind> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageKind::Jpeg);
    }

    if header.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageKind::Png);
    }

    if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
        return Some(ImageKind::Gif);
    }

    if header.len() >= MAGIC_PREFIX_LEN && header.starts_with(b"RIFF") && &header[8..12] == b"WEBP"
    {
        return Some(ImageKind::Webp);
    }

    None
}

/// Nome de arquivo sorteado, sem relação com o que o cliente mandou.
///
/// Some com três problemas de uma vez: travessia de diretório pelo nome,
/// colisão entre envios simultâneos (o carimbo de tempo em milissegundos
/// colidia) e adivinhação de URL de imagem de cadastro ainda não aprovado.
fn random_filename(extension: &str) -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);

    let mut name = String::with_capacity(32 + 1 + extension.len());

    for byte in bytes {
        name.push_str(&format!("{byte:02x}"));
    }

    name.push('.');
    name.push_str(extension);

    name
}

/// Apaga o que já subiu quando o resto da requisição não vinga. Sem isso um
/// envio interrompido no meio deixa objeto órfão no bucket para sempre.
async fn abort_with(storage: &Storage, written: &[UploadedFile], error: ApiError) -> ApiError {
    let keys: Vec<String> = written.iter().map(|f| f.filename.clone()).collect();

    storage.remove(&keys).await;

    error
}

pub async fn process_multipart(
    mut payload: Multipart,
    config: &AppConfig,
    storage: &Storage,
) -> Result<MultipartData, ApiError> {
    let mut uploaded_files: Vec<UploadedFile> = Vec::new();
    let mut fields = HashMap::new();
    let mut field_count = 0usize;
    // Soma tudo que entrou pelo corpo, arquivos e campos, para que muitos
    // arquivos pequenos não escapem do limite individual.
    let mut request_size = 0usize;

    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(field) => field,
            Err(e) => {
                return Err(abort_with(
                    storage,
                    &uploaded_files,
                    ApiError::FileUploadError(format!("Envio malformado: {e}")),
                )
                .await);
            }
        };

        field_count += 1;

        if field_count > MAX_MULTIPART_FIELDS {
            return Err(abort_with(
                storage,
                &uploaded_files,
                ApiError::PayloadTooLarge("Requisição com campos demais".to_string()),
            )
            .await);
        }

        let field_name = field.name().to_string();
        let is_file = field.content_disposition().get_filename().is_some();

        if is_file && field_name == "images" {
            if uploaded_files.len() >= config.max_files_per_request {
                return Err(abort_with(
                    storage,
                    &uploaded_files,
                    ApiError::PayloadTooLarge(format!(
                        "No máximo {} imagens por cadastro",
                        config.max_files_per_request
                    )),
                )
                .await);
            }

            // O arquivo é montado em memória antes de subir. O teto por arquivo
            // já é aplicado enquanto os pedaços chegam, então o buffer não passa
            // de max_file_size.
            let mut buffer: Vec<u8> = Vec::new();
            let mut kind: Option<ImageKind> = None;
            let mut total_size = 0usize;

            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(abort_with(
                            storage,
                            &uploaded_files,
                            ApiError::FileUploadError(format!("Envio interrompido: {e}")),
                        )
                        .await);
                    }
                };

                total_size += data.len();
                request_size += data.len();

                if total_size > config.max_file_size || request_size > config.max_request_size {
                    return Err(abort_with(
                        storage,
                        &uploaded_files,
                        ApiError::PayloadTooLarge(format!(
                            "Cada imagem pode ter no máximo {} bytes",
                            config.max_file_size
                        )),
                    )
                    .await);
                }

                buffer.extend_from_slice(&data);

                // Formato conferido assim que há bytes de assinatura: envio de
                // arquivo que não é imagem morre antes de subir qualquer coisa.
                if kind.is_none() && buffer.len() >= MAGIC_PREFIX_LEN {
                    match detect_image_kind(&buffer) {
                        Some(detected) => kind = Some(detected),
                        None => {
                            return Err(abort_with(
                                storage,
                                &uploaded_files,
                                ApiError::FileUploadError(
                                    "Envie imagens JPEG, PNG, GIF ou WebP".to_string(),
                                ),
                            )
                            .await);
                        }
                    }
                }
            }

            // Arquivo menor que a assinatura nunca chegou a ser reconhecido.
            let Some(kind) = kind else {
                return Err(abort_with(
                    storage,
                    &uploaded_files,
                    ApiError::FileUploadError("Envie imagens JPEG, PNG, GIF ou WebP".to_string()),
                )
                .await);
            };

            let key = random_filename(kind.extension());

            if let Err(e) = storage.put_image(&key, buffer, kind.content_type()).await {
                return Err(abort_with(storage, &uploaded_files, e).await);
            }

            uploaded_files.push(UploadedFile { filename: key });
        } else {
            let mut field_data = Vec::new();

            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(abort_with(
                            storage,
                            &uploaded_files,
                            ApiError::FileUploadError(format!("Envio interrompido: {e}")),
                        )
                        .await);
                    }
                };

                request_size += data.len();

                if field_data.len() + data.len() > config.max_field_size
                    || request_size > config.max_request_size
                {
                    return Err(abort_with(
                        storage,
                        &uploaded_files,
                        ApiError::PayloadTooLarge(format!("Campo {field_name} é grande demais")),
                    )
                    .await);
                }

                field_data.extend_from_slice(&data);
            }

            let field_value = match String::from_utf8(field_data) {
                Ok(value) => value,
                Err(_) => {
                    return Err(abort_with(
                        storage,
                        &uploaded_files,
                        ApiError::FileUploadError(format!("Campo {field_name} não é texto válido")),
                    )
                    .await);
                }
            };

            fields.insert(field_name, field_value);
        }
    }

    Ok(MultipartData {
        files: uploaded_files,
        fields,
    })
}

// Helper to parse BigDecimal from string field
pub fn parse_bigdecimal(field_name: &str, value: &str) -> Result<BigDecimal, ApiError> {
    BigDecimal::from_str(value).map_err(|_| {
        ApiError::ValidationError(
            vec![(
                field_name.to_string(),
                vec![format!("{} deve ser um número válido", field_name)],
            )]
            .into_iter()
            .collect(),
        )
    })
}

/// Limite do corpo aceito pelo actix antes mesmo de chegar aos handlers.
pub fn payload_config(config: &AppConfig) -> web::PayloadConfig {
    web::PayloadConfig::new(config.max_request_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_accepted_formats() {
        assert_eq!(
            detect_image_kind(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(ImageKind::Jpeg)
        );
        assert_eq!(
            detect_image_kind(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0]),
            Some(ImageKind::Png)
        );
        assert_eq!(detect_image_kind(b"GIF89a......"), Some(ImageKind::Gif));
        assert_eq!(
            detect_image_kind(b"RIFF\x00\x00\x00\x00WEBP"),
            Some(ImageKind::Webp)
        );
    }

    #[test]
    fn rejects_documents_that_pretend_to_be_images() {
        // Os dois casos que viravam XSS armazenado quando a extensão mandava.
        assert_eq!(detect_image_kind(b"<!doctype html><script>"), None);
        assert_eq!(detect_image_kind(b"<svg xmlns=\"http:"), None);
        // E o executável, que virava distribuição de malware no domínio.
        assert_eq!(
            detect_image_kind(b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00"),
            None
        );
    }

    #[test]
    fn filename_comes_from_the_content_not_from_the_client() {
        let name = random_filename("png");

        assert!(name.ends_with(".png"));
        assert_eq!(name.len(), 32 + ".png".len());
        assert!(name[..32].chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(name, random_filename("png"), "deveria ser sorteado");
    }
}
