use yew::prelude::*;
use crate::services::auth_service::{LoginCredentials, AuthError};
use crate::services::auth_context::{use_auth, login_and_update_context};

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub on_login_success: Callback<()>,
}

#[function_component(Login)]
pub fn login_page(props: &LoginProps) -> Html {
    let auth = use_auth();
    let username = use_state(String::new);
    let password = use_state(String::new);
    let error = use_state(|| None::<String>);

    let on_username_change = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password_change = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let auth = auth.clone();
        let on_login_success = props.on_login_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            if username.is_empty() || password.is_empty() {
                error.set(Some("Please fill in all fields".to_string()));
                return;
            }

            let credentials = LoginCredentials {
                username: (*username).clone(),
                password: (*password).clone(),
            };

            let error = error.clone();
            let auth = auth.clone();
            let on_login_success = on_login_success.clone();

            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match login_and_update_context(&auth, &credentials).await {
                    Ok(_) => {
                        on_login_success.emit(());
                    }
                    Err(e) => {
                        let error_msg = match e {
                            AuthError::InvalidCredentials => "Invalid username or password".to_string(),
                            _ => format!("Login failed: {}", e),
                        };
                        error.set(Some(error_msg));
                    }
                }
            });
        })
    };

    html! {
        <div class="auth-page">
            <div class="auth-container">
                <div class="auth-header">
                    <h1>{"CMS Login"}</h1>
                    <p>{"Sign in to access the admin panel"}</p>
                </div>

                if let Some(ref error_msg) = *error {
                    <div class="error-message">{"Error: "}{error_msg}</div>
                }

                <form class="auth-form" onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="username">{"Username"}</label>
                        <input
                            type="text"
                            id="username"
                            value={(*username).clone()}
                            oninput={on_username_change}
                            placeholder="Enter your username"
                            required=true
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">{"Password"}</label>
                        <input
                            type="password"
                            id="password"
                            value={(*password).clone()}
                            oninput={on_password_change}
                            placeholder="Enter your password"
                            required=true
                        />
                    </div>

                    <button 
                        type="submit" 
                        class="btn btn-primary" 
                        disabled={auth.loading}
                    >
                        if auth.loading {
                            {"Signing in..."}
                        } else {
                            {"Sign In"}
                        }
                    </button>
                </form>

                <div class="auth-footer">
                    <p>{"Demo credentials: admin / admin"}</p>
                    <p>
                        {"Don't have an account? "}
                        <a href="/signup" class="auth-link">{"Sign up here"}</a>
                    </p>
                </div>
            </div>
        </div>
    }
} 