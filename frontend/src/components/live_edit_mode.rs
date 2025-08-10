use yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{window, HtmlElement};
use serde_json::json;

use crate::services::navigation_service::{ComponentTemplate, update_component_template};
use crate::services::api_service::{SettingData, update_settings};

#[derive(Properties, PartialEq, Clone)]
pub struct LiveEditModeProps {
    pub enabled: bool,
    pub component_templates: Vec<ComponentTemplate>,
    pub on_templates_updated: Callback<Vec<ComponentTemplate>>,
}

#[derive(Clone, PartialEq)]
enum EditScope {
    Header,
    Footer,
    Container,
}

#[function_component(LiveEditMode)]
pub fn live_edit_mode(props: &LiveEditModeProps) -> Html {
    let selected_scope = use_state(|| Option::<EditScope>::None);
    let working_bg = use_state(String::new); // generic bg input
    let working_text = use_state(String::new); // generic text color input
    let working_video_url = use_state(String::new);
    let working_overlay_color = use_state(String::new);
    let working_overlay_opacity = use_state(|| "0".to_string());
    let working_bg_type = use_state(|| "none".to_string());

    {
        let selected_scope = selected_scope.clone();
        use_effect_with_deps(move |enabled| {
            // Apply highlight styles when enabled; remove them on cleanup to avoid interfering with acid mode visuals
            if *enabled {
                if let Some(doc) = window().and_then(|w| w.document()) {
                    for id in ["site-header", "site-footer", "site-container"] {
                        if let Some(el) = doc.get_element_by_id(id) {
                            let _ = el.set_attribute("data-live-editable", "true");
                            let _ = el.set_attribute("data-live-outline", "true");
                            if let Some(existing) = el.get_attribute("style") {
                                let _ = el.set_attribute("style", &format!("{}; outline: 2px dashed rgba(255,255,255,0.6); outline-offset: -2px;", existing));
                            } else {
                                let _ = el.set_attribute("style", "outline: 2px dashed rgba(255,255,255,0.6); outline-offset: -2px;");
                            }
                            let selected_scope_clone = selected_scope.clone();
                            let closure: Closure<dyn FnMut(web_sys::Event)> = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                if let Some(target) = e.target().and_then(|t| t.dyn_into::<HtmlElement>().ok()) {
                                    let id = target.id();
                                    let scope = match id.as_str() {
                                        "site-header" => Some(EditScope::Header),
                                        "site-footer" => Some(EditScope::Footer),
                                        "site-container" => Some(EditScope::Container),
                                        _ => None,
                                    };
                                    selected_scope_clone.set(scope);
                                }
                            }) as Box<dyn FnMut(_)>);
                            let _ = el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                            closure.forget();
                        }
                    }
                }
            }
            // Cleanup
            || {
                if let Some(doc) = window().and_then(|w| w.document()) {
                    for id in ["site-header", "site-footer", "site-container"] {
                        if let Some(el) = doc.get_element_by_id(id) {
                            let _ = el.remove_attribute("data-live-editable");
                            if let Some(existing) = el.get_attribute("style") {
                                // Strip our outline directives if present
                                let cleaned = existing
                                    .replace("outline: 2px dashed rgba(255,255,255,0.6);", "")
                                    .replace("outline-offset: -2px;", "");
                                let _ = el.set_attribute("style", cleaned.trim());
                            }
                            let _ = el.remove_attribute("data-live-outline");
                        }
                    }
                }
            }
        }, props.enabled);
    }

    // Initialize working values when scope changes
    {
        let working_bg = working_bg.clone();
        let working_text = working_text.clone();
        let working_video_url = working_video_url.clone();
        let working_overlay_color = working_overlay_color.clone();
        let working_overlay_opacity = working_overlay_opacity.clone();
        let working_bg_type = working_bg_type.clone();
        let templates = props.component_templates.clone();
        use_effect_with_deps(move |scope| {
            match &**scope {
                Some(EditScope::Header) => {
                    if let Some(tpl) = templates.iter().find(|t| t.component_type == "header" && t.is_active) {
                        let bg = tpl.template_data.get("background").and_then(|v| v.as_str()).unwrap_or("");
                        let text = tpl.template_data.get("text_color").and_then(|v| v.as_str()).unwrap_or("");
                        working_bg.set(bg.to_string());
                        working_text.set(text.to_string());
                    }
                }
                Some(EditScope::Footer) => {
                    if let Some(tpl) = templates.iter().find(|t| t.component_type == "footer" && t.is_active) {
                        let bg = tpl.template_data.get("background").and_then(|v| v.as_str()).unwrap_or("");
                        let text = tpl.template_data.get("text_color").and_then(|v| v.as_str()).unwrap_or("");
                        working_bg.set(bg.to_string());
                        working_text.set(text.to_string());
                    }
                }
                Some(EditScope::Container) => {
                    if let Some(doc) = window().and_then(|w| w.document()) {
                        let body = doc.body();
                        let url = body.as_ref().and_then(|b| b.get_attribute("data-bg-video-url")).unwrap_or_default();
                        let overlay_color = body.as_ref().and_then(|b| b.get_attribute("data-overlay-color")).unwrap_or_default();
                        let overlay_opacity = body.as_ref().and_then(|b| b.get_attribute("data-overlay-opacity")).unwrap_or_else(|| "0".to_string());
                        let bg_type = body.as_ref().and_then(|b| b.get_attribute("data-bg-type")).unwrap_or_else(|| "none".to_string());
                        working_video_url.set(url);
                        working_overlay_color.set(overlay_color);
                        working_overlay_opacity.set(overlay_opacity);
                        working_bg_type.set(bg_type);
                    }
                }
                None => {}
            }
            || ()
        }, selected_scope.clone());
    }

    let on_close_panel = {
        let selected_scope = selected_scope.clone();
        Callback::from(move |_| selected_scope.set(None))
    };

    let on_input_bg = {
        let working_bg = working_bg.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            working_bg.set(input.value());
        })
    };
    let on_input_text = {
        let working_text = working_text.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            working_text.set(input.value());
        })
    };
    let on_input_video_url = {
        let working_video_url = working_video_url.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            working_video_url.set(input.value());
        })
    };
    let on_input_overlay_color = {
        let working_overlay_color = working_overlay_color.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            working_overlay_color.set(input.value());
        })
    };
    let on_input_overlay_opacity = {
        let working_overlay_opacity = working_overlay_opacity.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            working_overlay_opacity.set(input.value());
        })
    };
    let on_change_bg_type = {
        let working_bg_type = working_bg_type.clone();
        Callback::from(move |e: Event| {
            let select = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            working_bg_type.set(select.value());
        })
    };

    let on_save = {
        let scope = selected_scope.clone();
        let bg = (*working_bg).clone();
        let text = (*working_text).clone();
        let video_url = (*working_video_url).clone();
        let overlay_color = (*working_overlay_color).clone();
        let overlay_opacity = (*working_overlay_opacity).clone();
        let bg_type = (*working_bg_type).clone();
        let templates = props.component_templates.clone();
        let on_templates_updated = props.on_templates_updated.clone();
        Callback::from(move |_| {
            let scope_now = (*scope).clone();
            let bg = bg.clone();
            let text = text.clone();
            let video_url = video_url.clone();
            let overlay_color = overlay_color.clone();
            let overlay_opacity = overlay_opacity.clone();
            let bg_type = bg_type.clone();
            let templates = templates.clone();
            let on_templates_updated = on_templates_updated.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match scope_now {
                    Some(EditScope::Header) => {
                        if let Some(mut tpl) = templates.into_iter().find(|t| t.component_type == "header" && t.is_active) {
                            let mut data = tpl.template_data.clone();
                            data["background"] = json!(bg);
                            if !text.is_empty() { data["text_color"] = json!(text); }
                            tpl.template_data = data;
                            if let Ok(updated) = update_component_template(tpl.id, &tpl).await {
                                // Ideally refetch all templates; for now, signal update with single replacement
                                on_templates_updated.emit(vec![updated]);
                            }
                        }
                    }
                    Some(EditScope::Footer) => {
                        if let Some(mut tpl) = templates.into_iter().find(|t| t.component_type == "footer" && t.is_active) {
                            let mut data = tpl.template_data.clone();
                            data["background"] = json!(bg);
                            if !text.is_empty() { data["text_color"] = json!(text); }
                            tpl.template_data = data;
                            if let Ok(updated) = update_component_template(tpl.id, &tpl).await {
                                on_templates_updated.emit(vec![updated]);
                            }
                        }
                    }
                    Some(EditScope::Container) => {
                        let mut settings = Vec::<SettingData>::new();
                        settings.push(SettingData { key: "container_background_type".to_string(), value: bg_type.clone(), setting_type: "container".to_string(), description: Some("Background mode".to_string()) });
                        if !video_url.is_empty() {
                            settings.push(SettingData { key: "container_background_video_url".to_string(), value: video_url.clone(), setting_type: "container".to_string(), description: Some("Background video URL".to_string()) });
                        }
                        if !overlay_color.is_empty() {
                            settings.push(SettingData { key: "container_overlay_color".to_string(), value: overlay_color.clone(), setting_type: "container".to_string(), description: Some("Overlay color".to_string()) });
                        }
                        settings.push(SettingData { key: "container_overlay_opacity".to_string(), value: overlay_opacity.clone(), setting_type: "container".to_string(), description: Some("Overlay opacity".to_string()) });
                        if !bg.is_empty() {
                            settings.push(SettingData { key: "container_background_color".to_string(), value: bg.clone(), setting_type: "container".to_string(), description: Some("Background color".to_string()) });
                        }
                        let _ = update_settings(settings).await; // ignore errors in UI for now
                        // Update body attributes so PublicLayout's inline reader can pick up immediately
                        if let Some(doc) = window().and_then(|w| w.document()) {
                            if let Some(body) = doc.body() {
                                let _ = body.set_attribute("data-bg-video-url", &video_url);
                                let _ = body.set_attribute("data-overlay-color", &overlay_color);
                                let _ = body.set_attribute("data-overlay-opacity", &overlay_opacity);
                                let _ = body.set_attribute("data-bg-type", &bg_type);
                                // Toggle acid-mode based on saved state in body data or bg_type proxy
                                let existing = body.get_attribute("class").unwrap_or_default();
                                if bg_type == "acid" || body.get_attribute("data-acid-enabled").as_deref() == Some("true") {
                                    if !existing.split_whitespace().any(|c| c == "acid-mode") {
                                        let new_class = if existing.is_empty() { "acid-mode".to_string() } else { format!("{} acid-mode", existing) };
                                        let _ = body.set_attribute("class", &new_class);
                                    }
                                }
                            }
                        }
                    }
                    None => {}
                }
            });
        })
    };

    if !props.enabled { return html!{}; }

    html! {
        <div class="live-edit-ui" style="position: fixed; top: 0; right: 0; padding: 8px; z-index: 9999;">
            <span class="live-edit-badge" style="background: rgba(0,0,0,0.7); color: white; padding: 6px 10px; border-radius: 6px; font-size: 12px;">{"Live Edit: click a highlighted area"}</span>
            {
                if let Some(scope) = &*selected_scope {
                    let (title, body) = match scope {
                        EditScope::Header => ("Header", html!{
                            <>
                                <label>{"Background"}</label>
                                <input type="text" value={(*working_bg).clone()} oninput={on_input_bg.clone()} />
                                <label>{"Text color"}</label>
                                <input type="text" value={(*working_text).clone()} oninput={on_input_text.clone()} />
                            </>
                        }),
                        EditScope::Footer => ("Footer", html!{
                            <>
                                <label>{"Background"}</label>
                                <input type="text" value={(*working_bg).clone()} oninput={on_input_bg.clone()} />
                                <label>{"Text color"}</label>
                                <input type="text" value={(*working_text).clone()} oninput={on_input_text.clone()} />
                            </>
                        }),
                        EditScope::Container => ("Container", html!{
                            <>
                                <label>{"Background type"}</label>
                                <select onchange={on_change_bg_type.clone()} value={(*working_bg_type).clone()}>
                                    <option value="none">{"None"}</option>
                                    <option value="color">{"Color"}</option>
                                    <option value="gradient">{"Gradient"}</option>
                                    <option value="image">{"Image"}</option>
                                    <option value="video">{"Video"}</option>
                                </select>
                                <label>{"Background / Gradient"}</label>
                                <input type="text" value={(*working_bg).clone()} oninput={on_input_bg.clone()} />
                                <label>{"Video URL"}</label>
                                <input type="text" value={(*working_video_url).clone()} oninput={on_input_video_url.clone()} />
                                <label>{"Overlay color"}</label>
                                <input type="text" value={(*working_overlay_color).clone()} oninput={on_input_overlay_color.clone()} />
                                <label>{"Overlay opacity (0..1 or % )"}</label>
                                <input type="text" value={(*working_overlay_opacity).clone()} oninput={on_input_overlay_opacity.clone()} />
                            </>
                        }),
                    };
                    html!{
                        <div class="live-edit-panel" style="position: fixed; top: 40px; right: 8px; background: white; padding: 12px; border-radius: 8px; box-shadow: 0 6px 24px rgba(0,0,0,0.2); min-width: 260px;">
                            <div style="display:flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                                <strong>{format!("Edit {}", title)}</strong>
                                <button onclick={on_close_panel.clone()}>{"×"}</button>
                            </div>
                            <div style="display: grid; gap: 6px;">{body}</div>
                            <div style="display:flex; justify-content: flex-end; gap: 8px; margin-top: 10px;">
                                <button class="btn-primary" onclick={on_save}>{"Save"}</button>
                            </div>
                        </div>
                    }
                } else { html!{} }
            }
        </div>
    }
}


