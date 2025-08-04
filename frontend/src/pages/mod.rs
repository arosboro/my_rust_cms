pub mod public;
pub mod admin;
pub mod auth;

mod dashboard;
mod post_list;
mod page_builder;
mod media_library;
mod user_management;
mod settings;
mod comment_moderation;
mod not_found;

// Export modules for direct access
// Pages are accessed via module::Page syntax
pub use dashboard::Dashboard;
pub use settings::Settings; 