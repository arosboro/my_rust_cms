// src/frontend/services/api_service.rs

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::services::auth_service::get_auth_token;

const API_BASE_URL: &str = "http://localhost:8081/api";

// Helper function to create authenticated requests
fn create_authenticated_request(method: &str, url: &str) -> Result<gloo_net::http::RequestBuilder, ApiServiceError> {
    let token = get_auth_token().map_err(|_| ApiServiceError::ServerError("Not authenticated".to_string()))?;
    
    let request_builder = match method {
        "GET" => Request::get(url),
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        _ => return Err(ApiServiceError::ServerError("Invalid HTTP method".to_string())),
    };
    
    Ok(request_builder.header("Authorization", &format!("Bearer {}", token)))
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub author: String,
    pub status: String,
    pub category_id: Option<i32>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Comment {
    pub id: Option<i32>,
    pub content: String,
    pub author: String,
    pub post_id: i32,
    pub status: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct MediaItem {
    pub id: Option<i32>,
    #[serde(rename = "file_name")]
    pub name: String,
    #[serde(rename = "media_type")]
    pub type_: String,
    pub url: String,
    #[serde(rename = "uploaded_at")]
    pub created_at: Option<String>,
    pub user_id: Option<i32>,
    pub size: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct PageItem {
    pub id: Option<i32>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Stats {
    pub total_posts: usize,
    pub total_users: usize,
    pub total_comments: usize,
    pub total_media: usize,
    pub system_status: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PerformanceMetrics {
    pub backend_metrics: BackendMetrics,
    pub frontend_metrics: FrontendMetrics,
    pub system_metrics: SystemMetrics,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BackendMetrics {
    pub avg_request_time: f64,
    pub max_request_time: f64,
    pub min_request_time: f64,
    pub total_requests: u64,
    pub error_rate: f64,
    pub db_query_avg_time: f64,
    pub db_connection_pool_active: u32,
    pub db_connection_pool_idle: u32,
    pub memory_usage_mb: f64,
    pub active_sessions: u32,
    pub session_avg_duration: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FrontendMetrics {
    pub wasm_bundle_size_kb: f64,
    pub page_load_time: f64,
    pub time_to_interactive: f64,
    pub first_contentful_paint: f64,
    pub largest_contentful_paint: f64,
    pub cumulative_layout_shift: f64,
    pub network_request_avg_time: f64,
    pub component_render_avg_time: f64,
    pub dom_nodes_count: u32,
    pub memory_usage_js_mb: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_total_mb: f64,
    pub memory_available_mb: f64,
    pub disk_usage_percent: f64,
    pub network_io_bytes_per_sec: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug)]
pub enum ApiServiceError {
    NetworkError(String),
    ParseError(String),
    ServerError(String),
}

impl std::fmt::Display for ApiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiServiceError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiServiceError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

// Posts API
pub async fn get_posts() -> Result<Vec<Post>, ApiServiceError> {
    let response = Request::get(&format!("{}/posts", API_BASE_URL))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let posts: Vec<Post> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(posts)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn get_post(id: i32) -> Result<Post, ApiServiceError> {
    let response = Request::get(&format!("{}/posts/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let post: Post = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(post)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn create_post(post: &Post) -> Result<Post, ApiServiceError> {
    let response = create_authenticated_request("POST", &format!("{}/posts", API_BASE_URL))?
        .json(post)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 201 {
        let created_post: Post = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(created_post)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn update_post(id: i32, post: &Post) -> Result<Post, ApiServiceError> {
    let response = create_authenticated_request("PUT", &format!("{}/posts/{}", API_BASE_URL, id))?
        .json(post)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let updated_post: Post = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(updated_post)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn delete_post(id: i32) -> Result<(), ApiServiceError> {
    let response = create_authenticated_request("DELETE", &format!("{}/posts/{}", API_BASE_URL, id))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        Ok(())
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Users API
pub async fn get_users() -> Result<Vec<User>, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/users", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let users: Vec<User> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(users)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn create_user(user: &User) -> Result<User, ApiServiceError> {
    let response = Request::post(&format!("{}/users", API_BASE_URL))
        .json(user)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 201 {
        let created_user: User = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(created_user)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn update_user(id: i32, user: &User) -> Result<User, ApiServiceError> {
    let response = Request::put(&format!("{}/users/{}", API_BASE_URL, id))
        .json(user)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let updated_user: User = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(updated_user)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn delete_user(id: i32) -> Result<(), ApiServiceError> {
    let response = Request::delete(&format!("{}/users/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 204 {
        Ok(())
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Comments API
pub async fn get_comments() -> Result<Vec<Comment>, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/comments", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let comments: Vec<Comment> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(comments)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

#[allow(dead_code)]
pub async fn create_comment(comment: &Comment) -> Result<Comment, ApiServiceError> {
    let response = Request::post(&format!("{}/comments", API_BASE_URL))
        .json(comment)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 201 {
        let created_comment: Comment = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(created_comment)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn update_comment(id: i32, comment: &Comment) -> Result<Comment, ApiServiceError> {
    let response = Request::put(&format!("{}/comments/{}", API_BASE_URL, id))
        .json(comment)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let updated_comment: Comment = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(updated_comment)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn delete_comment(id: i32) -> Result<(), ApiServiceError> {
    let response = Request::delete(&format!("{}/comments/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 204 {
        Ok(())
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Media API
pub async fn get_media() -> Result<Vec<MediaItem>, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/media", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let media: Vec<MediaItem> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(media)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

#[allow(dead_code)]
pub async fn create_media(media: &MediaItem) -> Result<MediaItem, ApiServiceError> {
    let response = Request::post(&format!("{}/media", API_BASE_URL))
        .json(media)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 201 {
        let created_media: MediaItem = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(created_media)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn delete_media(id: i32) -> Result<(), ApiServiceError> {
    let response = Request::delete(&format!("{}/media/{}", API_BASE_URL, id))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 204 {
        Ok(())
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Stats API
pub async fn get_stats() -> Result<Stats, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/stats", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let stats: Stats = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(stats)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Pages API
pub async fn get_pages() -> Result<Vec<PageItem>, ApiServiceError> {
    let request = create_authenticated_request("GET", &format!("{}/pages", API_BASE_URL))?;
    let response = request.send().await.map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    if response.ok() {
        let pages: Vec<PageItem> = response.json().await.map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(pages)
    } else {
        Err(ApiServiceError::NetworkError(format!("HTTP {}", response.status())))
    }
}

pub async fn create_page(page: &PageItem) -> Result<PageItem, ApiServiceError> {
    let request = create_authenticated_request("POST", &format!("{}/pages", API_BASE_URL))?
        .json(page)
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    let response = request.send().await.map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    if response.ok() {
        let created_page: PageItem = response.json().await.map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(created_page)
    } else {
        Err(ApiServiceError::NetworkError(format!("HTTP {}", response.status())))
    }
}

pub async fn update_page(id: i32, page: &PageItem) -> Result<PageItem, ApiServiceError> {
    let request = create_authenticated_request("PUT", &format!("{}/pages/{}", API_BASE_URL, id))?
        .json(page)
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    let response = request.send().await.map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    if response.ok() {
        let updated_page: PageItem = response.json().await.map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(updated_page)
    } else {
        Err(ApiServiceError::NetworkError(format!("HTTP {}", response.status())))
    }
}

pub async fn delete_page(id: i32) -> Result<(), ApiServiceError> {
    let request = create_authenticated_request("DELETE", &format!("{}/pages/{}", API_BASE_URL, id))?;
    let response = request.send().await.map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;
    
    if response.ok() {
        Ok(())
    } else {
        Err(ApiServiceError::NetworkError(format!("HTTP {}", response.status())))
    }
}

// Categories API
pub async fn get_categories() -> Result<Vec<Category>, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/categories", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let categories: Vec<Category> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(categories)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// Performance Metrics API
pub async fn get_performance_metrics() -> Result<PerformanceMetrics, ApiServiceError> {
    let response = Request::get(&format!("{}/performance", API_BASE_URL))
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let metrics: PerformanceMetrics = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(metrics)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

// System Settings structs
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Setting {
    pub id: i32,
    pub setting_key: String,
    pub setting_value: Option<String>,
    pub created_at: Option<String>,
    pub setting_type: String,
    pub description: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsRequest {
    pub settings: Vec<SettingData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingData {
    pub key: String,
    pub value: String,
    pub setting_type: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub rust_version: String,
    pub database_version: String,
    pub uptime: String,
    pub memory_usage: String,
    pub cpu_usage: String,
    pub disk_usage: String,
    pub active_sessions: i32,
    pub total_posts: i64,
    pub total_users: i64,
    pub total_media: i64,
    pub last_backup: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub created_at: String,
    pub backup_type: String,
    pub checksum: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupRequest {
    pub backup_type: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataSnapshot {
    pub timestamp: String,
    pub tables: Vec<TableSnapshot>,
    pub total_rows: i64,
    pub data_hash: String,
    pub integrity_verified: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TableSnapshot {
    pub table_name: String,
    pub row_count: i64,
    pub table_hash: String,
    pub last_modified: Option<String>,
}

// System Settings API
pub async fn get_settings(setting_type: Option<&str>) -> Result<Vec<Setting>, ApiServiceError> {
    let url = match setting_type {
        Some(t) => format!("{}/system/settings?setting_type={}", API_BASE_URL, t),
        None => format!("{}/system/settings", API_BASE_URL),
    };
    
    let response = create_authenticated_request("GET", &url)?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let settings: Vec<Setting> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(settings)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn update_settings(settings: Vec<SettingData>) -> Result<Vec<Setting>, ApiServiceError> {
    let request_body = SettingsRequest { settings };
    
    let response = create_authenticated_request("PUT", &format!("{}/system/settings", API_BASE_URL))?
        .json(&request_body)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let updated_settings: Vec<Setting> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(updated_settings)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn get_system_info() -> Result<SystemInfo, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/system/info", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let system_info: SystemInfo = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(system_info)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn create_backup(backup_request: BackupRequest) -> Result<BackupInfo, ApiServiceError> {
    let response = create_authenticated_request("POST", &format!("{}/system/backup", API_BASE_URL))?
        .json(&backup_request)
        .map_err(|e| ApiServiceError::ParseError(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 || response.status() == 201 {
        let backup_info: BackupInfo = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(backup_info)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn get_backups() -> Result<Vec<BackupInfo>, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/system/backups", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let backups: Vec<BackupInfo> = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(backups)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn get_data_snapshot() -> Result<DataSnapshot, ApiServiceError> {
    let response = create_authenticated_request("GET", &format!("{}/system/snapshot", API_BASE_URL))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let snapshot: DataSnapshot = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(snapshot)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}

pub async fn restore_backup(backup_id: &str) -> Result<String, ApiServiceError> {
    let response = create_authenticated_request("POST", &format!("{}/system/backup/{}/restore", API_BASE_URL, backup_id))?
        .send()
        .await
        .map_err(|e| ApiServiceError::NetworkError(e.to_string()))?;

    if response.status() == 200 {
        let result: String = response
            .json()
            .await
            .map_err(|e| ApiServiceError::ParseError(e.to_string()))?;
        Ok(result)
    } else {
        Err(ApiServiceError::ServerError(format!("HTTP {}", response.status())))
    }
}
