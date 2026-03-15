use crate::error::AppError;
use cloudinary::upload::result::UploadResult;
use cloudinary::upload::{Source, Upload, UploadOptions};
use reqwest::Url;
use std::env;
use std::fs;
use uuid::Uuid;

pub async fn upload_image(cloudinary_url: &str, bytes: Vec<u8>) -> Result<String, AppError> {
    let parsed_url = Url::parse(cloudinary_url)
        .map_err(|e| AppError::InternalError(format!("Invalid Cloudinary URL: {}", e)))?;

    let api_key = parsed_url.username().to_string();
    let api_secret = parsed_url
        .password()
        .ok_or_else(|| AppError::InternalError("Missing API secret in Cloudinary URL".to_string()))?
        .to_string();
    let cloud_name = parsed_url
        .host_str()
        .ok_or_else(|| AppError::InternalError("Missing cloud name in Cloudinary URL".to_string()))?
        .to_string();

    let upload = Upload::new(api_key, cloud_name, api_secret);

    let temp_path = env::temp_dir().join(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, bytes)
        .map_err(|e| AppError::InternalError(format!("Failed to write temp file: {}", e)))?;

    let result = upload
        .image(Source::Path(temp_path.clone()), &UploadOptions::default())
        .await
        .map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            AppError::InternalError(format!("Cloudinary upload failed: {}", e))
        })?;

    let _ = fs::remove_file(&temp_path);

    match result {
        UploadResult::Success(response) => Ok(response.secure_url),
        UploadResult::Error(err) => Err(AppError::InternalError(err.error.message)),
    }
}
