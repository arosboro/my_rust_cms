// backend/src/main.rs

mod config;
mod database;
mod schema;
mod models;
mod middleware;
mod services;
mod controllers;

use axum::{
    routing::{get, post, put, delete},
    http::StatusCode,
    response::IntoResponse,
    Router,
    middleware as axum_middleware,
};
use std::net::SocketAddr;
use tracing::info;
use tower_http::cors::CorsLayer;
use config::Config;
use database::{DbPool, establish_connection_pool};
use models::*;
use middleware::auth::{auth_middleware_with_services, admin_auth_middleware_with_services};
// Rate limiting temporarily disabled due to API changes
// use middleware::rate_limiting::{create_auth_rate_limiter, create_upload_rate_limiter};
use middleware::security_headers::security_headers_middleware;

use services::{SessionManager, SessionConfig};


// Database connection pool state
use std::sync::Arc;



#[derive(Clone)]
pub struct AppServices {
    pub db_pool: Arc<DbPool>,
    pub session_manager: SessionManager,
    pub db_service: services::DbService,
}

// Re-export controller types for convenience
pub use controllers::auth::{LoginRequest, LoginResponse, UserProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::new()?;
    
    // Set up logging
    std::env::set_var("RUST_LOG", &config.rust_log);
    tracing_subscriber::fmt::init();

    // Initialize database connection pool
    let pool = establish_connection_pool(&config.database_url)?;
    info!("Database connection pool established");
    
    // Store pool in state and initialize services
    let db_pool = Arc::new(pool);
    
    // Initialize session manager with custom config
    let session_config = SessionConfig {
        session_duration_hours: 24,
        cleanup_interval_minutes: 10, // More frequent cleanup for demo
        max_sessions_per_user: 3,
        enable_session_refresh: true,
        refresh_threshold_minutes: 30,
    };
    
    let session_manager = SessionManager::new(db_pool.clone(), session_config);
    
    // Start background session cleanup
    let _cleanup_task = session_manager.clone().start_background_cleanup().await;
    info!("Session cleanup background task started");
    
    let db_service = services::DbService::new(db_pool.clone());
    
    let app_services = AppServices {
        db_pool: db_pool.clone(),
        session_manager,
        db_service,
    };
    
    // Initialize with demo data in database
    {
        let mut conn = db_pool.get()?;
        
        // Create demo user if none exists, or update existing admin user password
        let users = User::list(&mut conn)?;
        if users.is_empty() {
            let hashed_password = bcrypt::hash("admin", bcrypt::DEFAULT_COST)
                .map_err(|e| format!("Password hashing failed: {}", e))?;
            let demo_user = NewUser {
                username: "admin".to_string(),
                password: hashed_password,
                email: Some("admin@example.com".to_string()),
                role: "admin".to_string(),
                status: "active".to_string(),
            };
            let _user = User::create(&mut conn, demo_user)?;
            info!("Created demo user: admin with password 'admin'");
        } else if let Some(admin_user) = users.iter().find(|u| u.username == "admin") {
            // Check if password is already hashed (bcrypt hashes start with $2a$, $2b$, or $2y$)
            if !admin_user.password.starts_with("$2") {
                let hashed_password = bcrypt::hash("admin", bcrypt::DEFAULT_COST)
                    .map_err(|e| format!("Password hashing failed: {}", e))?;
                let update_user = UpdateUser {
                    username: None,
                    password: Some(hashed_password),
                    email: None,
                    role: None,
                    status: None,
                };
                let _updated_user = User::update(&mut conn, admin_user.id, update_user)?;
                info!("Updated admin user password to properly hashed version");
            }
        }
        
        // Create demo category if none exists
        if Category::list(&mut conn)?.is_empty() {
            let demo_category = NewCategory {
                name: "General".to_string(),
            };
            let _category = Category::create(&mut conn, demo_category)?;
            info!("Created demo category: General");
        }
        
        // Create default navigation items if none exist
        if Navigation::list(&mut conn)?.is_empty() {
            let home_nav = NewNavigation {
                title: "Home".to_string(),
                url: "/".to_string(),
                order_position: 1,
                is_active: true,
                menu_area: "header".to_string(),
                parent_id: None,
                icon: Some("home".to_string()),
                css_class: None,
                target: Some("_self".to_string()),
                mobile_visible: true,
                description: Some("Homepage link".to_string()),
            };
            let _home = Navigation::create(&mut conn, home_nav)?;
            info!("Created default navigation item: Home");
            
            let posts_nav = NewNavigation {
                title: "Posts".to_string(),
                url: "/posts".to_string(),
                order_position: 2,
                is_active: true,
                menu_area: "header".to_string(),
                parent_id: None,
                icon: Some("article".to_string()),
                css_class: None,
                target: Some("_self".to_string()),
                mobile_visible: true,
                description: Some("View all posts".to_string()),
            };
            let _posts = Navigation::create(&mut conn, posts_nav)?;
            info!("Created default navigation item: Posts");
        }
        
        // Create default site settings if they don't exist
        use crate::models::setting::*;
        
        // Admin button visibility setting
        if Setting::find_by_key(&mut conn, "admin_button_visible")?.is_none() {
            let admin_button_setting = NewSetting {
                setting_key: "admin_button_visible".to_string(),
                setting_value: Some("true".to_string()),
                setting_type: "site".to_string(),
                description: Some("Show admin button in public navigation".to_string()),
            };
            let _setting = Setting::create(&mut conn, admin_button_setting)?;
            info!("Created default setting: admin_button_visible = true");
        }

        // Create default menu areas if they don't exist
        use diesel::prelude::*;
        use crate::schema::menu_areas;
        
        let existing_areas = menu_areas::table.load::<MenuArea>(&mut conn)?;
        if existing_areas.is_empty() {
            let default_areas = vec![
                (
                    "header",
                    "Header Menu",
                    serde_json::json!({
                        "layout": "horizontal",
                        "position": "sticky",
                        "background": "#ffffff",
                        "text_color": "#333333"
                    }),
                    Some("hamburger"),
                    true
                ),
                (
                    "footer", 
                    "Footer Menu",
                    serde_json::json!({
                        "layout": "horizontal",
                        "position": "bottom",
                        "background": "#f8f9fa",
                        "text_color": "#666666"
                    }),
                    None,
                    true
                ),
                (
                    "floating",
                    "Floating Menu", 
                    serde_json::json!({
                        "layout": "vertical",
                        "position": "fixed-right",
                        "background": "#ffffff",
                        "text_color": "#333333"
                    }),
                    None,
                    false
                ),
            ];

            for (area_name_str, display_name_str, settings_val, mobile_behavior_opt, is_active_val) in default_areas {
                let new_area = crate::models::navigation::NewMenuArea {
                    area_name: area_name_str.to_string(),
                    display_name: display_name_str.to_string(),
                    template_id: None,
                    settings: settings_val,
                    mobile_behavior: mobile_behavior_opt.map(|s| s.to_string()),
                    hamburger_icon: if area_name_str == "header" { Some("☰".to_string()) } else { None },
                    is_active: is_active_val,
                };
                
                use crate::schema::menu_areas::dsl::*;
                diesel::insert_into(menu_areas)
                    .values(&new_area)
                    .execute(&mut conn)?;
                info!("Created default menu area: {}", display_name_str);
            }
        }

        // Create default component templates if they don't exist
        use crate::schema::component_templates;
        
        let existing_templates = component_templates::table.load::<ComponentTemplate>(&mut conn)?;
        if existing_templates.is_empty() {
            let default_templates = vec![
                (
                    "Header Template",
                    "header",
                    serde_json::json!({
                        "position": "sticky",
                        "height": "80px",
                        "background": "#ffffff",
                        "container_width": "contained",
                        "navigation_layout": "horizontal",
                        "logo_type": "text",
                        "logo_size": "1.5rem",
                        "mobile_menu": "hamburger",
                        "mobile_breakpoint": "768px"
                    }),
                    serde_json::json!({
                        "mobile": "768px",
                        "tablet": "1024px", 
                        "desktop": "1200px"
                    }),
                    Some("contained"),
                    Some("1200px"),
                    true,
                    true
                ),
                (
                    "Footer Template",
                    "footer",
                    serde_json::json!({
                        "style": "simple",
                        "container_width": "full",
                        "padding": "3rem 0",
                        "navigation_layout": "horizontal",
                        "copyright_position": "center",
                        "copyright_text": "© 2024 My Rust CMS",
                        "additional_text": "Built with Rust & Yew"
                    }),
                    serde_json::json!({
                        "mobile": "768px",
                        "tablet": "1024px",
                        "desktop": "1200px"
                    }),
                    Some("full"),
                    None,
                    true,
                    true
                ),
                (
                    "Sidebar Template",
                    "sidebar",
                    serde_json::json!({
                        "position": "right",
                        "width": "300px",
                        "sticky": true,
                        "mobile_display": "hidden",
                        "mobile_breakpoint": "768px",
                        "sections": ["navigation", "recent_posts"]
                    }),
                    serde_json::json!({
                        "mobile": "768px",
                        "tablet": "1024px",
                        "desktop": "1200px"
                    }),
                    Some("fixed"),
                    Some("300px"),
                    false,
                    false
                ),
                (
                    "Modal Template",
                    "modal",
                    serde_json::json!({
                        "backdrop": "blur",
                        "position": "center",
                        "animation": "fade",
                        "max_width": "600px",
                        "z_index": 1000
                    }),
                    serde_json::json!({
                        "mobile": "95%",
                        "tablet": "80%",
                        "desktop": "600px"
                    }),
                    Some("responsive"),
                    Some("600px"),
                    false,
                    true
                ),
                (
                    "Main Container Template",
                    "main_container",
                    serde_json::json!({
                        "width_type": "fixed",
                        "max_width": "1200px",
                        "padding": "1rem",
                        "grid_system": "css_grid",
                        "responsive": true
                    }),
                    serde_json::json!({
                        "mobile": "100%",
                        "tablet": "90%", 
                        "desktop": "1200px"
                    }),
                    Some("fixed"),
                    Some("1200px"),
                    true,
                    true
                ),
            ];

            for (name_str, component_type_str, template_data_val, breakpoints_val, width_setting_opt, max_width_opt, is_default_val, is_active_val) in default_templates {
                let new_template = crate::models::navigation::NewComponentTemplate {
                    name: name_str.to_string(),
                    component_type: component_type_str.to_string(),
                    template_data: template_data_val,
                    breakpoints: breakpoints_val,
                    width_setting: width_setting_opt.map(|s| s.to_string()),
                    max_width: max_width_opt.map(|s| s.to_string()),
                    is_default: is_default_val,
                    is_active: is_active_val,
                };
                
                use crate::schema::component_templates::dsl::*;
                diesel::insert_into(component_templates)
                    .values(&new_template)
                    .execute(&mut conn)?;
                info!("Created default component template: {}", name_str);
            }
        }
    }

    // Configure CORS with proper security
    let cors = if config.is_development() {
        // Development CORS - more permissive
        CorsLayer::new()
            .allow_origin([
                "http://localhost:3000".parse().unwrap(),  // Frontend dev server
                "http://localhost:8080".parse().unwrap(),  // Frontend prod server  
                "http://127.0.0.1:3000".parse().unwrap(),
                "http://127.0.0.1:8080".parse().unwrap(),
            ])
    } else {
        // Production CORS - strict origins only
        // Replace with your actual production frontend domain
        CorsLayer::new()
            .allow_origin([
                "https://yourdomain.com".parse().unwrap(),
                "https://www.yourdomain.com".parse().unwrap(),
            ])
    }
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/posts", get(controllers::posts::get_posts))
        .route("/api/auth/login", post(controllers::auth::login))
        // TODO: Re-enable rate limiting when API is stabilized
        // .layer(create_auth_rate_limiter())
        .route("/api/posts/:id", get(controllers::posts::get_post))
        .route("/api/categories", get(controllers::admin::get_categories))
        .route("/api/navigation", get(controllers::navigation::get_navigation))
        .route("/api/navigation/area/:area", get(controllers::navigation::get_navigation_by_area))
        .route("/api/menu-areas/:name", get(controllers::navigation::get_menu_area_by_name))
        .route("/api/component-templates", get(controllers::navigation::get_component_templates))
        .route("/api/pages", get(controllers::pages::get_pages))
        .route("/api/pages/:id", get(controllers::pages::get_page))
        .route("/api/pages/slug/:slug", get(controllers::pages::get_page_by_slug))
        .route("/api/test", get(test_endpoint));

    // Authenticated routes (requires valid session)
    let auth_routes = Router::new()
        .route("/api/auth/logout", post(controllers::auth::logout))
        .route("/api/auth/me", get(controllers::auth::get_current_user))
        .route("/api/auth/sessions", get(controllers::sessions::get_user_sessions))
        .route("/api/auth/sessions/logout-all", post(controllers::sessions::logout_all_sessions))
        .layer(axum_middleware::from_fn_with_state(app_services.clone(), auth_middleware_with_services));

    // Admin-only routes (requires admin role)
    let admin_routes = Router::new()
        .route("/api/users", get(controllers::users::get_users).post(controllers::users::create_user))
        .route("/api/users/:id", put(controllers::users::update_user).delete(controllers::users::delete_user))
        .route("/api/posts", post(controllers::posts::create_post))
        .route("/api/posts/:id", put(controllers::posts::update_post).delete(controllers::posts::delete_post))
        .route("/api/comments", get(controllers::comments::get_comments).post(controllers::comments::create_comment))
        .route("/api/comments/:id", put(controllers::comments::update_comment).delete(controllers::comments::delete_comment))
        .route("/api/media", get(controllers::media::get_media))
        .route("/api/media/upload", post(controllers::media::upload_media))
        // TODO: Re-enable upload rate limiting when API is stabilized
        // .layer(create_upload_rate_limiter())
        .route("/api/media/:id", delete(controllers::media::delete_media))
        .route("/api/sessions", get(controllers::admin::get_sessions))
        .route("/api/settings", get(controllers::admin::get_settings))
        .route("/api/templates", get(controllers::admin::get_templates))
        .route("/api/components", get(controllers::admin::get_components))
        .route("/api/navigation", post(controllers::navigation::create_navigation_item))
        .route("/api/navigation/:id", put(controllers::navigation::update_navigation_item).delete(controllers::navigation::delete_navigation_item))
        // Enhanced navigation management routes
        .route("/api/menu-areas", get(controllers::navigation::get_menu_areas))
        .route("/api/menu-areas/:name", put(controllers::navigation::update_menu_area))
        .route("/api/menu-templates", get(controllers::navigation::get_menu_templates).post(controllers::navigation::create_menu_template))
        .route("/api/menu-templates/type/:template_type", get(controllers::navigation::get_menu_templates_by_type))
        .route("/api/component-templates", post(controllers::navigation::create_component_template))
        .route("/api/component-templates/admin", get(controllers::navigation::get_all_component_templates_admin))
        .route("/api/component-templates/:id", put(controllers::navigation::update_component_template))
        .route("/api/component-templates/:id/toggle", post(controllers::navigation::toggle_component_template))
        .route("/api/component-templates/type/:component_type", get(controllers::navigation::get_component_templates_by_type))
        .route("/api/pages", post(controllers::pages::create_page))
        .route("/api/pages/:id", put(controllers::pages::update_page).delete(controllers::pages::delete_page))
        .route("/api/stats", get(controllers::admin::get_stats))
        .route("/api/performance", get(controllers::admin::get_performance_metrics))
        .route("/api/admin/sessions", get(controllers::sessions::get_all_session_stats))
        .route("/api/admin/sessions/cleanup", post(controllers::sessions::manual_session_cleanup))
        .route("/api/admin/users/:id/sessions", get(controllers::sessions::get_admin_user_sessions))
        .route("/api/admin/users/:id/force-logout", post(controllers::sessions::force_logout_user))
        // System management routes
        .route("/api/system/settings", get(controllers::system::get_settings).put(controllers::system::update_settings))
        .route("/api/system/settings/:key", get(controllers::system::get_setting))
        .route("/api/system/info", get(controllers::system::get_system_info))
        .route("/api/system/backup", post(controllers::system::create_backup))
        .route("/api/system/backups", get(controllers::system::list_backups))
        .route("/api/system/backup/:id/restore", post(controllers::system::restore_backup))
        .route("/api/system/snapshot", get(controllers::system::get_data_snapshot))
        .layer(axum_middleware::from_fn_with_state(app_services.clone(), admin_auth_middleware_with_services));

    // Combine all routes
    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes)
        .nest_service("/uploads", tower_http::services::ServeDir::new("backend/uploads"))
        .with_state(app_services.clone())
        .layer(cors)
        .layer(axum_middleware::from_fn_with_state(
            config.clone(),
            security_headers_middleware
        ));

    // Run the server
    let addr = SocketAddr::new(config.backend_host.parse()?, config.backend_port);
    info!("Starting server at http://{}", addr);
    info!("Database URL: {}", config.database_url);
    info!("Environment: {}", config.rust_env);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> impl IntoResponse {
    "My Rust CMS Backend is running!"
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}


// Test endpoint
async fn test_endpoint() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    Ok(axum::Json(serde_json::json!({
        "message": "Backend is working!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
