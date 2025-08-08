use axum::{
    extract::{State, Path, Json, Query},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use md5;
use crate::{
    AppServices,
    models::{Comment, NewComment, UpdateComment, User},
    middleware::{
        validation::validate_text_content,
        errors::AppError,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentQueryParams {
    pub post_id: Option<i32>,
    pub page_id: Option<i32>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicCommentRequest {
    pub content: String,
    pub post_id: Option<i32>,
    pub page_id: Option<i32>,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentWithGravatar {
    pub id: i32,
    pub post_id: Option<i32>,
    pub page_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub author_username: Option<String>,
    pub author_email: Option<String>,
    pub gravatar_url: String,
}

fn generate_gravatar_url(email: &str, size: u32) -> String {
    let trimmed_email = email.trim().to_lowercase();
    let hash = format!("{:x}", md5::compute(trimmed_email.as_bytes()));
    format!("https://www.gravatar.com/avatar/{}?s={}&d=identicon&r=pg", hash, size)
}

/// Get all comments (admin only)
/// 
/// Returns a list of all comments in the system with author information.
/// Requires admin authentication.
pub async fn get_comments(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<crate::models::comment::CommentWithRelations>>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let comments = Comment::list_with_relations(&mut conn)?;
    Ok(ResponseJson(comments))
}

/// Get comments for a post or page (public endpoint)
/// 
/// Returns comments for a specific post or page with Gravatar URLs.
/// No authentication required.
pub async fn get_post_comments(
    State(services): State<AppServices>,
    Query(params): Query<CommentQueryParams>
) -> Result<ResponseJson<Vec<CommentWithGravatar>>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    let comments_with_relations = if let Some(post_id) = params.post_id {
        // Get comments for specific post with user details
        use crate::schema::{comments, users, posts};
        use diesel::prelude::*;
        
        comments::table
            .left_join(users::table.on(comments::user_id.eq(users::id.nullable())))
            .left_join(posts::table.on(comments::post_id.eq(posts::id.nullable())))
            .filter(comments::post_id.eq(post_id))
            .order(comments::created_at.asc())
            .select((
                comments::id,
                comments::post_id,
                comments::page_id,
                comments::user_id,
                comments::content,
                comments::created_at,
                comments::updated_at,
                users::username.nullable(),
                users::email.nullable(),
            ))
            .load::<(i32, Option<i32>, Option<i32>, Option<i32>, String, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<String>, Option<String>)>(&mut conn)?
    } else if let Some(page_id) = params.page_id {
        // Get comments for specific page with user details
        use crate::schema::{comments, users, pages};
        use diesel::prelude::*;
        
        comments::table
            .left_join(users::table.on(comments::user_id.eq(users::id.nullable())))
            .left_join(pages::table.on(comments::page_id.eq(pages::id.nullable())))
            .filter(comments::page_id.eq(page_id))
            .order(comments::created_at.asc())
            .select((
                comments::id,
                comments::post_id,
                comments::page_id,
                comments::user_id,
                comments::content,
                comments::created_at,
                comments::updated_at,
                users::username.nullable(),
                users::email.nullable(),
            ))
            .load::<(i32, Option<i32>, Option<i32>, Option<i32>, String, Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>, Option<String>, Option<String>)>(&mut conn)?
    } else {
        // If no post_id or page_id specified, return empty vec for public endpoint
        vec![]
    };
    
    let comments_with_gravatar: Vec<CommentWithGravatar> = comments_with_relations
        .into_iter()
        .map(|(id, post_id, page_id, user_id, content, created_at, updated_at, username, email)| {
            let gravatar_url = email
                .as_ref()
                .map(|e| generate_gravatar_url(e, 80))
                .unwrap_or_else(|| generate_gravatar_url("default@example.com", 80));
            
            CommentWithGravatar {
                id,
                post_id,
                page_id,
                user_id,
                content,
                created_at: created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                updated_at: updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                author_username: username,
                author_email: email,
                gravatar_url,
            }
        })
        .collect();
    
    Ok(ResponseJson(comments_with_gravatar))
}

/// Create a new comment (admin only)
/// 
/// Creates a new comment with validation.
/// Content is sanitized and validated for security.
/// Requires admin authentication.
pub async fn create_comment(
    State(services): State<AppServices>, 
    Json(comment_data): Json<serde_json::Value>
) -> Result<(StatusCode, ResponseJson<serde_json::Value>), AppError> {
    let content = comment_data["content"].as_str()
        .ok_or_else(|| AppError::ValidationError("Content is required".to_string()))?
        .to_string();
    let post_id = comment_data["post_id"].as_i64().map(|id| id as i32);
    let user_id = comment_data["user_id"].as_i64().map(|id| id as i32);
    
    // Validate content
    if content.trim().is_empty() {
        return Err(AppError::ValidationError("Content cannot be empty".to_string()));
    }
    
    validate_text_content(&content, 2000)?;
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    let page_id = comment_data["page_id"].as_i64().map(|id| id as i32);
    
    let new_comment = NewComment {
        post_id,
        page_id,
        user_id,
        content: content.trim().to_string(),
    };
    
    let created_comment = Comment::create(&mut conn, new_comment)?;
    
    Ok((StatusCode::CREATED, ResponseJson(serde_json::json!({
        "id": created_comment.id,
        "content": created_comment.content,
        "post_id": created_comment.post_id,
        "user_id": created_comment.user_id,
        "created_at": created_comment.created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
    }))))
}

/// Create a new comment (public endpoint, requires authentication)
/// 
/// Creates a new comment from authenticated users.
/// Content is sanitized and validated for security.
/// Requires user authentication but not admin.
pub async fn create_public_comment(
    State(services): State<AppServices>, 
    Json(comment_request): Json<PublicCommentRequest>
) -> Result<(StatusCode, ResponseJson<CommentWithGravatar>), AppError> {
    // Validate content
    if comment_request.content.trim().is_empty() {
        return Err(AppError::ValidationError("Content cannot be empty".to_string()));
    }
    
    // Ensure either post_id or page_id is provided, but not both
    match (comment_request.post_id, comment_request.page_id) {
        (Some(_), Some(_)) => return Err(AppError::ValidationError("Cannot specify both post_id and page_id".to_string())),
        (None, None) => return Err(AppError::ValidationError("Must specify either post_id or page_id".to_string())),
        _ => {} // Valid: exactly one is specified
    }
    
    validate_text_content(&comment_request.content, 2000)?;
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Verify user exists
    let user = User::find_by_id(&mut conn, comment_request.user_id)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    let new_comment = NewComment {
        post_id: comment_request.post_id,
        page_id: comment_request.page_id,
        user_id: Some(comment_request.user_id),
        content: comment_request.content.trim().to_string(),
    };
    
    let created_comment = Comment::create(&mut conn, new_comment)?;
    
    // Generate gravatar URL
    let gravatar_url = user.email
        .as_ref()
        .map(|e| generate_gravatar_url(e, 80))
        .unwrap_or_else(|| generate_gravatar_url("default@example.com", 80));
    
    let comment_with_gravatar = CommentWithGravatar {
        id: created_comment.id,
        post_id: created_comment.post_id,
        page_id: created_comment.page_id,
        user_id: created_comment.user_id,
        content: created_comment.content,
        created_at: created_comment.created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        updated_at: created_comment.updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        author_username: Some(user.username),
        author_email: user.email,
        gravatar_url,
    };
    
    Ok((StatusCode::CREATED, ResponseJson(comment_with_gravatar)))
}

/// Update an existing comment (admin only)
/// 
/// Updates comment content with validation.
/// Requires admin authentication.
pub async fn update_comment(
    State(services): State<AppServices>, 
    Path(id): Path<i32>, 
    Json(comment_data): Json<serde_json::Value>
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    let content = comment_data["content"].as_str().map(|s| s.to_string());
    
    if let Some(ref content_str) = content {
        if content_str.trim().is_empty() {
            return Err(AppError::ValidationError("Content cannot be empty".to_string()));
        }
        validate_text_content(content_str, 2000)?;
    }
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if comment exists
    let _existing_comment = Comment::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
    
    let update_comment = UpdateComment {
        content: content.map(|c| c.trim().to_string()),
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };
    
    let updated_comment = Comment::update(&mut conn, id, update_comment)?;
    
    Ok(ResponseJson(serde_json::json!({
        "id": updated_comment.id,
        "content": updated_comment.content,
        "post_id": updated_comment.post_id,
        "user_id": updated_comment.user_id,
        "created_at": updated_comment.created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        "updated_at": updated_comment.updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
    })))
}

/// Delete a comment (admin only)
/// 
/// Permanently deletes a comment.
/// Requires admin authentication.
pub async fn delete_comment(
    State(services): State<AppServices>, 
    Path(id): Path<i32>
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if comment exists
    let _existing_comment = Comment::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
    
    Comment::delete(&mut conn, id)?;
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Comment deleted successfully"
    })))
}