use yew::prelude::*;
use crate::services::navigation_service::{get_navigation_by_area, get_component_templates, ComponentTemplate};
use crate::services::api_service::get_settings;
use crate::pages::public::PublicPage;
use crate::pages::admin::design_system::{PublicColorScheme, apply_public_css_variables};

#[derive(Properties, PartialEq)]
pub struct PublicLayoutProps {
    pub children: Children,
    pub on_admin_click: Callback<()>,

    pub on_navigate: Option<Callback<PublicPage>>,
    pub current_page: String,
}

#[function_component(PublicLayout)]
pub fn public_layout(props: &PublicLayoutProps) -> Html {
    let header_navigation_items = use_state(Vec::new);
    let footer_navigation_items = use_state(Vec::new);
    let component_templates = use_state(Vec::<ComponentTemplate>::new);
    let loading = use_state(|| true);
    let admin_button_visible = use_state(|| true); // Default to true until loaded
    let site_title = use_state(|| "My Rust CMS".to_string());
    let acid_mode = use_state(|| false);

    // Load navigation items, component templates, and admin button setting
    {
        let header_navigation_items = header_navigation_items.clone();
        let footer_navigation_items = footer_navigation_items.clone();
        let component_templates = component_templates.clone();
        let loading = loading.clone();
        let admin_button_visible = admin_button_visible.clone();
        let site_title = site_title.clone();
        let acid_mode = acid_mode.clone();

        use_effect_with_deps(move |_| {
            web_sys::console::log_1(&"PublicLayout: Starting to fetch navigation items, templates, and settings".into());
            wasm_bindgen_futures::spawn_local(async move {
                // Load header and footer navigation items
                let header_nav_result = get_navigation_by_area("header").await;
                let footer_nav_result = get_navigation_by_area("footer").await;
                
                // Load component templates
                let templates_result = get_component_templates().await;
                
                // Load site and container settings
                let settings_result = get_settings(Some("site")).await;
                let container_settings_result = get_settings(Some("container")).await;
                
                match header_nav_result {
                    Ok(items) => {
                        web_sys::console::log_1(&format!("Header navigation items loaded: {:?}", items).into());
                        header_navigation_items.set(items);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Header navigation error: {:?}", e).into());
                    }
                }
                
                match footer_nav_result {
                    Ok(items) => {
                        web_sys::console::log_1(&format!("Footer navigation items loaded: {:?}", items).into());
                        footer_navigation_items.set(items);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Footer navigation error: {:?}", e).into());
                    }
                }
                
                match templates_result {
                    Ok(templates) => {
                        web_sys::console::log_1(&format!("Component templates loaded: {:?}", templates).into());
                        component_templates.set(templates);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Component templates error: {:?}", e).into());
                    }
                }
                
                match settings_result {
                    Ok(settings) => {
                        web_sys::console::log_1(&format!("Settings loaded: {:?}", settings).into());
                        // Find admin button setting
                        if let Some(setting) = settings.iter().find(|s| s.setting_key == "admin_button_visible") {
                            if let Some(value) = &setting.setting_value {
                                let visible = value.parse::<bool>().unwrap_or(true);
                                admin_button_visible.set(visible);
                                web_sys::console::log_1(&format!("Admin button visibility set to: {}", visible).into());
                            }
                        }
                        
                        // Find site title setting
                        if let Some(setting) = settings.iter().find(|s| s.setting_key == "site_title") {
                            if let Some(ref value) = setting.setting_value {
                                if !value.trim().is_empty() {
                                    site_title.set(value.trim().to_string());
                                    web_sys::console::log_1(&format!("Site title set to: {}", value.trim()).into());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Settings error: {:?}", e).into());
                        // Keep default value of true if settings fail to load
                    }
                }

                // Parse container settings for acid mode
                match container_settings_result {
                    Ok(settings) => {
                        if let Some(setting) = settings.iter().find(|s| s.setting_key == "container_acid_mode") {
                            if let Some(ref value) = setting.setting_value {
                                let enabled = value.trim().eq_ignore_ascii_case("true");
                                acid_mode.set(enabled);
                                web_sys::console::log_1(&format!("Acid mode set to: {}", enabled).into());
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Container settings error: {:?}", e).into());
                    }
                }
                
                loading.set(false);
            });
            || ()
        }, ());
    }

    // Apply default public theme on component mount
    {
        use_effect_with_deps(move |_| {
            web_sys::console::log_1(&"PublicLayout: Applying default public theme".into());
            let default_scheme = PublicColorScheme::default();
            apply_public_css_variables(&default_scheme);
            || ()
        }, ());
    }

    let on_admin_click = {
        let callback = props.on_admin_click.clone();
        Callback::from(move |_| callback.emit(()))
    };

    // Helper function to check if component is active
    let is_component_active = {
        let component_templates = component_templates.clone();
        move |component_type: &str| -> bool {
            component_templates.iter()
                .any(|t| t.component_type == component_type && t.is_active)
        }
    };

    // Helper function to get template styles (safe subset for public UI)
    let get_component_style = {
        let component_templates = component_templates.clone();
        move |component_type: &str| -> String {
            if let Some(template) = component_templates.iter()
                .find(|t| t.component_type == component_type && t.is_active) {
                let mut styles = Vec::new();
                
                if let Some(height) = template.template_data.get("height").and_then(|v| v.as_str()) {
                    if component_type == "header" {
                        let mut h = height.to_string();
                        if let Some(stripped) = h.strip_suffix("px") {
                            if let Ok(px) = stripped.trim().parse::<i32>() {
                                if px < 110 { h = "110px".to_string(); }
                            }
                        }
                        styles.push(format!("height: {}", h));
                    } else {
                        styles.push(format!("height: {}", height));
                    }
                } else if component_type == "header" {
                    styles.push("height: 110px".to_string());
                }
                
                // Only allow position overrides for non-header components to avoid layout breaks
                if component_type != "header" {
                    if let Some(position) = template.template_data.get("position").and_then(|v| v.as_str()) {
                        styles.push(format!("position: {}", position));
                    }
                }
                
                // Support both background (gradients/images) and background_color
                if component_type == "header" {
                    // For header, set background directly; coerce white to black per default theme requirement
                    if let Some(mut bg) = template.template_data.get("background").and_then(|v| v.as_str()) {
                        if bg.trim().eq_ignore_ascii_case("#ffffff") { bg = "#000000"; }
                        styles.push(format!("background: {}", bg));
                    } else if let Some(mut bg) = template.template_data.get("background_color").and_then(|v| v.as_str()) {
                        if bg.trim().eq_ignore_ascii_case("#ffffff") { bg = "#000000"; }
                        styles.push(format!("background-color: {}", bg));
                    } else {
                        // Fallback when no background provided
                        styles.push("background-color: #000000".to_string());
                    }
                } else if component_type == "footer" {
                    if let Some(bg) = template.template_data.get("background").and_then(|v| v.as_str()) {
                        styles.push(format!("--public-footer-bg: {}", bg));
                    } else if let Some(bg) = template.template_data.get("background_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--public-footer-bg: {}", bg));
                    }
                } else {
                    if let Some(background) = template.template_data.get("background").and_then(|v| v.as_str()) {
                        styles.push(format!("background: {}", background));
                    } else if let Some(background) = template.template_data.get("background_color").and_then(|v| v.as_str()) {
                        styles.push(format!("background-color: {}", background));
                    }
                }
                
                // Optional text color overrides via CSS variables for header/footer
                if component_type == "header" {
                    if let Some(text_color) = template.template_data.get("text_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--header-text: {}", text_color));
                    } else {
                        // Default to white for readability on black header
                        styles.push("--header-text: #ffffff".to_string());
                    }
                    if let Some(text_hover) = template.template_data.get("text_hover_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--header-text-hover: {}", text_hover));
                    } else {
                        styles.push("--header-text-hover: #f7fafc".to_string());
                    }
                    if let Some(nav_hover) = template.template_data.get("nav_hover_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--nav-hover-color: {}", nav_hover));
                    }
                    if let Some(nav_underline) = template.template_data.get("nav_underline_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--nav-underline-color: {}", nav_underline));
                    }
                    if let Some(thickness) = template.template_data.get("nav_underline_thickness").and_then(|v| v.as_str()) {
                        styles.push(format!("--nav-underline-thickness: {}", thickness));
                    }
                    if let Some(anim) = template.template_data.get("nav_underline_animation").and_then(|v| v.as_str()) {
                        styles.push(format!("--nav-underline-animation: {}", anim));
                    }
                }
                if component_type == "footer" {
                    if let Some(text_color) = template.template_data.get("text_color").and_then(|v| v.as_str()) {
                        styles.push(format!("--footer-text: {}", text_color));
                    }
                    if let Some(text_muted) = template.template_data.get("text_muted").and_then(|v| v.as_str()) {
                        styles.push(format!("--footer-text-muted: {}", text_muted));
                    }
                    if let Some(bg) = template.template_data.get("background").and_then(|v| v.as_str()) {
                        styles.push(format!("--footer-background: {}", bg));
                    }
                }

                if let Some(z_index_val) = template.template_data.get("z_index") {
                    if let Some(z) = z_index_val.as_i64() {
                        styles.push(format!("z-index: {}", z));
                    } else if let Some(z) = z_index_val.as_str() {
                        styles.push(format!("z-index: {}", z));
                    }
                }
                
                if let Some(padding) = template.template_data.get("padding").and_then(|v| v.as_str()) {
                    styles.push(format!("padding: {}", padding));
                }
                
                if let Some(margin) = template.template_data.get("margin").and_then(|v| v.as_str()) {
                    styles.push(format!("margin: {}", margin));
                }
                
                if let Some(border) = template.template_data.get("border").and_then(|v| v.as_str()) {
                    styles.push(format!("border: {}", border));
                }
                
                if let Some(box_shadow) = template.template_data.get("box_shadow").and_then(|v| v.as_str()) {
                    styles.push(format!("box-shadow: {}", box_shadow));
                }
                
                let style_string = styles.join("; ");
                if !style_string.is_empty() {
                    web_sys::console::log_1(&format!("Applying {} template styles: {}", component_type, style_string).into());
                }
                style_string
            } else {
                String::new()
            }
        }
    };

    // Global style variables derived from specific component templates (e.g., posts_list)
    let global_style_vars = {
        let component_templates = component_templates.clone();
        move || -> String {
            let mut vars: Vec<String> = Vec::new();
            if let Some(posts_tpl) = component_templates.iter().find(|t| t.component_type == "posts_list" && t.is_active) {
                if let Some(bg) = posts_tpl.template_data.get("card_background").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-card-bg: {}", bg));
                }
                if let Some(radius) = posts_tpl.template_data.get("card_radius").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-card-radius: {}", radius));
                }
                if let Some(shadow) = posts_tpl.template_data.get("card_shadow").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-card-shadow: {}", shadow));
                }
                if let Some(title_color) = posts_tpl.template_data.get("title_color").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-title-color: {}", title_color));
                }
                if let Some(meta_color) = posts_tpl.template_data.get("meta_color").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-meta-color: {}", meta_color));
                }
                if let Some(link_color) = posts_tpl.template_data.get("link_color").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-link-color: {}", link_color));
                }
                if let Some(grid_gap) = posts_tpl.template_data.get("grid_gap").and_then(|v| v.as_str()) {
                    vars.push(format!("--posts-grid-gap: {}", grid_gap));
                }
            }
            // Hero variables (background/text)
            if let Some(hero_tpl) = component_templates.iter().find(|t| t.component_type == "hero" && t.is_active) {
                if let Some(bg) = hero_tpl.template_data.get("background").and_then(|v| v.as_str()) {
                    vars.push(format!("--hero-bg: {}", bg));
                }
                if let Some(color) = hero_tpl.template_data.get("text_color").and_then(|v| v.as_str()) {
                    vars.push(format!("--hero-text: {}", color));
                }
            }
            // Buttons
            if let Some(btn_tpl) = component_templates.iter().find(|t| t.component_type == "header" && t.is_active) {
                if let Some(bg) = btn_tpl.template_data.get("button_primary_bg").and_then(|v| v.as_str()) {
                    vars.push(format!("--button-primary-bg: {}", bg));
                }
                if let Some(text) = btn_tpl.template_data.get("button_primary_text").and_then(|v| v.as_str()) {
                    vars.push(format!("--button-primary-text: {}", text));
                }
                if let Some(hover_bg) = btn_tpl.template_data.get("button_primary_hover_bg").and_then(|v| v.as_str()) {
                    vars.push(format!("--button-primary-hover-bg: {}", hover_bg));
                }
            }
            // Badges
            if let Some(badge_tpl) = component_templates.iter().find(|t| t.component_type == "header" && t.is_active) {
                if let Some(bg) = badge_tpl.template_data.get("badge_bg").and_then(|v| v.as_str()) {
                    vars.push(format!("--badge-bg: {}", bg));
                }
                if let Some(text) = badge_tpl.template_data.get("badge_text").and_then(|v| v.as_str()) {
                    vars.push(format!("--badge-text: {}", text));
                }
            }
            // Background animation
            if let Some(site_tpl) = component_templates.iter().find(|t| t.component_type == "main_container" && t.is_active) {
                if let Some(anim) = site_tpl.template_data.get("background_animation").and_then(|v| v.as_str()) {
                    vars.push(format!("--bg-animation: {}", anim));
                }
            }
            vars.join("; ")
        }
    };

    let on_nav_item_click = {
        let on_navigate = props.on_navigate.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(on_navigate) = &on_navigate {
                if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                    if let Some(url) = target.get_attribute("data-url") {
                        let page = match url.as_str() {
                            "/" => PublicPage::Home,
                            "/posts" => PublicPage::Posts,
                            url if url.starts_with("/post/") => {
                                if let Ok(id) = url.trim_start_matches("/post/").parse::<i32>() {
                                    PublicPage::Post(id)
                                } else {
                                    return;
                                }
                            }
                            url if url.starts_with("/page/") => {
                                let slug = url.trim_start_matches("/page/");
                                PublicPage::Page(slug.to_string())
                            }
                            _ => return,
                        };
                        on_navigate.emit(page);
                    }
                }
            }
        })
    };

    html! {
        <div class={if *acid_mode { "public-site acid-mode" } else { "public-site" }} style={global_style_vars()}>
            {if is_component_active("header") {
                html! {
                    <header class="site-header" style={get_component_style("header")}>
                        <div class="container">
                            <h1 class="site-title">{(*site_title).clone()}</h1>
                            <nav class="site-nav">
                                if !*loading {
                                    {{
                                        let items: Vec<_> = header_navigation_items.iter().filter(|item| item.is_active).collect();
                                        web_sys::console::log_1(&format!("Filtered header navigation items: {:?}", items).into());
                                        web_sys::console::log_1(&format!("Current page: {}", props.current_page).into());
                                        items.into_iter().map(|item| {
                                            let is_active = props.current_page == item.url.trim_start_matches('/');
                                            html! {
                                                <a 
                                                    href="#" 
                                                    class={if is_active { "nav-link active" } else { "nav-link" }}
                                                    data-url={item.url.clone()}
                                                    onclick={on_nav_item_click.clone()}
                                                >
                                                    {&item.title}
                                                </a>
                                            }
                                        }).collect::<Html>()
                                    }}
                                }
                                
                                {if *admin_button_visible {
                                    html! {
                                        <button class="nav-button admin-button" onclick={on_admin_click}>
                                            {"Admin"}
                                        </button>
                                    }
                                } else {
                                    html! {}
                                }}
                            </nav>
                        </div>
                    </header>
                }
            } else {
                html! {}
            }}

            <main class="site-main">
                <div class="container">
                    {props.children.clone()}
                </div>
            </main>

            {if is_component_active("footer") {
                html! {
                    <footer class="site-footer" style={get_component_style("footer")}>
                        <div class="container">
                            {if !footer_navigation_items.is_empty() {
                                html! {
                                    <nav class="footer-nav">
                                        {footer_navigation_items.iter().filter(|item| item.is_active).map(|item| {
                                            html! {
                                                <a 
                                                    href="#" 
                                                    class="footer-nav-link"
                                                    data-url={item.url.clone()}
                                                    onclick={on_nav_item_click.clone()}
                                                >
                                                    {&item.title}
                                                </a>
                                            }
                                        }).collect::<Html>()}
                                    </nav>
                                }
                            } else {
                                html! {}
                            }}
                            <p class="footer-copyright">{"© 2024 My Rust CMS. Built with Rust and Yew."}</p>
                        </div>
                    </footer>
                }
            } else {
                html! {}
            }}
        </div>
    }
} 