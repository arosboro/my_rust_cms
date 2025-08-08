use yew::prelude::*;
use wasm_bindgen::JsCast;
use crate::services::navigation_service::{MenuArea, ComponentTemplate, NavigationItem, get_menu_areas, get_component_templates, get_all_component_templates_admin, update_menu_area, update_component_template, get_navigation_by_area, toggle_component_template};
use crate::services::api_service::{SettingData, get_settings, update_settings};

#[derive(Clone, PartialEq)]
pub enum TemplateView {
    MenuAreas,
    ComponentTemplates,
    ContainerSettings,
}



#[derive(Clone, PartialEq)]
pub struct ContainerSettings {
    // Breakpoints
    pub mobile_breakpoint: String,
    pub tablet_breakpoint: String,
    pub desktop_breakpoint: String,
    pub wide_breakpoint: String,
    
    // Typography
    pub base_font_size: String,
    pub scale_ratio: String,
    pub line_height: String,
    
    // Container
    pub width_type: String,
    pub max_width: String,
    pub horizontal_padding: String,
}

impl Default for ContainerSettings {
    fn default() -> Self {
        Self {
            mobile_breakpoint: "768px".to_string(),
            tablet_breakpoint: "1024px".to_string(),
            desktop_breakpoint: "1200px".to_string(),
            wide_breakpoint: "1440px".to_string(),
            base_font_size: "16px".to_string(),
            scale_ratio: "1.25".to_string(),
            line_height: "1.5".to_string(),
            width_type: "fixed".to_string(),
            max_width: "1200px".to_string(),
            horizontal_padding: "1rem".to_string(),
        }
    }
}



#[function_component(TemplateManager)]
pub fn template_manager() -> Html {
    let current_view = use_state(|| TemplateView::MenuAreas);
    let menu_areas = use_state(Vec::<MenuArea>::new);

    let component_templates = use_state(Vec::<ComponentTemplate>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Load initial data
    {
        let menu_areas = menu_areas.clone();
        let component_templates = component_templates.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                
                // Check authentication first
                if let Err(_) = crate::services::auth_service::get_auth_token() {
                    error.set(Some("Please log in to access the admin panel".to_string()));
                    loading.set(false);
                    return;
                }
                
                // Load template data in parallel
                log::info!("🔐 Authentication check passed, loading template data...");
                let areas_result = get_menu_areas().await;
                let components_result = get_all_component_templates_admin().await;
                
                match (areas_result, components_result) {
                    (Ok(areas), Ok(components)) => {
                        log::info!("✅ Successfully loaded {} menu areas and {} component templates", areas.len(), components.len());
                        menu_areas.set(areas);
                        component_templates.set(components);
                        loading.set(false);
                    }
                    (Err(areas_err), Ok(_)) => {
                        log::error!("❌ Failed to load menu areas: {:?}", areas_err);
                        error.set(Some("Failed to load menu areas".to_string()));
                        loading.set(false);
                    }
                    (Ok(_), Err(components_err)) => {
                        log::error!("❌ Failed to load component templates: {:?}", components_err);
                        
                        // Try fallback to public endpoint if auth failed
                        if components_err.to_string().contains("Not authenticated") {
                            log::info!("🔄 Trying fallback to public component templates...");
                            match get_component_templates().await {
                                Ok(fallback_components) => {
                                    log::info!("✅ Loaded {} public component templates", fallback_components.len());
                                    component_templates.set(fallback_components);
                                    error.set(Some("Loaded in read-only mode. Please log in for full access.".to_string()));
                                    loading.set(false);
                                    return;
                                }
                                Err(fallback_err) => {
                                    log::error!("❌ Fallback also failed: {:?}", fallback_err);
                                }
                            }
                        }
                        
                        error.set(Some("Failed to load component templates".to_string()));
                        loading.set(false);
                    }
                    (Err(areas_err), Err(components_err)) => {
                        log::error!("❌ Failed to load both menu areas and component templates. Areas: {:?}, Components: {:?}", areas_err, components_err);
                        
                        // Check if it's an auth issue and try fallback for components
                        if components_err.to_string().contains("Not authenticated") {
                            log::info!("🔄 Trying fallback to public component templates...");
                            match get_component_templates().await {
                                Ok(fallback_components) => {
                                    log::info!("✅ Loaded {} public component templates", fallback_components.len());
                                    component_templates.set(fallback_components);
                                    menu_areas.set(Vec::new()); // Set empty menu areas
                                    error.set(Some("Authentication required. Limited functionality available.".to_string()));
                                    loading.set(false);
                                    return;
                                }
                                Err(fallback_err) => {
                                    log::error!("❌ Fallback also failed: {:?}", fallback_err);
                                }
                            }
                        }
                        
                        error.set(Some("Failed to load template data".to_string()));
                        loading.set(false);
                    }
                }
            });
            || ()
        }, ());
    }

    let switch_to_areas = {
        let current_view = current_view.clone();
        Callback::from(move |_| current_view.set(TemplateView::MenuAreas))
    };

    let switch_to_components = {
        let current_view = current_view.clone();
        Callback::from(move |_| current_view.set(TemplateView::ComponentTemplates))
    };

    let switch_to_container = {
        let current_view = current_view.clone();
        Callback::from(move |_| current_view.set(TemplateView::ContainerSettings))
    };

    html! {
        <div class="template-manager">
            <div class="page-header">
                <div>
                    <h1>{"Template Manager"}</h1>
                    <p>{"Configure menu areas, component templates, and global container settings"}</p>
                </div>
            </div>

            {if let Some(ref error_msg) = *error {
                html! {
                    <div class="error-message">
                        <strong>{"Error: "}</strong>{error_msg}
                    </div>
                }
            } else {
                html! {}
            }}

            <div class="template-tabs">
                <button 
                    class={if *current_view == TemplateView::MenuAreas { "tab-button active" } else { "tab-button" }}
                    onclick={switch_to_areas}
                >
                    {"📍 Menu Areas"}
                </button>
                <button 
                    class={if *current_view == TemplateView::ComponentTemplates { "tab-button active" } else { "tab-button" }}
                    onclick={switch_to_components}
                >
                    {"🧩 Component Templates"}
                </button>
                <button 
                    class={if *current_view == TemplateView::ContainerSettings { "tab-button active" } else { "tab-button" }}
                    onclick={switch_to_container}
                >
                    {"📦 Container Settings"}
                </button>
            </div>

            <div class="template-content">
                {if *loading {
                    html! {
                        <div class="loading">
                            <div class="loading-spinner"></div>
                            <p>{"Loading template configuration..."}</p>
                        </div>
                    }
                } else {
                    match (*current_view).clone() {
                        TemplateView::MenuAreas => html! { 
                            <MenuAreasView 
                                menu_areas={(*menu_areas).clone()}
                                on_toggle={{
                                    let menu_areas = menu_areas.clone();
                                    let error = error.clone();
                                    Callback::from(move |(area_name, is_active): (String, bool)| {
                                        web_sys::console::log_1(&format!("Toggle {} to {}", area_name, is_active).into());
                                        
                                        // Update local state immediately for responsive UI
                                        let mut areas = (*menu_areas).clone();
                                        if let Some(area) = areas.iter_mut().find(|a| a.area_name == area_name) {
                                            area.is_active = is_active;
                                            let updated_area = area.clone(); // Clone before setting state
                                            menu_areas.set(areas.clone());
                                            
                                            // Persist to backend
                                            let menu_areas_clone = menu_areas.clone();
                                            let error_clone = error.clone();
                                            let area_name_clone = area_name.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                match update_menu_area(&area_name_clone, &updated_area).await {
                                                    Ok(_) => {
                                                        web_sys::console::log_1(&format!("Successfully updated {} area", area_name_clone).into());
                                                    }
                                                    Err(e) => {
                                                        error_clone.set(Some(format!("Failed to update {}: {:?}", area_name_clone, e)));
                                                        // Revert local state on error
                                                        let mut reverted_areas = areas;
                                                        if let Some(revert_area) = reverted_areas.iter_mut().find(|a| a.area_name == area_name_clone) {
                                                            revert_area.is_active = !is_active;
                                                            menu_areas_clone.set(reverted_areas);
                                                        }
                                                    }
                                                }
                                            });
                                        } else {
                                            // Handle standard areas that might not exist in backend yet
                                            error.set(Some(format!("Area '{}' not found in backend", area_name)));
                                        }
                                    })
                                }}
                            /> 
                        },
                        TemplateView::ComponentTemplates => html! { 
                            <ComponentTemplatesView 
                                component_templates={(*component_templates).clone()}
                                on_modify={Callback::noop()}
                                on_template_toggled={{
                                    let component_templates = component_templates.clone();
                                    Callback::from(move |updated_template: ComponentTemplate| {
                                        // Update the component template in the local state
                                        let mut templates = (*component_templates).clone();
                                        if let Some(index) = templates.iter().position(|t| t.id == updated_template.id) {
                                            templates[index] = updated_template;
                                            component_templates.set(templates);
                                        }
                                    })
                                }}
                            /> 
                        },
                        TemplateView::ContainerSettings => html! { <ContainerSettingsView /> },
                    }
                }}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MenuAreasViewProps {
    pub menu_areas: Vec<MenuArea>,
    pub on_toggle: Callback<(String, bool)>,
}

