use cfg_if::cfg_if;
use crate::api::email::{SignupRequest, LoginRequest};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::ActionType;
use leptos::prelude::*;
use leptos::task::spawn_local;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use crate::api::email::{SignupResponse, LoginResponse};
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());
    let (show_signup, set_show_signup) = signal(false);

    // Signup form signals
    let (su_username, set_su_username) = signal(String::new());
    let (su_password, set_su_password) = signal(String::new());
    let (su_confirm, set_su_confirm) = signal(String::new());
    let (su_email, set_su_email) = signal(String::new());
    let (su_error, set_su_error) = signal(String::new());
    let (su_success, set_su_success) = signal(String::new());

    let on_login = move |_| {
        let u = username.get();
        let p = password.get();
        if u.trim().is_empty() || p.trim().is_empty() {
            set_error.set("Username and password are required".to_string());
            return;
        }

        // Check if password matches locally
        let password_matches = app_store.get().check_password(&u, &p);

        if password_matches {
            // Credentials correct — try local login
            let mut result: Option<Result<(String, String), String>> = None;
            app_store.update(|store| {
                result = Some(store.login_with_credentials(&u, &p));
            });

            if let Some(Ok((name, role_str))) = result {
                set_error.set(String::new()); // clear any previous error
                app_store.update(|store| {
                    store.add_notification(
                        format!("Welcome, {}!", name),
                        crate::stores::NotificationType::Success,
                    );
                });
                let user_id = app_store.get().current_user.id;
                let org_id = app_store.get().current_user.organization_id;
                undo_store.update(|undo| {
                    undo.record_action(create_action(
                        ActionType::Login,
                        "Auth",
                        &format!("User '{}' logged in", name),
                        user_id,
                        &name,
                        &role_str,
                        org_id,
                    ));
                });
                return;
            } else if let Some(Err(e)) = result {
                if !e.to_lowercase().contains("not validated") {
                    set_error.set(e);
                    return;
                }
                // Not validated locally — fall through to server check
            }
        }

        // Local validation failed or password didn't match — try server
        let u_clone = u.clone();
        let p_clone = p.clone();
        let app_store_clone = app_store;
        let undo_store_clone = undo_store;
        let set_error_clone = set_error;
        spawn_local(async move {
            let req = LoginRequest {
                username: u_clone.clone(),
                password: p_clone.clone(),
            };
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let resp = gloo_net::http::Request::post("/api/login")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;
                    match resp {
                        Ok(r) => {
                            if let Ok(login_resp) = r.json::<LoginResponse>().await {
                                if login_resp.success {
                                    if let (Some(name), Some(email)) = (login_resp.display_name, login_resp.email) {
                                        app_store_clone.update(|store| {
                                            store.login_server_validated(&name, &email);
                                        });
                                        set_error_clone.set(String::new());
                                        let user_id = app_store_clone.get().current_user.id;
                                        let org_id = app_store_clone.get().current_user.organization_id;
                                        undo_store_clone.update(|undo| {
                                            undo.record_action(create_action(
                                                ActionType::Login,
                                                "Auth",
                                                &format!("User '{}' logged in", name),
                                                user_id,
                                                &name,
                                                "Owner",
                                                org_id,
                                            ));
                                        });
                                    }
                                } else {
                                    set_error_clone.set("Invalid username or password".to_string());
                                }
                            } else {
                                set_error_clone.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(e) => {
                            set_error_clone.set(format!("Network error: {:?}", e));
                        }
                    }
                } else {
                    let _ = (req, app_store_clone, undo_store_clone, set_error_clone);
                }
            }
        });
    };

    let on_signup = move |_| {
        set_su_error.set(String::new());
        set_su_success.set(String::new());

        let u = su_username.get();
        let p = su_password.get();
        let c = su_confirm.get();
        let e = su_email.get();

        if u.trim().is_empty() {
            set_su_error.set("Username is required".to_string());
            return;
        }
        if p.len() < 3 {
            set_su_error.set("Password must be at least 3 characters".to_string());
            return;
        }
        if p != c {
            set_su_error.set("Passwords do not match".to_string());
            return;
        }
        if !e.contains('@') {
            set_su_error.set("A valid email is required".to_string());
            return;
        }

        let set_err = set_su_error.clone();
        let set_succ = set_su_success.clone();
        let set_u = set_su_username.clone();
        let set_p = set_su_password.clone();
        let set_c = set_su_confirm.clone();
        let set_e = set_su_email.clone();
        let email_for_msg = e.clone();
        let app_store_for_signup = app_store;

        spawn_local(async move {
            let req = SignupRequest {
                username: u.clone(),
                password: p.clone(),
                email: e,
            };
            cfg_if! {
                if #[cfg(feature = "hydrate")] {
                    let resp = gloo_net::http::Request::post("/api/signup")
                        .json(&req)
                        .unwrap()
                        .send()
                        .await;

                    match resp {
                        Ok(r) => {
                            if let Ok(signup_resp) = r.json::<SignupResponse>().await {
                                if signup_resp.success {
                                    // Also save credentials locally (unvalidated)
                                    app_store_for_signup.update(|store| {
                                        let _ = store.register_user(&u, &p, &email_for_msg);
                                    });
                                    set_succ.set(format!(
                                        "Account created! A validation email has been sent to {}. Check /emailvalid to validate and then sign in.",
                                        email_for_msg
                                    ));
                                    set_u.set(String::new());
                                    set_p.set(String::new());
                                    set_c.set(String::new());
                                    set_e.set(String::new());
                                } else {
                                    set_err.set(signup_resp.message);
                                }
                            } else {
                                set_err.set("Failed to parse server response".to_string());
                            }
                        }
                        Err(err) => {
                            set_err.set(format!("Network error: {:?}", err));
                        }
                    }
                } else {
                    let _ = (req, set_err, set_succ, set_u, set_p, set_c, set_e, email_for_msg, app_store_for_signup, p, u);
                }
            }
        });
    };

    view! {
        <div class="login-screen">
            <div class="login-card">
                <div class="login-header">
                    <h1>"Farley"</h1>
                    <p>"Sign in to your account"</p>
                </div>

                <div class="login-form">
                    <div class="login-field">
                        <label>"Username"</label>
                        <input
                            type="text"
                            class="login-input"
                            placeholder="Enter your username"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="login-field">
                        <label>"Password"</label>
                        <input
                            type="password"
                            class="login-input"
                            placeholder="Enter your password"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    {move || {
                        let err = error.get();
                        if err.is_empty() {
                            ().into_any()
                        } else {
                            view! {
                                <div class="login-error">{err}</div>
                            }.into_any()
                        }
                    }}

                    <button class="login-btn" on:click=on_login>
                        "Sign In"
                    </button>

                    <button class="signup-btn" on:click=move |_| set_show_signup.set(true)>
                        "Sign Up"
                    </button>
                </div>
            </div>
        </div>

        // Signup Modal
        {move || {
            if show_signup.get() {
                view! {
                    <div class="signup-overlay" on:click=move |_| set_show_signup.set(false)>
                        <div class="signup-modal" on:click=move |ev| ev.stop_propagation()>
                            <div class="signup-header">
                                <h2>"Create Account"</h2>
                                <button class="signup-close" on:click=move |_| set_show_signup.set(false)>
                                    "✕"
                                </button>
                            </div>

                            <div class="signup-form">
                                <div class="login-field">
                                    <label>"Username"</label>
                                    <input
                                        type="text"
                                        class="login-input"
                                        placeholder="Choose a username"
                                        on:input=move |ev| set_su_username.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="login-field">
                                    <label>"Email"</label>
                                    <input
                                        type="email"
                                        class="login-input"
                                        placeholder="Enter your email"
                                        on:input=move |ev| set_su_email.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="login-field">
                                    <label>"Password"</label>
                                    <input
                                        type="password"
                                        class="login-input"
                                        placeholder="Choose a password"
                                        on:input=move |ev| set_su_password.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="login-field">
                                    <label>"Confirm Password"</label>
                                    <input
                                        type="password"
                                        class="login-input"
                                        placeholder="Confirm your password"
                                        on:input=move |ev| set_su_confirm.set(event_target_value(&ev))
                                    />
                                </div>

                                {move || {
                                    let err = su_error.get();
                                    if err.is_empty() {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <div class="login-error">{err}</div>
                                        }.into_any()
                                    }
                                }}

                                {move || {
                                    let success = su_success.get();
                                    if success.is_empty() {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <div class="signup-success">{success}</div>
                                        }.into_any()
                                    }
                                }}

                                <button class="login-btn" on:click=on_signup>
                                    "Create Account"
                                </button>

                                <p class="signup-hint">
                                    "A validation email will be sent. Check "
                                    <a href="/emailvalid" target="_blank">"/emailvalid"</a>
                                    " to view it."
                                </p>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }
        }}
    }
}
