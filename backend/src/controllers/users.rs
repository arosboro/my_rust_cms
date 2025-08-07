use axum::{
    extract::{State, Path, Json, Extension},
    response::Json as ResponseJson,

};
use serde::{Deserialize, Serialize};
use crate::{
    AppServices,
    models::{User, NewUser, UpdateUser},
    middleware::{
        auth::AuthenticatedUser,
        validation::{validate_username, validate_email, validate_password},
        errors::AppError,
    },
};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct PromoteUserRequest {
    pub role: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

/// Get all users (admin only)
/// 
/// Returns a list of all users in the system.
/// Requires admin authentication.
pub async fn get_users(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<UserResponse>>, AppError> {
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let users = User::list(&mut conn)?;
    
    let user_responses: Vec<UserResponse> = users.into_iter().map(|user| UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        status: user.status,
        email_verified: user.email_verified,
        created_at: user.created_at,
    }).collect();
    
    Ok(ResponseJson(user_responses))
}

/// Create a new user (admin only)
/// 
/// Creates a new user with validation and duplicate checking.
/// Passwords are automatically hashed using bcrypt.
/// Requires admin authentication.
pub async fn create_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    State(services): State<AppServices>,
    Json(user_req): Json<CreateUserRequest>,
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    
    // Validate input
    validate_username(&user_req.username)?;
    validate_password(&user_req.password)?;
    
    if let Some(ref email) = user_req.email {
        validate_email(email)?;
    }
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if username already exists
    if User::find_by_username(&mut conn, &user_req.username)?.is_some() {
        return Err(AppError::ConflictError("Username already exists".to_string()));
    }
    
    // Check if email already exists (if provided)
    if let Some(ref email) = user_req.email {
        if User::find_by_email(&mut conn, email)?.is_some() {
            return Err(AppError::ConflictError("Email already exists".to_string()));
        }
    }
    
    // Hash password
    let hashed_password = bcrypt::hash(&user_req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;
    
    let new_user = NewUser {
        username: user_req.username,
        password: hashed_password,
        email: user_req.email,
        role: user_req.role.unwrap_or_else(|| "user".to_string()),
        status: "active".to_string(),
        email_verified: Some(true), // Admin-created users are pre-verified
        email_verification_token: None,
        email_verification_expires_at: None,
    };
    
    let created_user = User::create(&mut conn, new_user)?;
    
    Ok(ResponseJson(serde_json::json!({
        "id": created_user.id,
        "username": created_user.username,
        "email": created_user.email,
        "role": created_user.role,
        "status": created_user.status
    })))
}

/// Update an existing user (admin only)
/// 
/// Updates user information with validation.
/// Passwords are automatically hashed if provided.
/// Requires admin authentication.
pub async fn update_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    State(services): State<AppServices>,
    Path(id): Path<i32>,
    Json(user_req): Json<UpdateUserRequest>,
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    
    // Validate input if provided
    if let Some(ref username) = user_req.username {
        validate_username(username)?;
    }
    
    if let Some(ref email) = user_req.email {
        validate_email(email)?;
    }
    
    if let Some(ref password) = user_req.password {
        validate_password(password)?;
    }
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if user exists
    let _existing_user = User::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Hash password if provided
    let hashed_password = if let Some(password) = user_req.password {
        Some(bcrypt::hash(&password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?)
    } else {
        None
    };
    
    let update_user = UpdateUser {
        username: user_req.username,
        password: hashed_password,
        email: user_req.email,
        role: user_req.role,
        status: user_req.status,
        email_verified: None, // Don't change verification status in regular updates
        email_verification_token: None,
        email_verification_expires_at: None,
    };
    
    let updated_user = User::update(&mut conn, id, update_user)?;
    
    Ok(ResponseJson(serde_json::json!({
        "id": updated_user.id,
        "username": updated_user.username,
        "email": updated_user.email,
        "role": updated_user.role,
        "status": updated_user.status
    })))
}

/// Delete a user (admin only)
/// 
/// Deletes a user and all associated sessions.
/// Prevents self-deletion for safety.
/// Requires admin authentication.
pub async fn delete_user(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(services): State<AppServices>,
    Path(id): Path<i32>,
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    
    // Prevent self-deletion
    if auth_user.id == id {
        return Err(AppError::ValidationError("Cannot delete your own account".to_string()));
    }
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if user exists
    let _existing_user = User::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    User::delete(&mut conn, id)?;
    
    // Also cleanup user's sessions
    let _ = services.session_manager.logout_all_user_sessions(id).await;
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "User deleted successfully"
    })))
}

/// Promote or change user role (admin only)
/// 
/// Allows admins to promote users to editor or demote them back to user.
/// Admins can only promote to 'editor' role or demote to 'user' role.
/// Cannot change another admin's role for security.
/// Requires admin authentication.
pub async fn promote_user(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(services): State<AppServices>,
    Path(id): Path<i32>,
    Json(promote_req): Json<PromoteUserRequest>,
) -> Result<ResponseJson<serde_json::Value>, AppError> {
    
    // Validate role
    if !["user", "editor"].contains(&promote_req.role.as_str()) {
        return Err(AppError::ValidationError("Invalid role. Only 'user' and 'editor' roles are allowed.".to_string()));
    }
    
    // Prevent self-role change
    if auth_user.id == id {
        return Err(AppError::ValidationError("Cannot change your own role".to_string()));
    }
    
    let mut conn = services.db_pool.get()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    // Check if user exists
    let existing_user = User::find_by_id(&mut conn, id)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Prevent changing another admin's role
    if existing_user.role == "admin" {
        return Err(AppError::ValidationError("Cannot change an admin's role".to_string()));
    }
    
    let update_user = UpdateUser {
        username: None,
        password: None,
        email: None,
        role: Some(promote_req.role.clone()),
        status: None,
        email_verified: None,
        email_verification_token: None,
        email_verification_expires_at: None,
    };
    
    let updated_user = User::update(&mut conn, id, update_user)?;
    
    let action = if promote_req.role == "editor" { "promoted to" } else { "demoted to" };
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": format!("User {} {} {} role", updated_user.username, action, promote_req.role),
        "user": UserResponse {
            id: updated_user.id,
            username: updated_user.username,
            email: updated_user.email,
            role: updated_user.role,
            status: updated_user.status,
            email_verified: updated_user.email_verified,
            created_at: updated_user.created_at,
        }
    })))
}