#[function_component(MenuAreasView)]
pub fn menu_areas_view(props: &MenuAreasViewProps) -> Html {
    let get_area_info = |area_name: &str| -> (String, String, String, bool) {
        if let Some(area) = props.menu_areas.iter().find(|a| a.area_name == area_name) {
            (
                area.display_name.clone(),
                format!("Status: {}", if area.is_active { "Active" } else { "Inactive" }),
                if area.is_active { "area-status active" } else { "area-status inactive" }.to_string(),
                area.is_active
            )
        } else {
            // Default data for standard areas
            match area_name {
                "header" => ("Header Menu".to_string(), "Status: Active".to_string(), "area-status active".to_string(), true),
                "footer" => ("Footer Menu".to_string(), "Status: Active".to_string(), "area-status active".to_string(), true),
                "floating" => ("Floating Menu".to_string(), "Status: Disabled".to_string(), "area-status inactive".to_string(), false),
                _ => ("Unknown".to_string(), "Status: Unknown".to_string(), "area-status".to_string(), false)
            }
        }
    };

    let handle_toggle = {
        let on_toggle = props.on_toggle.clone();
        Callback::from(move |(area_name, is_active): (String, bool)| {
            // For now, just call the callback - the parent will handle the actual API call
            on_toggle.emit((area_name, is_active));
        })
    };

    html! {
        <div class="menu-areas-section">
            <h2>{"Menu Areas Configuration"}</h2>
            <p>{"Enable or disable different menu areas for your site"}</p>
            
            <div class="area-cards">
                <div class="area-card">
                    <div class="area-header">
                        <h3>{"📱 Header Menu"}</h3>
                        <div class="area-toggle">
                            <label class="toggle-switch">
                                <input 
                                    type="checkbox"
                                    checked={get_area_info("header").3}
                                    onchange={
                                        let handle_toggle = handle_toggle.clone();
                                        Callback::from(move |e: Event| {
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                                            handle_toggle.emit(("header".to_string(), target.checked()));
                                        })
                                    }
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    <p>{"Main navigation with mobile hamburger support"}</p>
                    <div class={get_area_info("header").2}>{get_area_info("header").1}</div>
                </div>
                
                <div class="area-card">
                    <div class="area-header">
                        <h3>{"🦶 Footer Menu"}</h3>
                        <div class="area-toggle">
                            <label class="toggle-switch">
                                <input 
                                    type="checkbox"
                                    checked={get_area_info("footer").3}
                                    onchange={
                                        let handle_toggle = handle_toggle.clone();
                                        Callback::from(move |e: Event| {
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                                            handle_toggle.emit(("footer".to_string(), target.checked()));
                                        })
                                    }
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    <p>{"Footer navigation with layout options"}</p>
                    <div class={get_area_info("footer").2}>{get_area_info("footer").1}</div>
                </div>
                
                <div class="area-card">
                    <div class="area-header">
                        <h3>{"🎈 Floating Menu"}</h3>
                        <div class="area-toggle">
                            <label class="toggle-switch">
                                <input 
                                    type="checkbox"
                                    checked={get_area_info("floating").3}
                                    onchange={
                                        let handle_toggle = handle_toggle.clone();
                                        Callback::from(move |e: Event| {
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                                            handle_toggle.emit(("floating".to_string(), target.checked()));
                                        })
                                    }
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    <p>{"Floating navigation elements"}</p>
                    <div class={get_area_info("floating").2}>{get_area_info("floating").1}</div>
                </div>

                // Show custom menu areas
                {for props.menu_areas.iter().filter(|area| area.area_name.starts_with("custom_")).map(|area| {
                    html! {
                        <div class="area-card">
                            <div class="area-header">
                                <h3>{format!("🧩 {}", area.display_name)}</h3>
                                <div class="area-toggle">
                                    <label class="toggle-switch">
                                        <input 
                                            type="checkbox"
                                            checked={area.is_active}
                                            onchange={
                                                let handle_toggle = handle_toggle.clone();
                                                let area_name = area.area_name.clone();
                                                Callback::from(move |e: Event| {
                                                    let target = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                                                    handle_toggle.emit((area_name.clone(), target.checked()));
                                                })
                                            }
                                        />
                                        <span class="slider"></span>
                                    </label>
                                </div>
                            </div>
                            <p>{"Custom menu for page builder integration"}</p>
                            <div class={if area.is_active { "area-status active" } else { "area-status inactive" }}>
                                {format!("Status: {}", if area.is_active { "Active" } else { "Inactive" })}
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}



// Helper functions for rendering component templates
fn render_component_preview(component_type: &str, header_navigation: &UseStateHandle<Vec<NavigationItem>>, footer_navigation: &UseStateHandle<Vec<NavigationItem>>) -> Html {
    match component_type {
        "Comments" => html! {
            <div class="live-comments-preview">
                <div class="comment-bubble">
                    <div class="comment-avatar">
                        <img src="https://www.gravatar.com/avatar/c5d59b67e7a7eb5b7e5e1e5e2e5e3e5e?s=32&d=identicon&r=pg" 
                             alt="Avatar" style="width: 32px; height: 32px; border-radius: 50%;" />
                    </div>
                    <div class="comment-content">
                        <div class="comment-header">
                            <span class="comment-author">{"John Doe"}</span>
                            <span class="comment-time">{"2 hours ago"}</span>
                        </div>
                        <div class="comment-text">
                            <p>{"Great post! Really helpful content."}</p>
                        </div>
                    </div>
                </div>
                <div class="comment-form-preview">
                    <div style="padding: 8px; background: #f5f5f5; border-radius: 8px; font-size: 12px; text-align: center; color: #666;">
                        {"💬 Comment Form (Login Required)"}
                    </div>
                </div>
            </div>
        },
        "header" => html! {
            <div class="live-header-preview">
                <div class="live-nav-bar">
                    <span class="live-logo">{"🏠 My Site"}</span>
                    <div class="live-nav-items">
                        {
                            if (*header_navigation).is_empty() {
                                html! {
                                    <>
                                        <span>{"Home"}</span>
                                        <span>{"Posts"}</span>
                                        <span>{"About"}</span>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        {for (*header_navigation).iter().take(4).map(|item| {
                                            html! { <span>{&item.title}</span> }
                                        })}
                                    </>
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        },
        "footer" => html! {
            <div class="live-footer-preview">
                <div class="live-footer-content">
                    <div class="live-footer-nav">
                        {
                            if (*footer_navigation).is_empty() {
                                html! {
                                    <>
                                        <span>{"Privacy"}</span>
                                        <span>{"Terms"}</span>
                                        <span>{"Contact"}</span>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        {for (*footer_navigation).iter().take(4).map(|item| {
                                            html! { <span>{&item.title}</span> }
                                        })}
                                    </>
                                }
                            }
                        }
                    </div>
                    <p>{"© 2024 My Rust CMS - Built with Rust & Yew"}</p>
                </div>
            </div>
        },
        "sidebar" => html! {
            <div class="live-sidebar-preview">
                <div class="sidebar-header">{"📋 Sidebar"}</div>
                <div class="sidebar-items">
                    <div class="sidebar-item">{"Navigation"}</div>
                    <div class="sidebar-item">{"Recent Posts"}</div>
                    <div class="sidebar-item">{"Categories"}</div>
                    <div class="sidebar-item">{"Archives"}</div>
                </div>
            </div>
        },
        "modal" => html! {
            <div class="live-modal-preview">
                <div class="modal-backdrop">
                    <div class="modal-content">
                        <div class="modal-header">
                            <span>{"Modal Title"}</span>
                            <span class="modal-close">{"×"}</span>
                        </div>
                        <div class="modal-body">{"Content area"}</div>
                        <div class="modal-footer">
                            <button class="modal-btn">{"Cancel"}</button>
                            <button class="modal-btn primary">{"Confirm"}</button>
                        </div>
                    </div>
                </div>
            </div>
        },
        "main_container" => html! {
            <div class="live-container-preview">
                <div class="container-outline">
                    <div class="container-header">{"📦 Main Container"}</div>
                    <div class="container-content">
                        <div class="content-block">{"Header"}</div>
                        <div class="content-block main">{"Main Content Area"}</div>
                        <div class="content-block">{"Footer"}</div>
                    </div>
                    <div class="container-info">{"Max-width: 1200px"}</div>
                </div>
            </div>
        },
        _ => html! {
            <div class="generic-component-preview">
                <div class="preview-placeholder">
                    <span>{"Component Preview"}</span>
                </div>
            </div>
        }
    }
}

fn get_component_title(component_type: &str) -> &'static str {
    match component_type {
        "header" => "🎯 Header Component",
        "footer" => "📍 Footer Component", 
        "sidebar" => "📋 Sidebar Component",
        "modal" => "🪟 Modal Component",
        "main_container" => "📦 Container Component",
        "Comments" => "💬 Comments Component",
        _ => "🧩 Component Template"
    }
}

fn get_component_description(component_type: &str) -> &'static str {
    match component_type {
        "header" => "Main site header with navigation, logo, and mobile responsive design",
        "footer" => "Site footer with navigation links, copyright, and customizable layout options",
        "sidebar" => "Configurable sidebar for additional navigation, widgets, and content areas",
        "modal" => "Overlay modals for forms, dialogs, and interactive content with backdrop styling",
        "main_container" => "Main content container with width, padding, and responsive layout settings",
        "Comments" => "Interactive comment system with Gravatar avatars, text message styling, and authentication",
        _ => "Customizable component template with configurable properties"
    }
}

fn get_component_name(component_type: &str) -> &'static str {
    match component_type {
        "header" => "Header",
        "footer" => "Footer",
        "sidebar" => "Sidebar",
        "modal" => "Modal",
        "main_container" => "Container",
        "Comments" => "Comments",
        _ => "Component"
    }
}

fn render_component_tags(component_type: &str) -> Html {
    match component_type {
        "header" => html! {
            <>
                <span class="property-tag">{"Sticky Position"}</span>
                <span class="property-tag">{"1200px Max"}</span>
                <span class="property-tag">{"Mobile Responsive"}</span>
            </>
        },
        "footer" => html! {
            <>
                <span class="property-tag">{"Full Width"}</span>
                <span class="property-tag">{"Horizontal Layout"}</span>
                <span class="property-tag">{"Responsive"}</span>
            </>
        },
        "Comments" => html! {
            <>
                <span class="property-tag">{"Gravatar Avatars"}</span>
                <span class="property-tag">{"Text Message Style"}</span>
                <span class="property-tag">{"Authentication"}</span>
            </>
        },
        "sidebar" => html! {
            <>
                <span class="property-tag">{"250px Width"}</span>
                <span class="property-tag">{"Left/Right Position"}</span>
                <span class="property-tag">{"Collapsible"}</span>
            </>
        },
        "modal" => html! {
            <>
                <span class="property-tag">{"Center Positioned"}</span>
                <span class="property-tag">{"Backdrop Blur"}</span>
                <span class="property-tag">{"Animation"}</span>
            </>
        },
        "main_container" => html! {
            <>
                <span class="property-tag">{"Max Width"}</span>
                <span class="property-tag">{"Auto Margins"}</span>
                <span class="property-tag">{"Responsive"}</span>
            </>
        },
        _ => html! {
            <>
                <span class="property-tag">{"Configurable"}</span>
                <span class="property-tag">{"Custom Layout"}</span>
            </>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ComponentTemplatesViewProps {
    pub component_templates: Vec<ComponentTemplate>,
    pub on_modify: Callback<(String, String)>, // (component_id, property)
    pub on_template_toggled: Callback<ComponentTemplate>, // Called when template is toggled
}

#[function_component(ComponentTemplatesView)]
pub fn component_templates_view(props: &ComponentTemplatesViewProps) -> Html {
    let editing_component = use_state(|| None::<String>);
    let editing_template = use_state(|| None::<ComponentTemplate>);
    let saving = use_state(|| false);
    let save_error = use_state(|| None::<String>);
    
    // Navigation items for live previews
    let header_navigation = use_state(Vec::<NavigationItem>::new);
    let footer_navigation = use_state(Vec::<NavigationItem>::new);

    // Load navigation items for previews
    {
        let header_navigation = header_navigation.clone();
        let footer_navigation = footer_navigation.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Load header navigation
                if let Ok(header_items) = get_navigation_by_area("header").await {
                    header_navigation.set(header_items);
                }
                
                // Load footer navigation  
                if let Ok(footer_items) = get_navigation_by_area("footer").await {
                    footer_navigation.set(footer_items);
                }
            });
            || ()
        }, ());
    }

    let close_editor = {
        let editing_component = editing_component.clone();
        let editing_template = editing_template.clone();
        let save_error = save_error.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            editing_component.set(None);
            editing_template.set(None);
            save_error.set(None);
        })
    };

    let save_template = {
        let editing_template = editing_template.clone();
        let saving = saving.clone();
        let save_error = save_error.clone();
        let close_editor = close_editor.clone();
        Callback::from(move |_| {
            if let Some(template) = (*editing_template).clone() {
                saving.set(true);
                save_error.set(None);
                let saving_clone = saving.clone();
                let save_error_clone = save_error.clone();
                let close_editor_clone = close_editor.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    match update_component_template(template.id, &template).await {
                        Ok(_) => {
                            web_sys::console::log_1(&"Component template saved successfully".into());
                            saving_clone.set(false);
                            // Close editor programmatically - create a dummy mouse event
                            if let Some(_window) = web_sys::window() {
                                if let Ok(event) = web_sys::MouseEvent::new("click") {
                                    close_editor_clone.emit(event);
                                }
                            }
                        }
                        Err(e) => {
                            save_error_clone.set(Some(format!("Failed to save template: {:?}", e)));
                            saving_clone.set(false);
                        }
                    }
                });
            }
        })
    };

    // Helper function to update template data
    let update_template_data = {
        let editing_template = editing_template.clone();
        Callback::from(move |(key, value): (String, serde_json::Value)| {
            if let Some(mut template) = (*editing_template).clone() {
                if let Some(data) = template.template_data.as_object_mut() {
                    data.insert(key, value);
                    editing_template.set(Some(template));
                }
            }
        })
    };

    html! {
        <div class="component-templates-section">
            <h2>{"Component Templates"}</h2>
            <p>{"Manage templates for major layout components with live styling"}</p>
            
            {if let Some(ref component_id) = *editing_component {
                html! {
                    <div class="editor-modal">
                        <div class="editor-overlay" onclick={close_editor.clone()}></div>
                        <div class="editor-panel">
                            <div class="editor-header">
                                <h3>{format!("Edit {} Component", component_id.replace("_", " "))}</h3>
                                <button class="close-btn" onclick={close_editor.clone()}>{"×"}</button>
                            </div>
                            <div class="editor-content">
                                <div class="editor-sidebar">
                                    {match component_id.as_str() {
                                        "header" => html! {
                                            <>
                                                <div class="property-group">
                                                    <h4>{"Header Layout"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Position"}</label>
                                                        <select 
                                                            class="property-select"
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("position".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        >
                                                            <option value="static">{"Static"}</option>
                                                            <option value="sticky" selected=true>{"Sticky"}</option>
                                                            <option value="fixed">{"Fixed"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Container Width"}</label>
                                                        <select class="property-select">
                                                            <option value="full">{"Full Width"}</option>
                                                            <option value="contained" selected=true>{"Container (1200px)"}</option>
                                                            <option value="fluid">{"Fluid"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Height"}</label>
                                                        <input 
                                                            type="text" 
                                                            value={{
                                                                if let Some(template) = (*editing_template).as_ref() {
                                                                    template.template_data.get("height")
                                                                        .and_then(|v| v.as_str())
                                                                        .unwrap_or("80px")
                                                                        .to_string()
                                                                } else {
                                                                    "80px".to_string()
                                                                }
                                                            }}
                                                            class="property-input"
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("height".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                            onblur={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: FocusEvent| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("height".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Z-Index"}</label>
                                                        <input type="number" value="1000" class="property-input" />
                                                    </div>
                                                </div>
                                                
                                                <div class="property-group">
                                                    <h4>{"Navigation Style"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Navigation Layout"}</label>
                                                        <select class="property-select">
                                                            <option value="horizontal" selected=true>{"Horizontal"}</option>
                                                            <option value="centered">{"Centered"}</option>
                                                            <option value="split">{"Logo Left / Menu Right"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Item Spacing"}</label>
                                                        <input type="text" value="2rem" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Hover Effect"}</label>
                                                        <select class="property-select">
                                                            <option value="underline" selected=true>{"Underline"}</option>
                                                            <option value="background">{"Background Color"}</option>
                                                            <option value="scale">{"Scale"}</option>
                                                            <option value="none">{"None"}</option>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Logo Settings"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Logo Type"}</label>
                                                        <select class="property-select">
                                                            <option value="text" selected=true>{"Text Logo"}</option>
                                                            <option value="image">{"Image Logo"}</option>
                                                            <option value="icon">{"Icon + Text"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Logo Size"}</label>
                                                        <input type="text" value="1.5rem" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Logo Font Weight"}</label>
                                                        <select class="property-select">
                                                            <option value="400">{"Normal"}</option>
                                                            <option value="600" selected=true>{"Semi-Bold"}</option>
                                                            <option value="700">{"Bold"}</option>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Mobile Behavior"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Mobile Menu Style"}</label>
                                                        <select class="property-select">
                                                            <option value="hamburger" selected=true>{"Hamburger Menu"}</option>
                                                            <option value="dots">{"Three Dots"}</option>
                                                            <option value="hidden">{"Hide Menu"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Mobile Breakpoint"}</label>
                                                        <input type="text" value="768px" class="property-input" />
                                                    </div>
                                                </div>
                                            </>
                                        },
                                        "footer" => html! {
                                            <>
                                                <div class="property-group">
                                                    <h4>{"Footer Layout"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Footer Style"}</label>
                                                        <select class="property-select">
                                                            <option value="simple" selected=true>{"Simple"}</option>
                                                            <option value="multi-column">{"Multi-Column"}</option>
                                                            <option value="centered">{"Centered"}</option>
                                                            <option value="minimal">{"Minimal"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Container Width"}</label>
                                                        <select class="property-select">
                                                            <option value="full" selected=true>{"Full Width"}</option>
                                                            <option value="contained">{"Container (1200px)"}</option>
                                                            <option value="fluid">{"Fluid"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Vertical Padding"}</label>
                                                        <input type="text" value="3rem 0" class="property-input" />
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Footer Navigation"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Navigation Layout"}</label>
                                                        <select class="property-select">
                                                            <option value="horizontal" selected=true>{"Horizontal"}</option>
                                                            <option value="vertical">{"Vertical"}</option>
                                                            <option value="grid">{"Grid (2x2)"}</option>
                                                            <option value="hidden">{"Hidden"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Link Spacing"}</label>
                                                        <input type="text" value="1.5rem" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Separator"}</label>
                                                        <select class="property-select">
                                                            <option value="none" selected=true>{"None"}</option>
                                                            <option value="pipe">{"Pipe (|)"}</option>
                                                            <option value="dot">{"Dot (•)"}</option>
                                                            <option value="line">{"Divider Line"}</option>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Copyright & Text"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Copyright Position"}</label>
                                                        <select class="property-select">
                                                            <option value="center" selected=true>{"Center"}</option>
                                                            <option value="left">{"Left"}</option>
                                                            <option value="right">{"Right"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Copyright Text"}</label>
                                                        <input type="text" value="© 2024 My Rust CMS" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Additional Text"}</label>
                                                        <input type="text" value="Built with Rust & Yew" class="property-input" />
                                                    </div>
                                                </div>
                                            </>
                                        },
                                        "sidebar" => html! {
                                            <>
                                                <div class="property-group">
                                                    <h4>{"Sidebar Layout"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Position"}</label>
                                                        <select 
                                                            class="property-select"
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("position".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        >
                                                            <>
                                                                    <option value="left" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("position").and_then(|v| v.as_str()))
                                                                            .unwrap_or("right") == "left"
                                                                    }>{"Left Side"}</option>
                                                                    <option value="right" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("position").and_then(|v| v.as_str()))
                                                                            .unwrap_or("right") == "right"
                                                                    }>{"Right Side"}</option>
                                                                    <option value="both" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("position").and_then(|v| v.as_str()))
                                                                            .unwrap_or("right") == "both"
                                                                    }>{"Both Sides"}</option>
                                                            </>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Width"}</label>
                                                        <input 
                                                            type="text" 
                                                            class="property-input"
                                                            value={{
                                                                if let Some(t) = (*editing_template).as_ref() {
                                                                    t.template_data.get("width").and_then(|v| v.as_str()).unwrap_or("300px").to_string()
                                                                } else { "300px".to_string() }
                                                            }}
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("width".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                            onblur={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: FocusEvent| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("width".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Sticky"}</label>
                                                        <select 
                                                            class="property-select"
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                                                                        let value = target.value();
                                                                        let is_sticky = value == "true";
                                                                        update_template_data.emit(("sticky".to_string(), serde_json::Value::Bool(is_sticky)));
                                                                    }
                                                                })
                                                            }}
                                                        >
                                                            <>
                                                                    <option value="true" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("sticky").and_then(|v| v.as_bool()))
                                                                            .unwrap_or(true)
                                                                    }>{"Sticky"}</option>
                                                                    <option value="false" selected={
                                                                        !((*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("sticky").and_then(|v| v.as_bool()))
                                                                            .unwrap_or(true))
                                                                    }>{"Not Sticky"}</option>
                                                            </>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Mobile Behavior"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Mobile Display"}</label>
                                                        <select 
                                                            class="property-select"
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("mobile_display".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        >
                                                            <>
                                                                    <option value="hidden" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("mobile_display").and_then(|v| v.as_str()))
                                                                            .unwrap_or("hidden") == "hidden"
                                                                    }>{"Hidden"}</option>
                                                                    <option value="bottom" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("mobile_display").and_then(|v| v.as_str()))
                                                                            .unwrap_or("hidden") == "bottom"
                                                                    }>{"Move to Bottom"}</option>
                                                                    <option value="drawer" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("mobile_display").and_then(|v| v.as_str()))
                                                                            .unwrap_or("hidden") == "drawer"
                                                                    }>{"Slide-out Drawer"}</option>
                                                                    <option value="accordion" selected={
                                                                        (*editing_template)
                                                                            .as_ref()
                                                                            .and_then(|t| t.template_data.get("mobile_display").and_then(|v| v.as_str()))
                                                                            .unwrap_or("hidden") == "accordion"
                                                                    }>{"Collapsible"}</option>
                                                            </>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Mobile Breakpoint"}</label>
                                                        <input 
                                                            type="text" 
                                                            class="property-input"
                                                            value={{
                                                                if let Some(t) = (*editing_template).as_ref() {
                                                                    t.template_data.get("mobile_breakpoint").and_then(|v| v.as_str()).unwrap_or("768px").to_string()
                                                                } else { "768px".to_string() }
                                                            }}
                                                            onchange={{
                                                                let update_template_data = update_template_data.clone();
                                                                Callback::from(move |e: Event| {
                                                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                        let value = target.value();
                                                                        update_template_data.emit(("mobile_breakpoint".to_string(), serde_json::Value::String(value)));
                                                                    }
                                                                })
                                                            }}
                                                        />
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Content Sections"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Default Sections"}</label>
                                                        <div class="checkbox-list">
                                                            <label class="checkbox-item">
                                                                <input type="checkbox" disabled=true checked=true />
                                                                <span>{"Navigation Links"}</span>
                                                            </label>
                                                            <label class="checkbox-item">
                                                                <input type="checkbox" disabled=true checked=true />
                                                                <span>{"Recent Posts"}</span>
                                                            </label>
                                                            <label class="checkbox-item">
                                                                <input type="checkbox" disabled=true />
                                                                <span>{"Categories"}</span>
                                                            </label>
                                                            <label class="checkbox-item">
                                                                <input type="checkbox" disabled=true />
                                                                <span>{"Archives"}</span>
                                                            </label>
                                                        </div>
                                                    </div>
                                                </div>
                                            </>
                                        },
                                        "modal" => html! {
                                            <>
                                                <div class="property-group">
                                                    <h4>{"Modal Behavior"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Animation Style"}</label>
                                                        <select class="property-select">
                                                            <option value="fade" selected=true>{"Fade In/Out"}</option>
                                                            <option value="scale">{"Scale In/Out"}</option>
                                                            <option value="slide-down">{"Slide Down"}</option>
                                                            <option value="slide-up">{"Slide Up"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Animation Duration"}</label>
                                                        <input type="text" value="300ms" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Close on Backdrop Click"}</label>
                                                        <select class="property-select">
                                                            <option value="true" selected=true>{"Yes"}</option>
                                                            <option value="false">{"No"}</option>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Modal Sizing"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Default Size"}</label>
                                                        <select class="property-select">
                                                            <option value="small">{"Small (400px)"}</option>
                                                            <option value="medium" selected=true>{"Medium (600px)"}</option>
                                                            <option value="large">{"Large (800px)"}</option>
                                                            <option value="xl">{"Extra Large (1000px)"}</option>
                                                        </select>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Max Height"}</label>
                                                        <input type="text" value="90vh" class="property-input" />
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Mobile Behavior"}</label>
                                                        <select class="property-select">
                                                            <option value="responsive" selected=true>{"Responsive"}</option>
                                                            <option value="fullscreen">{"Full Screen"}</option>
                                                            <option value="bottom-sheet">{"Bottom Sheet"}</option>
                                                        </select>
                                                    </div>
                                                </div>

                                                <div class="property-group">
                                                    <h4>{"Backdrop & Overlay"}</h4>
                                                    <div class="property-item">
                                                        <label>{"Backdrop Blur"}</label>
                                                        <input type="range" min="0" max="10" value="4" class="property-slider" />
                                                        <span class="slider-value">{"4px"}</span>
                                                    </div>
                                                    <div class="property-item">
                                                        <label>{"Backdrop Opacity"}</label>
                                                        <input type="range" min="0" max="100" value="50" class="property-slider" />
                                                        <span class="slider-value">{"50%"}</span>
                                                    </div>
                                                </div>
                                            </>
                                        },

                                        _ => html! {
                                            <div class="property-group">
                                                <h4>{"Component Properties"}</h4>
                                                <p>{"Select a component to customize its properties."}</p>
                                            </div>
                                        }
                                    }}
                                </div>
                                
                                <div class="editor-preview">
                                    <h4>{"Live Preview"}</h4>
                                    <div class="preview-container">
                                        {match component_id.as_str() {
                                            "header" => html! {
                                                <div class="preview-header">
                                                    <div class="preview-nav">
                                                        <span class="preview-logo">{"🏠 My Site"}</span>
                                                        <div class="preview-menu">
                                                            {
                                                                if (*header_navigation).is_empty() {
                                                                    html! {
                                                                        <>
                                                                            <span class="preview-item">{"Home"}</span>
                                                                            <span class="preview-item">{"Posts"}</span>
                                                                            <span class="preview-item">{"About"}</span>
                                                                        </>
                                                                    }
                                                                } else {
                                                                    html! {
                                                                        <>
                                                                            {for (*header_navigation).iter().take(5).map(|item| {
                                                                                html! { <span class="preview-item">{&item.title}</span> }
                                                                            })}
                                                                        </>
                                                                    }
                                                                }
                                                            }
                                                        </div>
                                                    </div>
                                                </div>
                                            },
                                            "footer" => html! {
                                                <div class="preview-footer">
                                                    <div class="preview-footer-content">
                                                        <div class="preview-footer-nav">
                                                            {
                                                                if (*footer_navigation).is_empty() {
                                                                    html! {
                                                                        <>
                                                                            <span class="preview-footer-item">{"Privacy"}</span>
                                                                            <span class="preview-footer-item">{"Terms"}</span>
                                                                            <span class="preview-footer-item">{"Contact"}</span>
                                                                        </>
                                                                    }
                                                                } else {
                                                                    html! {
                                                                        <>
                                                                            {for (*footer_navigation).iter().take(4).map(|item| {
                                                                                html! { <span class="preview-footer-item">{&item.title}</span> }
                                                                            })}
                                                                        </>
                                                                    }
                                                                }
                                                            }
                                                        </div>
                                                        <p class="preview-footer-text">{"© 2024 My Rust CMS - Built with Rust & Yew"}</p>
                                                    </div>
                                                </div>
                                            },
                                            _ => html! {
                                                <div class="preview-placeholder">
                                                    <p>{format!("{} Component Preview", component_id)}</p>
                                                </div>
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                            
                            {if let Some(ref error_msg) = *save_error {
                                html! {
                                    <div class="error-message" style="margin: 1rem;">
                                        <strong>{"Error: "}</strong>{error_msg}
                                    </div>
                                }
                            } else {
                                html! {}
                            }}

                            <div class="editor-actions">
                                <button 
                                    class="btn-primary" 
                                    onclick={save_template.clone()}
                                    disabled={*saving}
                                >
                                    {if *saving { "Saving..." } else { "Save Changes" }}
                                </button>
                                <button class="btn-secondary">{"Reset to Default"}</button>
                                <button class="btn-secondary" onclick={close_editor.clone()}>{"Cancel"}</button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
            
            <div class="template-component-grid">
                {for props.component_templates.iter()
                    .filter(|template| template.component_type != "main_container")
                    .map(|template| {
                    let template_clone = template.clone();
                    let editing_component = editing_component.clone();
                    let editing_template = editing_template.clone();
                    let on_template_toggled = props.on_template_toggled.clone();
                    
                    html! {
                        <div class={format!("component-card {}", if template.is_default { "primary" } else { "secondary" })}>
                            <div class="component-preview">
                                {render_component_preview(&template.component_type, &header_navigation, &footer_navigation)}
                            </div>
                            <div class="component-info">
                                <h4>{get_component_title(&template.component_type)}</h4>
                                <p>{get_component_description(&template.component_type)}</p>
                                <div class="component-properties">
                                    <span class={format!("property-tag {}", if template.is_active { "active" } else { "inactive" })}>
                                        {if template.is_active { "Active" } else { "Inactive" }}
                                    </span>
                                    {render_component_tags(&template.component_type)}
                                </div>
                                <div class="component-actions">
                                    <button 
                                        class="btn-primary"
                                        onclick={{
                                            let template_clone = template_clone.clone();
                                            let component_type = template.component_type.clone();
                                            Callback::from(move |_| {
                                                editing_component.set(Some(component_type.clone()));
                                                editing_template.set(Some(template_clone.clone()));
                                            })
                                        }}
                                    >
                                        {format!("Customize {}", get_component_name(&template.component_type))}
                                    </button>
                                    {
                                        if template.component_type != "sidebar" {
                                            html! {
                                                <div class="toggle-switch">
                                                    <input 
                                                        type="checkbox"
                                                        id={format!("toggle-{}", template.id)}
                                                        checked={template.is_active}
                                                        onchange={{
                                                            let template_id = template.id;
                                                            let on_template_toggled = on_template_toggled.clone();
                                                            Callback::from(move |_| {
                                                                let on_template_toggled = on_template_toggled.clone();
                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                    match toggle_component_template(template_id).await {
                                                                        Ok(updated_template) => {
                                                                            on_template_toggled.emit(updated_template);
                                                                            log::info!("✅ Toggled component template {}", template_id);
                                                                        }
                                                                        Err(e) => {
                                                                            log::error!("❌ Failed to toggle component template: {:?}", e);
                                                                        }
                                                                    }
                                                                });
                                                            })
                                                        }}
                                                    />
                                                    <label for={format!("toggle-{}", template.id)} class="slider"></label>
                                                </div>
                                            }
                                        } else { html!{} }
                                    }
                                </div>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

#[function_component(ContainerSettingsView)]
pub fn container_settings_view() -> Html {
    let settings = use_state(ContainerSettings::default);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success_message = use_state(|| None::<String>);

    // Load container settings from backend
    {
        let settings = settings.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match get_settings(Some("container")).await {
                    Ok(backend_settings) => {
                        let mut container_settings = ContainerSettings::default();
                        
                        // Map backend settings to container settings
                        for setting in backend_settings {
                            match setting.setting_key.as_str() {
                                "container_mobile_breakpoint" => container_settings.mobile_breakpoint = setting.setting_value.unwrap_or_default(),
                                "container_tablet_breakpoint" => container_settings.tablet_breakpoint = setting.setting_value.unwrap_or_default(),
                                "container_desktop_breakpoint" => container_settings.desktop_breakpoint = setting.setting_value.unwrap_or_default(),
                                "container_wide_breakpoint" => container_settings.wide_breakpoint = setting.setting_value.unwrap_or_default(),
                                "container_base_font_size" => container_settings.base_font_size = setting.setting_value.unwrap_or_default(),
                                "container_scale_ratio" => container_settings.scale_ratio = setting.setting_value.unwrap_or_default(),
                                "container_line_height" => container_settings.line_height = setting.setting_value.unwrap_or_default(),
                                "container_width_type" => container_settings.width_type = setting.setting_value.unwrap_or_default(),
                                "container_max_width" => container_settings.max_width = setting.setting_value.unwrap_or_default(),
                                "container_horizontal_padding" => container_settings.horizontal_padding = setting.setting_value.unwrap_or_default(),
                                _ => {}
                            }
                        }
                        
                        settings.set(container_settings);
                    },
                    Err(e) => {
                        error.set(Some(format!("Failed to load settings: {}", e)));
                    }
                }
                loading.set(false);
            });
            || ()
        }, ());
    }

    // Save settings callback
    let save_settings = {
        let settings = settings.clone();
        let loading = loading.clone();
        let error = error.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            let settings = settings.clone();
            let loading = loading.clone();
            let error = error.clone();
            let success_message = success_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);
                success_message.set(None);

                let settings_data = vec![
                    SettingData {
                        key: "container_mobile_breakpoint".to_string(),
                        value: settings.mobile_breakpoint.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Mobile breakpoint for responsive design".to_string()),
                    },
                    SettingData {
                        key: "container_tablet_breakpoint".to_string(),
                        value: settings.tablet_breakpoint.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Tablet breakpoint for responsive design".to_string()),
                    },
                    SettingData {
                        key: "container_desktop_breakpoint".to_string(),
                        value: settings.desktop_breakpoint.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Desktop breakpoint for responsive design".to_string()),
                    },
                    SettingData {
                        key: "container_wide_breakpoint".to_string(),
                        value: settings.wide_breakpoint.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Wide screen breakpoint for responsive design".to_string()),
                    },
                    SettingData {
                        key: "container_base_font_size".to_string(),
                        value: settings.base_font_size.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Base font size for typography system".to_string()),
                    },
                    SettingData {
                        key: "container_scale_ratio".to_string(),
                        value: settings.scale_ratio.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Scale ratio for typography system".to_string()),
                    },
                    SettingData {
                        key: "container_line_height".to_string(),
                        value: settings.line_height.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Line height for typography system".to_string()),
                    },
                    SettingData {
                        key: "container_width_type".to_string(),
                        value: settings.width_type.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Container width type (fixed/fluid/hybrid)".to_string()),
                    },
                    SettingData {
                        key: "container_max_width".to_string(),
                        value: settings.max_width.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Maximum container width".to_string()),
                    },
                    SettingData {
                        key: "container_horizontal_padding".to_string(),
                        value: settings.horizontal_padding.clone(),
                        setting_type: "container".to_string(),
                        description: Some("Container horizontal padding".to_string()),
                    },
                ];

                match update_settings(settings_data).await {
                    Ok(_) => {
                        success_message.set(Some("Container settings saved successfully!".to_string()));
                    },
                    Err(e) => {
                        error.set(Some(format!("Failed to save settings: {}", e)));
                    }
                }
                loading.set(false);
            });
        })
    };

    // Reset to defaults callback
    let reset_settings = {
        let settings = settings.clone();
        Callback::from(move |_| {
            settings.set(ContainerSettings::default());
        })
    };

    // Input change handlers
    let on_mobile_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.mobile_breakpoint = input.value();
            settings.set(current_settings);
        })
    };

    let on_tablet_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.tablet_breakpoint = input.value();
            settings.set(current_settings);
        })
    };

    let on_desktop_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.desktop_breakpoint = input.value();
            settings.set(current_settings);
        })
    };

    let on_wide_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.wide_breakpoint = input.value();
            settings.set(current_settings);
        })
    };

    let on_font_size_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.base_font_size = select.value();
            settings.set(current_settings);
        })
    };

    let on_scale_ratio_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.scale_ratio = select.value();
            settings.set(current_settings);
        })
    };

    let on_line_height_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.line_height = input.value();
            settings.set(current_settings);
        })
    };

    let on_width_type_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.width_type = select.value();
            settings.set(current_settings);
        })
    };

    let on_max_width_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.max_width = input.value();
            settings.set(current_settings);
        })
    };

    let on_padding_change = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut current_settings = (*settings).clone();
            current_settings.horizontal_padding = input.value();
            settings.set(current_settings);
        })
    };

    html! {
        <div class="container-settings-section">
            <h2>{"📦 Container Settings"}</h2>
            <p>{"Manage global breakpoints, typography, and container settings that tie into the design system"}</p>
            
            if let Some(error_msg) = (*error).as_ref() {
                <div class="error-message">
                    <span class="error-icon">{"⚠️"}</span>
                    <span>{error_msg}</span>
                </div>
            }
            
            if let Some(success_msg) = (*success_message).as_ref() {
                <div class="success-message">
                    <span class="success-icon">{"✅"}</span>
                    <span>{success_msg}</span>
                </div>
            }
            
            if *loading {
                <div class="loading-overlay">{"Loading..."}</div>
            }
            
            <div class="settings-grid">
                <div class="settings-card">
                    <h3>{"📐 Global Breakpoints"}</h3>
                    <p>{"Define responsive breakpoints for the entire site"}</p>
                    
                    <div class="breakpoint-list">
                        <div class="breakpoint-item">
                            <label>{"Mobile (max-width)"}</label>
                            <div class="breakpoint-input-group">
                                <input 
                                    type="text" 
                                    value={settings.mobile_breakpoint.clone()} 
                                    class="breakpoint-input"
                                    onchange={on_mobile_change}
                                />
                                <span class="breakpoint-preview">{format!("< {}", settings.mobile_breakpoint)}</span>
                            </div>
                        </div>
                        <div class="breakpoint-item">
                            <label>{"Tablet (max-width)"}</label>
                            <div class="breakpoint-input-group">
                                <input 
                                    type="text" 
                                    value={settings.tablet_breakpoint.clone()} 
                                    class="breakpoint-input"
                                    onchange={on_tablet_change}
                                />
                                <span class="breakpoint-preview">{format!("{} - {}", settings.mobile_breakpoint, settings.tablet_breakpoint)}</span>
                            </div>
                        </div>
                        <div class="breakpoint-item">
                            <label>{"Desktop (max-width)"}</label>
                            <div class="breakpoint-input-group">
                                <input 
                                    type="text" 
                                    value={settings.desktop_breakpoint.clone()} 
                                    class="breakpoint-input"
                                    onchange={on_desktop_change}
                                />
                                <span class="breakpoint-preview">{format!("{} - {}", settings.tablet_breakpoint, settings.desktop_breakpoint)}</span>
                            </div>
                        </div>
                        <div class="breakpoint-item">
                            <label>{"Wide Screen (max-width)"}</label>
                            <div class="breakpoint-input-group">
                                <input 
                                    type="text" 
                                    value={settings.wide_breakpoint.clone()} 
                                    class="breakpoint-input"
                                    onchange={on_wide_change}
                                />
                                <span class="breakpoint-preview">{format!("> {}", settings.desktop_breakpoint)}</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="settings-card">
                    <h3>{"🎯 Typography System"}</h3>
                    <p>{"Configure global typography that integrates with the design system"}</p>
                    
                    <div class="typography-settings">
                        <div class="typography-group">
                            <h4>{"Base Settings"}</h4>
                            <div class="typography-item">
                                <label>{"Base Font Size"}</label>
                                <select class="typography-select" onchange={on_font_size_change}>
                                    <option value="14px" selected={settings.base_font_size == "14px"}>{"14px (Small)"}</option>
                                    <option value="16px" selected={settings.base_font_size == "16px"}>{"16px (Standard)"}</option>
                                    <option value="18px" selected={settings.base_font_size == "18px"}>{"18px (Large)"}</option>
                                </select>
                            </div>
                            <div class="typography-item">
                                <label>{"Scale Ratio"}</label>
                                <select class="typography-select" onchange={on_scale_ratio_change}>
                                    <option value="1.125" selected={settings.scale_ratio == "1.125"}>{"1.125 (Minor Second)"}</option>
                                    <option value="1.25" selected={settings.scale_ratio == "1.25"}>{"1.25 (Major Third)"}</option>
                                    <option value="1.5" selected={settings.scale_ratio == "1.5"}>{"1.5 (Perfect Fifth)"}</option>
                                    <option value="1.618" selected={settings.scale_ratio == "1.618"}>{"1.618 (Golden Ratio)"}</option>
                                </select>
                            </div>
                            <div class="typography-item">
                                <label>{"Line Height"}</label>
                                <input 
                                    type="number" 
                                    value={settings.line_height.clone()} 
                                    step="0.1" 
                                    min="1" 
                                    max="2" 
                                    class="typography-input"
                                    onchange={on_line_height_change}
                                />
                            </div>
                        </div>
                        
                        <div class="typography-preview">
                            <h4>{"Typography Preview"}</h4>
                            <div class="preview-text" style={format!("font-size: {}; line-height: {}", settings.base_font_size, settings.line_height)}>
                                <h1 class="preview-h1">{"Heading 1 - Main Title"}</h1>
                                <h2 class="preview-h2">{"Heading 2 - Section Title"}</h2>
                                <p class="preview-paragraph">
                                    {"This is a paragraph demonstrating the body text with current typography settings."}
                                </p>
                                <code class="preview-code">{"console.log('Code example');"}</code>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="settings-card">
                    <h3>{"📦 Container Configuration"}</h3>
                    <p>{"Configure the main container that wraps site content"}</p>
                    
                    <div class="container-settings">
                        <div class="container-group">
                            <h4>{"Container Width"}</h4>
                            <div class="container-item">
                                <label>{"Width Type"}</label>
                                <select class="container-select" onchange={on_width_type_change}>
                                    <option value="fixed" selected={settings.width_type == "fixed"}>{"Fixed Width"}</option>
                                    <option value="fluid" selected={settings.width_type == "fluid"}>{"Fluid Width"}</option>
                                    <option value="hybrid" selected={settings.width_type == "hybrid"}>{"Hybrid"}</option>
                                </select>
                            </div>
                            <div class="container-item">
                                <label>{"Max Width"}</label>
                                <input 
                                    type="text" 
                                    value={settings.max_width.clone()} 
                                    class="container-input"
                                    onchange={on_max_width_change}
                                />
                            </div>
                            <div class="container-item">
                                <label>{"Horizontal Padding"}</label>
                                <input 
                                    type="text" 
                                    value={settings.horizontal_padding.clone()} 
                                    class="container-input"
                                    onchange={on_padding_change}
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="settings-actions">
                <button 
                    class="btn-primary large" 
                    onclick={save_settings}
                    disabled={*loading}
                >
                    {if *loading { "Saving..." } else { "Save Global Settings" }}
                </button>
                <button 
                    class="btn-secondary" 
                    onclick={reset_settings}
                    disabled={*loading}
                >
                    {"Reset to Defaults"}
                </button>
                <button class="btn-secondary" disabled={*loading}>{"Export Settings"}</button>
            </div>
        </div>
    }
}