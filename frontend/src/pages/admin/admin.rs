use yew::prelude::*;
use crate::components::admin::{AdminSidebar, AdminHeader};
use crate::components::admin::sidebar::AdminTab;
use crate::pages::admin::{dashboard::AdminDashboard, post_list::PostList, post_editor::PostEditor, page_builder::PageBuilder, media_library::MediaLibrary, enhanced_user_management::EnhancedUserManagement, comment_moderation::CommentModeration, navigation_manager::NavigationManager, template_manager::TemplateManager, analytics::Analytics, system_settings::SystemSettings, design_system::DesignSystemPage};
use crate::pages::admin::design_system::{AdminColorScheme, apply_admin_css_variables};
use crate::services::auth_service::User;
use crate::services::api_service::get_settings;

#[derive(Properties, PartialEq)]
pub struct AdminProps {
    pub current_tab: AdminTab,
    pub on_public_click: Callback<()>,
    pub on_logout: Callback<()>,
    pub on_tab_change: Callback<AdminTab>,
    pub current_user: Option<User>,
}

#[function_component(Admin)]
pub fn admin(props: &AdminProps) -> Html {
    let on_tab_change = props.on_tab_change.clone();

    // Load saved admin theme from database on component mount and cleanup on unmount
    use_effect_with_deps(|_| {
        wasm_bindgen_futures::spawn_local(async {
            // Try to load the saved theme from database
            match get_settings(Some("theme")).await {
                Ok(theme_settings) => {
                    if !theme_settings.is_empty() {
                        let mut current_theme = "Light Preset".to_string();
                        let mut found_current_theme = false;
                        let mut saved_admin_schemes: std::collections::HashMap<String, AdminColorScheme> = std::collections::HashMap::new();
                        
                        // Process theme settings to find the current theme
                        for setting in theme_settings {
                            match setting.setting_key.as_str() {
                                "theme_current_admin" => {
                                    if let Some(theme_name) = setting.setting_value {
                                        current_theme = theme_name;
                                        found_current_theme = true;
                                        log::info!("🎨 Admin loading saved theme: {}", current_theme);
                                    }
                                },
                                key if key.starts_with("theme_admin_") => {
                                    let theme_name = key.strip_prefix("theme_admin_").unwrap_or("");
                                    if let Some(theme_json) = setting.setting_value {
                                        if let Ok(scheme) = serde_json::from_str::<AdminColorScheme>(&theme_json) {
                                            saved_admin_schemes.insert(theme_name.to_string(), scheme);
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                        
                        // Apply the saved theme if found
                        if found_current_theme {
                            let scheme = match current_theme.as_str() {
                                "Dark Preset" => AdminColorScheme::dark_mode(),
                                "Light Preset" => AdminColorScheme::default(),
                                custom_name => {
                                    saved_admin_schemes.get(custom_name)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            log::warn!("🎨 Custom theme '{}' not found in admin, using light preset", custom_name);
                                            AdminColorScheme::default()
                                        })
                                }
                            };
                            apply_admin_css_variables(&scheme);
                            log::info!("✅ Applied saved admin theme: {}", current_theme);
                        } else {
                            // No current theme setting found, use default
                            let default_scheme = AdminColorScheme::default();
                            apply_admin_css_variables(&default_scheme);
                            log::info!("🎨 No saved theme found, using default light theme");
                        }
                    } else {
                        // No theme settings at all, use default
                        let default_scheme = AdminColorScheme::default();
                        apply_admin_css_variables(&default_scheme);
                        log::info!("🎨 No theme settings in database, using default light theme");
                    }
                },
                Err(err) => {
                    // Database error, use default but don't override if CSS variables already set
                    log::error!("❌ Failed to load admin theme settings: {:?}", err);
                    let default_scheme = AdminColorScheme::default();
                    apply_admin_css_variables(&default_scheme);
                    log::info!("🎨 Using default theme due to database error");
                }
            }
        });
        
        // Cleanup function to restore body styles when leaving admin
        || {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(body) = document.body() {
                    // Remove admin-body class
                    let existing_class = body.class_name();
                    let new_class = existing_class.replace("admin-body", "").trim().to_string();
                    body.set_class_name(&new_class);
                }
            }
        }
    }, ());

    html! {
        <div class="admin-layout">
            <AdminHeader 
                on_public_click={props.on_public_click.clone()}
                on_logout={props.on_logout.clone()}
                current_user={props.current_user.clone()}
            />
            <div class="admin-content">
                <AdminSidebar 
                    on_tab_click={on_tab_change.clone()} 
                    active_tab={props.current_tab.clone()} 
                    on_public_click={props.on_public_click.clone()}
                />
                <main class="admin-main">
                    {match props.current_tab {
                        AdminTab::Dashboard => html! { <AdminDashboard on_navigate={on_tab_change.clone()} /> },
                        AdminTab::Posts => html! { <PostList on_navigate={on_tab_change.clone()} /> },
                        AdminTab::PostCreate => {
                            let on_save = {
                                let on_tab_change = on_tab_change.clone();
                                Callback::from(move |_| on_tab_change.emit(AdminTab::Posts))
                            };
                            let on_cancel = {
                                let on_tab_change = on_tab_change.clone();
                                Callback::from(move |_| on_tab_change.emit(AdminTab::Posts))
                            };
                            html! { <PostEditor post={None} on_save={on_save} on_cancel={on_cancel} /> }
                        },
                        AdminTab::Pages => html! { <PageBuilder /> },
                        AdminTab::Media => html! { <MediaLibrary /> },
                        AdminTab::Users => html! { <EnhancedUserManagement /> },
                        AdminTab::Comments => html! { <CommentModeration /> },
                        AdminTab::Navigation => html! { <NavigationManager /> },
                        AdminTab::Templates => html! { <TemplateManager /> },
                        AdminTab::Analytics => html! { <Analytics /> },
                        AdminTab::DesignSystem => html! { <DesignSystemPage /> },
                        AdminTab::SystemSettings => html! { <SystemSettings /> },
                    }}
                </main>
            </div>
        </div>
    }
} 