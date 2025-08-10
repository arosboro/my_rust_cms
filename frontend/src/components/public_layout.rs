use yew::prelude::*;
use crate::services::navigation_service::{get_navigation_by_area, get_component_templates, ComponentTemplate};
use crate::services::api_service::get_public_settings;
use std::collections::HashMap;
use crate::pages::public::PublicPage;
use crate::pages::admin::design_system::{PublicColorScheme, apply_public_css_variables};
use wasm_bindgen::JsCast;
use crate::services::auth_context::use_auth;
use crate::components::LiveEditMode;

#[derive(Properties, PartialEq)]
pub struct PublicLayoutProps {
    pub children: Children,
    pub on_admin_click: Callback<()>,

    pub on_navigate: Option<Callback<PublicPage>>,
    pub current_page: String,
}

#[function_component(PublicLayout)]
pub fn public_layout(props: &PublicLayoutProps) -> Html {
    let auth = use_auth();
    let header_navigation_items = use_state(Vec::new);
    let footer_navigation_items = use_state(Vec::new);
    let component_templates = use_state(Vec::<ComponentTemplate>::new);
    let loading = use_state(|| true);
    let admin_button_visible = use_state(|| true); // Default to true until loaded
    let site_title = use_state(|| "My Rust CMS".to_string());
    let acid_mode = use_state(|| false);
    let site_style = use_state(|| String::new());
    let inner_container_style = use_state(|| String::new());
    let live_edit_enabled = use_state(|| false);

    // Load navigation items, component templates, and admin button setting
    {
        let header_navigation_items = header_navigation_items.clone();
        let footer_navigation_items = footer_navigation_items.clone();
        let component_templates = component_templates.clone();
        let loading = loading.clone();
        let admin_button_visible = admin_button_visible.clone();
        let site_title = site_title.clone();
        let acid_mode = acid_mode.clone();
        let site_style = site_style.clone();
        let inner_container_style = inner_container_style.clone();

        use_effect_with_deps(move |_| {
            web_sys::console::log_1(&"PublicLayout: Starting to fetch navigation items, templates, and settings".into());
            wasm_bindgen_futures::spawn_local(async move {
                // Load header and footer navigation items
                let header_nav_result = get_navigation_by_area("header").await;
                let footer_nav_result = get_navigation_by_area("footer").await;
                
                // Load component templates
                let templates_result = get_component_templates().await;
                
                // Load site and container settings
                let settings_result = get_public_settings(Some("site")).await;
                let container_settings_result = get_public_settings(Some("container")).await;
                
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
                        let mut map: HashMap<String, String> = HashMap::new();
                        for s in &settings {
                            if let Some(val) = s.setting_value.clone() {
                                map.insert(s.setting_key.clone(), val);
                            }
                        }

                        if let Some(setting) = settings.iter().find(|s| s.setting_key == "container_acid_mode") {
                            if let Some(ref value) = setting.setting_value {
                                let enabled = value.trim().eq_ignore_ascii_case("true");
                                acid_mode.set(enabled);
                                web_sys::console::log_1(&format!("Acid mode set to: {}", enabled).into());
                            }
                        }

                        // Apply background and layout from container settings
                        let width_type = map.get("container_width_type").cloned().unwrap_or_default();
                        let max_width = map.get("container_max_width").cloned().unwrap_or_default();
                        let horizontal_padding = map.get("container_horizontal_padding").cloned().unwrap_or_default();

                        let mut container_css: Vec<String> = Vec::new();
                        if width_type == "fixed" && !max_width.is_empty() { container_css.push(format!("max-width: {}", max_width)); }
                        if !horizontal_padding.is_empty() { container_css.push(format!("padding-left: {}; padding-right: {}", horizontal_padding, horizontal_padding)); }
                        inner_container_style.set(container_css.join("; "));

                        // Backgrounds
                        let background_type = map.get("container_background_type").map(|s| s.as_str()).unwrap_or("none");
                        let overlay_color_raw = map.get("container_overlay_color").cloned().unwrap_or_default();
                        let overlay_opacity_raw = map.get("container_overlay_opacity").cloned().unwrap_or_else(|| "0".to_string());

                        // Helper: parse opacity which may be a float (0..1) or percentage (e.g., "30%")
                        fn parse_opacity(value: &str) -> f32 {
                            let v = value.trim();
                            if let Some(stripped) = v.strip_suffix('%') {
                                if let Ok(p) = stripped.trim().parse::<f32>() { return (p / 100.0).clamp(0.0, 1.0); }
                                return 0.0;
                            }
                            v.parse::<f32>().ok().map(|f| f.clamp(0.0, 1.0)).unwrap_or(0.0)
                        }

                        // Helper: build overlay gradient layer from color and opacity
                        fn build_overlay_layer(color: &str, opacity: f32) -> Option<String> {
                            if opacity <= 0.0 { return None; }
                            let c = color.trim();
                            if c.is_empty() { return None; }
                            // Supported: #RRGGBB, rgb(r,g,b), rgba(r,g,b,a)
                            if c.starts_with('#') && c.len() == 7 {
                                let r = u8::from_str_radix(&c[1..3], 16).unwrap_or(0);
                                let g = u8::from_str_radix(&c[3..5], 16).unwrap_or(0);
                                let b = u8::from_str_radix(&c[5..7], 16).unwrap_or(0);
                                Some(format!(
                                    "linear-gradient(rgba({}, {}, {}, {}), rgba({}, {}, {}, {}))",
                                    r, g, b, opacity, r, g, b, opacity
                                ))
                            } else if c.starts_with("rgb(") || c.starts_with("rgba(") {
                                // Normalize into rgba with provided opacity by replacing any existing alpha
                                // Simple approach: extract numbers
                                let inside = c.trim_start_matches("rgba(").trim_start_matches("rgb(").trim_end_matches(")");
                                let parts: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
                                if parts.len() >= 3 {
                                    let r = parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                                    let g = parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                                    let b = parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                                    Some(format!(
                                        "linear-gradient(rgba({}, {}, {}, {}), rgba({}, {}, {}, {}))",
                                        r, g, b, opacity, r, g, b, opacity
                                    ))
                                } else { None }
                            } else {
                                None
                            }
                        }

                        let overlay_alpha = parse_opacity(&overlay_opacity_raw);
                        let overlay_layer: Option<String> = build_overlay_layer(&overlay_color_raw, overlay_alpha);

                        let mut bg_layers: Vec<String> = Vec::new();
                        if let Some(layer) = overlay_layer { bg_layers.push(layer); }

                        match background_type {
                            "color" => {
                                if let Some(color) = map.get("container_background_color") { bg_layers.push(color.clone()); }
                            }
                            "gradient" => {
                                let from = map.get("container_gradient_from").cloned().unwrap_or("#000000".to_string());
                                let to = map.get("container_gradient_to").cloned().unwrap_or("#222222".to_string());
                                let angle = map.get("container_gradient_angle").cloned().unwrap_or("180deg".to_string());
                                bg_layers.push(format!("linear-gradient({} , {} , {} )", angle, from, to));
                            }
                            "image" => {
                                let url = map.get("container_background_image_url").cloned().unwrap_or_default();
                                if !url.is_empty() { bg_layers.push(format!("url('{}')", url)); }
                            }
                            "video" => {
                                // video will be rendered as element below; optional poster image layer could be added later
                            }
                            _ => {}
                        }

                        let mut bg_css: Vec<String> = Vec::new();
                        if !bg_layers.is_empty() {
                            bg_css.push(format!("background: {}", bg_layers.join(", ")));
                        }
                        if let Some(size) = map.get("container_background_image_size") { if !size.is_empty() { bg_css.push(format!("background-size: {}", size)); } }
                        if let Some(pos) = map.get("container_background_image_position") { if !pos.is_empty() { bg_css.push(format!("background-position: {}", pos)); } }
                        if matches!(background_type, "image" | "video") { bg_css.push("background-repeat: no-repeat".to_string()); bg_css.push("background-attachment: scroll".to_string()); }

                        let css = bg_css.join("; ");
                        if !css.is_empty() { site_style.set(css); }

                        // Sync container settings to <body> data-* attributes for live editing/preview
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(body) = document.body() {
                                    let _ = body.set_attribute("data-bg-type", background_type);
                                    let _ = body.set_attribute("data-acid-enabled", if *acid_mode { "true" } else { "false" });
                                    if let Some(url) = map.get("container_background_video_url") {
                                        let _ = body.set_attribute("data-bg-video-url", url);
                                    }
                                    if let Some(looping) = map.get("container_background_video_loop") {
                                        let _ = body.set_attribute("data-bg-video-loop", looping);
                                    }
                                    if let Some(autoplay) = map.get("container_background_video_autoplay") {
                                        let _ = body.set_attribute("data-bg-video-autoplay", autoplay);
                                    }
                                    if let Some(muted) = map.get("container_background_video_muted") {
                                        let _ = body.set_attribute("data-bg-video-muted", muted);
                                    }
                                    let _ = body.set_attribute("data-overlay-color", &overlay_color_raw);
                                    let _ = body.set_attribute("data-overlay-opacity", &overlay_opacity_raw);
                                }
                            }
                        }

                        // Create/update a dedicated background video layer attached to <body>
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                // Remove any existing layer by id
                                if let Some(existing) = document.get_element_by_id("bg-video-layer") {
                                    if background_type != "video" { let _ = existing.remove(); }
                                }
                                if background_type == "video" {
                                    if let Some(url) = map.get("container_background_video_url").cloned() {
                                        if !url.is_empty() {
                                            let looping = map.get("container_background_video_loop").cloned().unwrap_or_else(|| "true".to_string());
                                            let autoplay = map.get("container_background_video_autoplay").cloned().unwrap_or_else(|| "true".to_string());
                                            let muted = map.get("container_background_video_muted").cloned().unwrap_or_else(|| "true".to_string());

                                            let container = document.get_element_by_id("bg-video-layer").unwrap_or_else(|| {
                                                let div = document.create_element("div").unwrap();
                                                div.set_attribute("id", "bg-video-layer").ok();
                                                div.set_attribute("style", "position: fixed; inset: 0; width: 100%; height: 100%; z-index: -2; pointer-events: none; overflow: hidden;").ok();
                                                document.body().unwrap().append_child(&div).ok();
                                                div
                                            });

                                            // YouTube detection
                                            let is_youtube = url.contains("youtube.com") || url.contains("youtu.be");
                                            if is_youtube {
                                                // Extract id
                                                let id = (|| {
                                                    if let Some(idx) = url.find("youtu.be/") { return url[idx+9..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                                    if let Some(idx) = url.find("watch?v=") { return url[idx+8..].split(['&', '#']).next().map(|s| s.to_string()); }
                                                    if let Some(idx) = url.find("/shorts/") { return url[idx+8..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                                    if let Some(idx) = url.find("/embed/") { return url[idx+7..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                                    None
                                                })();
                                                if let Some(id) = id {
                                                    let embed_src = format!(
                                                        "https://www.youtube.com/embed/{}?autoplay={}&mute={}&loop={}&playlist={}&controls=0&showinfo=0&modestbranding=1&iv_load_policy=3&rel=0&playsinline=1",
                                                        id,
                                                        if autoplay == "true" { 1 } else { 0 },
                                                        if muted == "true" { 1 } else { 0 },
                                                        if looping == "true" { 1 } else { 0 },
                                                        id
                                                    );
                                                    container.set_inner_html(&format!(
                                                        "<iframe src=\"{}\" style=\"position:absolute; inset:0; width:100%; height:100%; border:0; pointer-events:none;\" allow=\"autoplay; encrypted-media; picture-in-picture\"></iframe>",
                                                        embed_src
                                                    ));
                                                }
                                            } else {
                                                container.set_inner_html(&format!(
                                                    "<video src=\"{}\" {} {} {} playsinline style=\"position:absolute; inset:0; width:100%; height:100%; object-fit:cover;\"></video>",
                                                    url,
                                                    if autoplay == "true" { "autoplay" } else { "" },
                                                    if looping == "true" { "loop" } else { "" },
                                                    if muted == "true" { "muted" } else { "" },
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("Container settings error: {:?}", e).into());
                    }
                }
                
                loading.set(false);
            });
            // Cleanup when PublicLayout unmounts: remove background media & data attrs
            || {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(el) = document.get_element_by_id("bg-video-layer") {
                            // Attempt to pause any <video> before removal
                            if let Some(video_el) = el.query_selector("video").ok().flatten() {
                                let _ = js_sys::Reflect::get(&video_el, &wasm_bindgen::JsValue::from_str("pause"))
                                    .ok()
                                    .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                                    .and_then(|f| f.call0(&video_el).ok());
                            }
                            let _ = el.remove();
                        }
                        if let Some(body) = document.body() {
                            let _ = body.remove_attribute("data-bg-type");
                            let _ = body.remove_attribute("data-bg-video-url");
                            let _ = body.remove_attribute("data-bg-video-loop");
                            let _ = body.remove_attribute("data-bg-video-autoplay");
                            let _ = body.remove_attribute("data-bg-video-muted");
                            let _ = body.remove_attribute("data-overlay-color");
                            let _ = body.remove_attribute("data-overlay-opacity");
                        }
                    }
                }
            }
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

    // Mirror acid-mode class to <body> for maximum selector compatibility
    {
        let acid_mode = acid_mode.clone();
        use_effect_with_deps(move |enabled| {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        let existing = body.get_attribute("class").unwrap_or_default();
                        let has = existing.split_whitespace().any(|c| c == "acid-mode");
                        let new_class = if **enabled {
                            if has { existing } else { format!("{}{}acid-mode", existing, if existing.is_empty() { "" } else { " " }) }
                        } else {
                            existing
                                .split_whitespace()
                                .filter(|c| *c != "acid-mode")
                                .collect::<Vec<_>>()
                                .join(" ")
                        };
                        let _ = body.set_attribute("class", &new_class);
                    }
                }
            }
            || ()
        }, acid_mode);
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
        <div class={if *acid_mode { "public-site acid-mode" } else { "public-site" }} style={format!("{}{}{}; position: relative; z-index: 1",
            global_style_vars(),
            if !(*site_style).is_empty() { "; " } else { "" },
            (*site_style).clone()
        )}>
            {if is_component_active("header") {
                html! {
                    <header id="site-header" class="site-header" style={get_component_style("header")}>
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

            <main id="site-container" class="site-main">
                // Background video layer
                { {
                    let attrs = || {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(body) = document.body() {
                                    let url = body.get_attribute("data-bg-video-url");
                                    if let Some(url) = url {
                                        let looping = body.get_attribute("data-bg-video-loop").unwrap_or_else(|| "true".to_string());
                                        let autoplay = body.get_attribute("data-bg-video-autoplay").unwrap_or_else(|| "true".to_string());
                                        let muted = body.get_attribute("data-bg-video-muted").unwrap_or_else(|| "true".to_string());
                                        return Some((url, looping, autoplay, muted));
                                    }
                                }
                            }
                        }
                        None
                    };
                    if let Some((url, looping, autoplay, muted)) = attrs() {
                        // Detect YouTube and render an iframe background if so; otherwise use <video>
                        let is_youtube = url.contains("youtube.com") || url.contains("youtu.be");
                        if is_youtube {
                            let video_id = (|| {
                                // youtu.be/<id>
                                if let Some(idx) = url.find("youtu.be/") { return url[idx+9..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                // youtube.com/watch?v=<id>
                                if let Some(idx) = url.find("watch?v=") { return url[idx+8..].split(['&', '#']).next().map(|s| s.to_string()); }
                                // youtube.com/shorts/<id>
                                if let Some(idx) = url.find("/shorts/") { return url[idx+8..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                // youtube.com/embed/<id>
                                if let Some(idx) = url.find("/embed/") { return url[idx+7..].split(['?', '&', '#']).next().map(|s| s.to_string()); }
                                None
                            })();
                            if let Some(id) = video_id {
                                let embed_src = format!(
                                    "https://www.youtube.com/embed/{}?autoplay={}&mute={}&loop={}&playlist={}&controls=0&showinfo=0&modestbranding=1&iv_load_policy=3&rel=0&playsinline=1",
                                    id,
                                    if autoplay == "true" { 1 } else { 0 },
                                    if muted == "true" { 1 } else { 0 },
                                    if looping == "true" { 1 } else { 0 },
                                    id
                                );
                                return html! {
                                    <iframe
                                        class="bg-video-layer"
                                        src={embed_src}
                                        style="position: fixed; inset: 0; width: 100%; height: 100%; object-fit: cover; z-index: -2; pointer-events: none; border: 0;"
                                        allow="autoplay; encrypted-media; picture-in-picture"
                                        loading="eager"
                                    />
                                };
                            }
                        }
                        html!{
                            <video
                                class="bg-video-layer"
                                src={url}
                                autoplay={autoplay == "true"}
                                loop={looping == "true"}
                                muted={muted == "true"}
                                playsinline=true
                                style="position: fixed; inset: 0; width: 100%; height: 100%; object-fit: cover; z-index: -2; pointer-events: none;"
                            />
                        }
                    } else { html!{} }
                } }
                <div class="site-content" style="position: relative; z-index: 1;">
                    <div class="container" style={(*inner_container_style).clone()}>
                        {props.children.clone()}
                    </div>
                </div>
            </main>

            {if is_component_active("footer") {
                html! {
                    <footer id="site-footer" class="site-footer" style={get_component_style("footer")}>
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
            { if auth.is_authenticated && auth.user.as_ref().map(|u| u.role.as_str() == "admin").unwrap_or(false) {
                let on_toggle = {
                    let live_edit_enabled = live_edit_enabled.clone();
                    Callback::from(move |_| live_edit_enabled.set(!*live_edit_enabled))
                };
                html!{
                    <>
                        <button onclick={on_toggle} style="position: fixed; bottom: 16px; right: 16px; z-index: 9999; padding: 10px 14px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.1); background: #111; color: #fff; opacity: 0.9; pointer-events: auto;">{
                            if *live_edit_enabled { "Disable Live Edit" } else { "Enable Live Edit" }
                        }</button>
                        <LiveEditMode
                            enabled={*live_edit_enabled}
                            component_templates={(*component_templates).clone()}
                            on_templates_updated={
                                let component_templates = component_templates.clone();
                                Callback::from(move |updated: Vec<ComponentTemplate>| {
                                    if updated.is_empty() { return; }
                                    let mut map: std::collections::HashMap<i32, ComponentTemplate> = component_templates.iter().map(|t| (t.id, t.clone())).collect();
                                    for u in updated { map.insert(u.id, u); }
                                    component_templates.set(map.into_values().collect());
                                })
                            }
                        />
                    </>
                }
            } else { html!{} }}
        </div>
    }
} 