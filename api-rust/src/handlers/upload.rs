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

#[derive(Debug)]
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
    fn a_file_shorter_than_the_signature_is_not_recognised() {
        // Não dá para decidir sem os bytes de assinatura, e "não decidi" tem
        // que valer recusa: o process_multipart trata isto como erro.
        assert_eq!(detect_image_kind(b"RIFF"), None);
        assert_eq!(detect_image_kind(b""), None);
    }

    #[test]
    fn a_webp_header_needs_the_webp_marker_not_just_riff() {
        // RIFF sozinho é contêiner de áudio (WAV, AVI), não de imagem.
        assert_eq!(detect_image_kind(b"RIFF\x00\x00\x00\x00WAVE"), None);
    }

    #[test]
    fn the_extension_and_the_content_type_walk_together() {
        // O content-type gravado no objeto vem daqui, nunca do que o cliente
        // declarou: é o que impede o bucket de servir um arquivo qualquer como
        // documento.
        for (kind, extension, content_type) in [
            (ImageKind::Jpeg, "jpg", "image/jpeg"),
            (ImageKind::Png, "png", "image/png"),
            (ImageKind::Gif, "gif", "image/gif"),
            (ImageKind::Webp, "webp", "image/webp"),
        ] {
            assert_eq!(kind.extension(), extension);
            assert_eq!(kind.content_type(), content_type);
        }
    }

    #[test]
    fn parses_coordinates_and_refuses_what_is_not_a_number() {
        assert_eq!(
            parse_bigdecimal("latitude", "-30.0346").unwrap(),
            BigDecimal::from_str("-30.0346").unwrap()
        );

        for lixo in ["", "abc", "-30,03", "30.0.1", "NaN"] {
            let erro = parse_bigdecimal("latitude", lixo)
                .expect_err(&format!("{lixo:?} não é coordenada"));

            assert!(
                matches!(erro, ApiError::ValidationError(_)),
                "{lixo:?} deveria virar erro de validação, não 500"
            );
        }
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

/// Testes do parser de multipart.
///
/// Todos passam por um `Storage` apontado para um endereço morto: os caminhos
/// exercitados aqui recusam a requisição antes de qualquer objeto subir, então
/// a limpeza que eles disparam não tem chave nenhuma para apagar e não sai da
/// máquina. Um caso que chegasse a subir de verdade pertence aos testes de
/// integração, não a este módulo.
#[cfg(test)]
mod multipart_tests {
    use super::*;
    use actix_web::{test, FromRequest};

    const BOUNDARY: &str = "----------------------------fronteira";

    fn config() -> AppConfig {
        AppConfig::sample()
    }

    fn storage(config: &AppConfig) -> Storage {
        Storage::new(&config.storage)
    }

    /// Monta o corpo de um multipart. Cada parte é (cabeçalho da disposição,
    /// bytes do conteúdo).
    fn body(parts: &[(String, Vec<u8>)]) -> Vec<u8> {
        let mut out = Vec::new();

        for (disposition, content) in parts {
            out.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
            out.extend_from_slice(format!("Content-Disposition: {disposition}\r\n\r\n").as_bytes());
            out.extend_from_slice(content);
            out.extend_from_slice(b"\r\n");
        }

        out.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());

        out
    }

    fn text_part(name: &str, value: &str) -> (String, Vec<u8>) {
        (
            format!("form-data; name=\"{name}\""),
            value.as_bytes().to_vec(),
        )
    }

    fn file_part(name: &str, filename: &str, content: &[u8]) -> (String, Vec<u8>) {
        (
            format!("form-data; name=\"{name}\"; filename=\"{filename}\""),
            content.to_vec(),
        )
    }

    async fn parse(
        parts: &[(String, Vec<u8>)],
        config: &AppConfig,
    ) -> Result<MultipartData, ApiError> {
        let (req, mut payload) = test::TestRequest::default()
            .insert_header((
                actix_web::http::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={BOUNDARY}"),
            ))
            .set_payload(body(parts))
            .to_http_parts();

        let multipart = Multipart::from_request(&req, &mut payload)
            .await
            .expect("o corpo deveria ser um multipart válido");

        let storage = storage(config);

        process_multipart(multipart, config, &storage).await
    }

    #[actix_web::test]
    async fn reads_the_text_fields_into_the_map() {
        let config = config();
        let data = parse(
            &[
                text_part("name", "Bunker 034"),
                text_part("city", "Porto Alegre"),
                text_part("genres", "Techno, Acid"),
            ],
            &config,
        )
        .await
        .expect("campos de texto simples deveriam passar");

        assert_eq!(
            data.fields.get("name").map(String::as_str),
            Some("Bunker 034")
        );
        assert_eq!(
            data.fields.get("genres").map(String::as_str),
            Some("Techno, Acid")
        );
        assert!(data.files.is_empty(), "nenhum arquivo foi enviado");
    }

    #[actix_web::test]
    async fn refuses_a_file_that_is_not_an_image() {
        // O caso que virava XSS armazenado: um .html com a extensão trocada.
        let config = config();
        let erro = parse(
            &[file_part(
                "images",
                "foto.png",
                b"<!doctype html><script>alert(document.cookie)</script>",
            )],
            &config,
        )
        .await
        .expect_err("documento não é imagem");

        assert!(
            matches!(erro, ApiError::FileUploadError(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn refuses_a_file_too_short_to_be_recognised() {
        let config = config();
        let erro = parse(&[file_part("images", "foto.png", b"RIFF")], &config)
            .await
            .expect_err("sem assinatura não dá para reconhecer");

        assert!(
            matches!(erro, ApiError::FileUploadError(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn refuses_an_image_over_the_size_limit() {
        let config = AppConfig {
            max_file_size: 64,
            ..config()
        };

        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend(std::iter::repeat_n(0u8, 1024));

        let erro = parse(&[file_part("images", "foto.png", &png)], &config)
            .await
            .expect_err("passou do teto por arquivo");

        assert!(
            matches!(erro, ApiError::PayloadTooLarge(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn refuses_an_oversized_text_field() {
        // Sem este teto, um "about" de 2 GB é lido inteiro para a memória
        // antes de qualquer validação.
        let config = AppConfig {
            max_field_size: 32,
            ..config()
        };

        let erro = parse(&[text_part("about", &"a".repeat(500))], &config)
            .await
            .expect_err("passou do teto por campo");

        assert!(
            matches!(erro, ApiError::PayloadTooLarge(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn refuses_a_body_over_the_request_limit() {
        // Muitos campos pequenos, cada um dentro do teto individual, somando
        // mais que o corpo inteiro permite.
        let config = AppConfig {
            max_field_size: 64,
            max_request_size: 100,
            ..config()
        };

        let parts: Vec<_> = (0..10)
            .map(|i| text_part(&format!("campo{i}"), &"a".repeat(50)))
            .collect();

        let erro = parse(&parts, &config)
            .await
            .expect_err("a soma passou do teto do corpo");

        assert!(
            matches!(erro, ApiError::PayloadTooLarge(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn refuses_a_body_with_too_many_fields() {
        // Um corpo com um milhão de campos minúsculos passa por baixo de
        // qualquer limite de tamanho.
        let config = config();
        let parts: Vec<_> = (0..=MAX_MULTIPART_FIELDS)
            .map(|i| text_part(&format!("campo{i}"), "x"))
            .collect();

        let erro = parse(&parts, &config)
            .await
            .expect_err("passou do número de campos");

        assert!(
            matches!(erro, ApiError::PayloadTooLarge(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn accepts_exactly_the_field_limit() {
        let config = config();
        let parts: Vec<_> = (0..MAX_MULTIPART_FIELDS)
            .map(|i| text_part(&format!("campo{i}"), "x"))
            .collect();

        let data = parse(&parts, &config).await.expect("no limite ainda passa");

        assert_eq!(data.fields.len(), MAX_MULTIPART_FIELDS);
    }

    #[actix_web::test]
    async fn refuses_a_text_field_that_is_not_valid_utf8() {
        let config = config();
        let erro = parse(
            &[(
                "form-data; name=\"about\"".to_string(),
                vec![0xFF, 0xFE, 0xFD],
            )],
            &config,
        )
        .await
        .expect_err("bytes soltos não são texto");

        assert!(
            matches!(erro, ApiError::FileUploadError(_)),
            "veio {erro:?}"
        );
    }

    #[actix_web::test]
    async fn only_the_images_field_uploads() {
        // Um arquivo enviado com outro nome de campo não vira objeto no
        // bucket: ele cai no caminho de campo de texto.
        let config = config();
        let data = parse(
            &[file_part("anexo", "contrato.pdf", b"%PDF-1.7 conteudo")],
            &config,
        )
        .await
        .expect("não é upload, é campo");

        assert!(data.files.is_empty(), "nada deveria ter subido");
        assert!(data.fields.contains_key("anexo"));
    }
}
