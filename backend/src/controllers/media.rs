use axum::{
    extract::{State, Path, Multipart, Extension},
    response::Json as ResponseJson,
    http::StatusCode,
};
use std::path::Path as StdPath;
use tokio::fs;
use uuid::Uuid;
use crate::{
    AppServices,
    models::{Media, NewMedia},
    middleware::{
        validation::validate_file_upload,
        errors::AppError,
        auth::AuthenticatedUser,
    },
};

/// Get all media files (admin only)
/// 
/// Returns a list of all uploaded media files.
/// Requires admin authentication.
pub async fn get_media(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<Media>>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let media = Media::list(&mut conn)?;
    Ok(ResponseJson(media))
}

/// Upload a new media file (admin only)
/// 
/// Handles file upload with validation and security checks.
/// Validates file type, size, and filename for security.
/// Generates unique filenames to prevent conflicts.
/// Requires admin authentication.
pub async fn upload_media(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(services): State<AppServices>, 
    mut multipart: Multipart
) -> Result<(StatusCode, ResponseJson<serde_json::Value>), AppError> {
    // Create upload directory if it doesn't exist
    let upload_dir = "uploads";
    if !StdPath::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create upload directory");
                AppError::InternalError(format!("Failed to create upload directory: {}", e))
            })?;
    }

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::ValidationError(format!("Invalid multipart data: {}", e)))? {
        
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name()
                .ok_or_else(|| AppError::ValidationError("Missing filename".to_string()))?
                .to_string();
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let data = field.bytes().await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to read multipart file bytes");
                    AppError::ValidationError(format!("Failed to read file data: {}", e))
                })?;
            
            // Validate file upload
            validate_file_upload(&file_name, &content_type, data.len())?;
            
            // Generate unique filename
            let file_extension = StdPath::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);
            
            // Save file
            fs::write(&file_path, &data).await
                .map_err(|e| {
                    tracing::error!(error = %e, path = %file_path, "Failed to save uploaded file");
                    AppError::InternalError(format!("Failed to save file: {}", e))
                })?;
            
            // Save to database
            let mut conn = services.db_pool.get()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            
            let new_media = NewMedia {
                file_name: file_name.clone(),
                url: format!("/uploads/{}", unique_filename),
                media_type: Some(content_type.clone()),
                user_id: Some(auth_user.id),
            };
            
            let created_media = Media::create(&mut conn, new_media)
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to insert media record");
                    e
                })?;
            
            return Ok((StatusCode::CREATED, ResponseJson(serde_json::json!({
                "success": true,
                "message": "File uploaded successfully",
                "media": {
                    "id": created_media.id,
                    "name": file_name,
                    "type_": content_type,
                    "size": format!("{} bytes", data.len()),
                    "url": format!("/uploads/{}", unique_filename),
                    "created_at": created_media.uploaded_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                }
            }))));
        }
    }
    
    Err(AppError::ValidationError("No file provided".to_string()))
}

/// Delete a media file (admin only)
/// 
/// Deletes a media file from both database and disk.
/// Requires admin authentication.
pub async fn delete_media(
    State(services): State<AppServices>, 
    Path(id): Path<i32>
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Get media info before deletion to clean up file
    let media = Media::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Media not found".to_string()))?;
    
    // Delete from database
    Media::delete(&mut conn, id)?;
    
    // Delete actual file from disk
    // Extract filename from URL and delete physical file
    if let Some(filename) = media.url.strip_prefix("/uploads/") {
        let file_path = format!("uploads/{}", filename);
        if StdPath::new(&file_path).exists() {
            if let Err(e) = fs::remove_file(&file_path).await {
                tracing::warn!("Failed to delete file {}: {}", file_path, e);
            }
        }
    }
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Media deleted successfully"
    })))
}