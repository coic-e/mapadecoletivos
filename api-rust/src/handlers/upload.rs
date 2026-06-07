use actix_multipart::Multipart;
use actix_web::web;
use bigdecimal::BigDecimal;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::AppConfig;
use crate::errors::ApiError;

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub filename: String,
}

pub struct MultipartData {
    pub files: Vec<UploadedFile>,
    pub fields: HashMap<String, String>,
}

pub async fn process_multipart(
    mut payload: Multipart,
    config: &AppConfig,
) -> Result<MultipartData, ApiError> {
    let mut uploaded_files = Vec::new();
    let mut fields = HashMap::new();

    // Ensure upload directory exists
    fs::create_dir_all(&config.upload_dir).map_err(|e| {
        ApiError::FileUploadError(format!("Failed to create upload directory: {}", e))
    })?;

    while let Some(field) = payload.next().await {
        let mut field = field.map_err(|e| ApiError::FileUploadError(e.to_string()))?;

        // Get the field name
        let field_name = field.name().to_string();

        // Check if this is a file field
        let content_disposition = field.content_disposition();
        let is_file = content_disposition.get_filename().is_some();

        if is_file && field_name == "images" {
            // Process file upload

            // Get original filename
            let filename = content_disposition
                .get_filename()
                .ok_or_else(|| ApiError::FileUploadError("Missing filename".to_string()))?;

            // Generate unique filename: {timestamp}-{original_filename}
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let safe_filename = sanitize_filename(filename);
            let unique_filename = format!("{}-{}", timestamp, safe_filename);

            // Build file path
            let filepath = PathBuf::from(&config.upload_dir).join(&unique_filename);

            // Check file size as we read
            let mut total_size: usize = 0;
            let mut file = web::block(move || std::fs::File::create(filepath))
                .await
                .map_err(|e| ApiError::FileUploadError(format!("File creation error: {}", e)))?
                .map_err(|e| ApiError::FileUploadError(format!("File creation error: {}", e)))?;

            // Read field data and write to file
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| ApiError::FileUploadError(e.to_string()))?;
                total_size += data.len();

                // Check if file exceeds max size
                if total_size > config.max_file_size {
                    return Err(ApiError::FileUploadError(format!(
                        "File size exceeds maximum of {} bytes",
                        config.max_file_size
                    )));
                }

                file = web::block(move || file.write_all(&data).map(|_| file))
                    .await
                    .map_err(|e| ApiError::FileUploadError(format!("File write error: {}", e)))?
                    .map_err(|e| ApiError::FileUploadError(format!("File write error: {}", e)))?;
            }

            uploaded_files.push(UploadedFile {
                filename: unique_filename,
            });
        } else {
            // Process text field
            let mut field_data = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| ApiError::FileUploadError(e.to_string()))?;
                field_data.extend_from_slice(&data);
            }

            // Convert bytes to string
            let field_value = String::from_utf8(field_data).map_err(|e| {
                ApiError::FileUploadError(format!("Invalid UTF-8 in field {}: {}", field_name, e))
            })?;

            fields.insert(field_name, field_value);
        }
    }

    Ok(MultipartData {
        files: uploaded_files,
        fields,
    })
}

// Keep old function name for backward compatibility
pub async fn process_multipart_files(
    payload: Multipart,
    config: &AppConfig,
) -> Result<Vec<UploadedFile>, ApiError> {
    let data = process_multipart(payload, config).await?;
    Ok(data.files)
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn cleanup_files(upload_dir: &str, filenames: &[String]) {
    for filename in filenames {
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
