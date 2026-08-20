use actix_multipart::Multipart;
use actix_web::web;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use bigdecimal::BigDecimal;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use crate::config::AppConfig;
use crate::errors::ApiError;

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

/// Apaga o que já foi gravado quando o resto da requisição não vinga. Sem isso
/// um envio interrompido no meio deixa lixo permanente no disco.
fn abort_with(upload_dir: &str, written: &[UploadedFile], error: ApiError) -> ApiError {
    let filenames: Vec<String> = written.iter().map(|f| f.filename.clone()).collect();

    cleanup_files(upload_dir, &filenames);

    error
}

pub async fn process_multipart(
    mut payload: Multipart,
    config: &AppConfig,
) -> Result<MultipartData, ApiError> {
    let mut uploaded_files: Vec<UploadedFile> = Vec::new();
    let mut fields = HashMap::new();
    let mut field_count = 0usize;
    // Soma tudo que entrou pelo corpo, arquivos e campos, para que muitos
    // arquivos pequenos não escapem do limite individual.
    let mut request_size = 0usize;

    // Ensure upload directory exists
    fs::create_dir_all(&config.upload_dir)
        .map_err(|e| ApiError::InternalError(format!("Falha ao criar diretório de upload: {e}")))?;

    while let Some(field) = payload.next().await {
        let mut field = field.map_err(|e| {
            abort_with(
                &config.upload_dir,
                &uploaded_files,
                ApiError::FileUploadError(format!("Envio malformado: {e}")),
            )
        })?;

        field_count += 1;

        if field_count > MAX_MULTIPART_FIELDS {
            return Err(abort_with(
                &config.upload_dir,
                &uploaded_files,
                ApiError::PayloadTooLarge("Requisição com campos demais".to_string()),
            ));
        }

        // Get the field name
        let field_name = field.name().to_string();

        // Check if this is a file field
        let is_file = field.content_disposition().get_filename().is_some();

        if is_file && field_name == "images" {
            if uploaded_files.len() >= config.max_files_per_request {
                return Err(abort_with(
                    &config.upload_dir,
                    &uploaded_files,
                    ApiError::PayloadTooLarge(format!(
                        "No máximo {} imagens por cadastro",
                        config.max_files_per_request
                    )),
                ));
            }

            let mut header: Vec<u8> = Vec::with_capacity(MAGIC_PREFIX_LEN);
            let mut file: Option<std::fs::File> = None;
            let mut written: Option<UploadedFile> = None;
            let mut total_size: usize = 0;

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| {
                    // O arquivo desta rodada ainda não entrou em uploaded_files.
                    if let Some(partial) = &written {
                        cleanup_files(&config.upload_dir, std::slice::from_ref(&partial.filename));
                    }

                    abort_with(
                        &config.upload_dir,
                        &uploaded_files,
                        ApiError::FileUploadError(format!("Envio interrompido: {e}")),
                    )
                })?;

                total_size += data.len();
                request_size += data.len();

                if total_size > config.max_file_size || request_size > config.max_request_size {
                    if let Some(partial) = &written {
                        cleanup_files(&config.upload_dir, std::slice::from_ref(&partial.filename));
                    }

                    return Err(abort_with(
                        &config.upload_dir,
                        &uploaded_files,
                        ApiError::PayloadTooLarge(format!(
                            "Cada imagem pode ter no máximo {} bytes",
                            config.max_file_size
                        )),
                    ));
                }

                // Enquanto o formato não estiver confirmado, nada toca o disco.
                match file.as_mut() {
                    None => {
                        header.extend_from_slice(&data);

                        if header.len() >= MAGIC_PREFIX_LEN {
                            let kind = detect_image_kind(&header).ok_or_else(|| {
                                abort_with(
                                    &config.upload_dir,
                                    &uploaded_files,
                                    ApiError::FileUploadError(
                                        "Envie imagens JPEG, PNG, GIF ou WebP".to_string(),
                                    ),
                                )
                            })?;

                            let filename = random_filename(kind.extension());
                            let filepath = PathBuf::from(&config.upload_dir).join(&filename);

                            let mut handle = std::fs::File::create(&filepath).map_err(|e| {
                                abort_with(
                                    &config.upload_dir,
                                    &uploaded_files,
                                    ApiError::InternalError(format!("Falha ao gravar upload: {e}")),
                                )
                            })?;

                            handle.write_all(&header).map_err(|e| {
                                cleanup_files(&config.upload_dir, std::slice::from_ref(&filename));

                                abort_with(
                                    &config.upload_dir,
                                    &uploaded_files,
                                    ApiError::InternalError(format!("Falha ao gravar upload: {e}")),
                                )
                            })?;

                            written = Some(UploadedFile {
                                filename: filename.clone(),
                            });
                            file = Some(handle);
                        }
                    }
                    Some(handle) => {
                        handle.write_all(&data).map_err(|e| {
                            if let Some(partial) = &written {
                                cleanup_files(
                                    &config.upload_dir,
                                    std::slice::from_ref(&partial.filename),
                                );
                            }

                            abort_with(
                                &config.upload_dir,
                                &uploaded_files,
                                ApiError::InternalError(format!("Falha ao gravar upload: {e}")),
                            )
                        })?;
                    }
                }
            }

            match written {
                Some(uploaded) => uploaded_files.push(uploaded),
                // Terminou antes dos bytes de assinatura: não é imagem válida.
                None => {
                    return Err(abort_with(
                        &config.upload_dir,
                        &uploaded_files,
                        ApiError::FileUploadError(
                            "Envie imagens JPEG, PNG, GIF ou WebP".to_string(),
                        ),
                    ));
                }
            }
        } else {
            // Process text field
            let mut field_data = Vec::new();

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| {
                    abort_with(
                        &config.upload_dir,
                        &uploaded_files,
                        ApiError::FileUploadError(format!("Envio interrompido: {e}")),
                    )
                })?;

                request_size += data.len();

                if field_data.len() + data.len() > config.max_field_size
                    || request_size > config.max_request_size
                {
                    return Err(abort_with(
                        &config.upload_dir,
                        &uploaded_files,
                        ApiError::PayloadTooLarge(format!("Campo {field_name} é grande demais")),
                    ));
                }

                field_data.extend_from_slice(&data);
            }

            // Convert bytes to string
            let field_value = String::from_utf8(field_data).map_err(|_| {
                abort_with(
                    &config.upload_dir,
                    &uploaded_files,
                    ApiError::FileUploadError(format!("Campo {field_name} não é texto válido")),
                )
            })?;

            fields.insert(field_name, field_value);
        }
    }

    Ok(MultipartData {
        files: uploaded_files,
        fields,
    })
}

pub fn cleanup_files(upload_dir: &str, filenames: &[String]) {
    for filename in filenames {
        // Defesa em profundidade: mesmo os nomes sendo gerados aqui, um caminho
        // com separador nunca deveria chegar a um remove_file.
        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            log::warn!("nome de arquivo suspeito ignorado na limpeza: {filename}");
            continue;
        }

        let filepath = PathBuf::from(upload_dir).join(filename);
        let _ = fs::remove_file(filepath); // Ignore errors on cleanup
    }
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
