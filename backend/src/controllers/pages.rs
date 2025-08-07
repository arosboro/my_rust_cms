use axum::{
    extract::{State, Path, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use crate::{
    AppServices,
    models::{Page, NewPage, UpdatePage},
    middleware::{
        validation::validate_text_content,
        errors::AppError,
    },
};

// Frontend-compatible Page structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FrontendPage {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Page> for FrontendPage {
    fn from(page: Page) -> Self {
        FrontendPage {
            id: Some(page.id),
            title: page.title,
            slug: page.slug,
            content: page.content,
            status: page.status,
            created_at: page.created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            updated_at: page.updated_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// Get all pages (public endpoint)
/// 
/// Returns a list of all published pages.
/// No authentication required for public access.
pub async fn get_pages(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<FrontendPage>>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let pages = Page::list(&mut conn)?;
    let frontend_pages: Vec<FrontendPage> = pages.into_iter().map(FrontendPage::from).collect();
    Ok(ResponseJson(frontend_pages))
}

/// Get a specific page by ID (public endpoint)
/// 
/// Returns a single page by its ID.
/// No authentication required for public access.
pub async fn get_page(
    State(services): State<AppServices>, 
    Path(id): Path<i32>
) -> Result<ResponseJson<FrontendPage>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    let page = Page::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;
    
    Ok(ResponseJson(FrontendPage::from(page)))
}

/// Get a page by slug (public endpoint)
/// 
/// Returns a page by its URL slug.
/// Generates slug from title since database doesn't store slugs.
/// No authentication required for public access.
pub async fn get_page_by_slug(
    State(services): State<AppServices>, 
    Path(slug): Path<String>
) -> Result<ResponseJson<FrontendPage>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Since database doesn't have slug field, we need to search by generating slugs from titles
    let pages = Page::list(&mut conn)?;
    
    for page in pages {
        let frontend_page = FrontendPage::from(page);
        if frontend_page.slug == slug {
            return Ok(ResponseJson(frontend_page));
        }
    }
    
    Err(AppError::NotFound("Page not found".to_string()))
}

/// Create a new page (admin only)
/// 
/// Creates a new page with validation.
/// Content is sanitized and validated for security.
/// Requires admin authentication.
pub async fn create_page(
    State(services): State<AppServices>, 
    Json(page): Json<FrontendPage>
) -> Result<(StatusCode, ResponseJson<FrontendPage>), AppError> {
    // Validate input
    if page.title.trim().is_empty() {
        return Err(AppError::ValidationError("Title cannot be empty".to_string()));
    }
    
    validate_text_content(&page.title, 200)?;
    validate_text_content(&page.content, 200000)?;
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    let new_page = NewPage {
        title: page.title.trim().to_string(),
        content: page.content.trim().to_string(),
        user_id: Some(2), // Using admin user ID (should get from auth in production)
        slug: page.slug.trim().to_string(),
        status: page.status.trim().to_string(),
    };
    
    let created_page = Page::create(&mut conn, new_page)?;
    let response = FrontendPage::from(created_page);
    
    Ok((StatusCode::CREATED, ResponseJson(response)))
}

/// Update an existing page (admin only)
/// 
/// Updates a page with validation and sanitization.
/// Requires admin authentication.
pub async fn update_page(
    State(services): State<AppServices>, 
    Path(id): Path<i32>, 
    Json(page): Json<FrontendPage>
) -> Result<ResponseJson<FrontendPage>, AppError> {
    // Validate input
    if page.title.trim().is_empty() {
        return Err(AppError::ValidationError("Title cannot be empty".to_string()));
    }
    
    validate_text_content(&page.title, 200)?;
    validate_text_content(&page.content, 200000)?;
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if page exists
    let _existing_page = Page::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;
    
    let update_page = UpdatePage {
        title: Some(page.title.trim().to_string()),
        content: Some(page.content.trim().to_string()),
        user_id: None,
        updated_at: Some(chrono::Utc::now().naive_utc()),
        slug: Some(page.slug.trim().to_string()),
        status: Some(page.status.trim().to_string()),
    };
    
    let updated_page = Page::update(&mut conn, id, update_page)?;
    Ok(ResponseJson(FrontendPage::from(updated_page)))
}

/// Delete a page (admin only)
/// 
/// Permanently deletes a page.
/// Requires admin authentication.
pub async fn delete_page(
    State(services): State<AppServices>, 
    Path(id): Path<i32>
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if page exists
    let _existing_page = Page::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;
    
    Page::delete(&mut conn, id)?;
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Page deleted successfully"
    })))
}