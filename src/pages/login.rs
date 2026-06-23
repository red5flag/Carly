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

    let (changelog_open, set_changelog_open) = signal(false);
    let (show_username, set_show_username) = signal(true);
    let (show_password, set_show_password) = signal(false);
    let (su_show_pass, set_su_show_pass) = signal(false);
    let (su_show_confirm, set_su_show_confirm) = signal(false);

    view! {
        <div class="lp-screen">

            // ── RED HEADER ──
            <div class="lp-header">
                <div class="lp-logo">"C"</div>
                <div class="lp-title">"LOGIN"</div>
                <div class="lp-version">"2.1.1"</div>
            </div>

            // ── SAVED PROFILES ROW ──
            <div class="lp-saved-row">
                <div class="lp-saved-icon">"⚙"</div>
                <div class="lp-saved-label">"SAVED PROFILES"</div>
                <div class="lp-saved-arrow">"▼"</div>
            </div>

            // ── USERNAME ROW ──
            <div class="lp-field-row">
                <button class="lp-visibility-btn" title="Toggle username visibility"
                    on:click=move |_| set_show_username.update(|v| *v = !*v)>
                    {move || if show_username.get() { "◉" } else { "○" }}
                </button>
                {move || {
                    let vis = show_username.get();
                    view! {
                        <input
                            type={if vis { "text" } else { "password" }}
                            class="lp-field-input"
                            placeholder="Username"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    }
                }}
            </div>

            // ── PASSWORD ROW ──
            <div class="lp-field-row">
                <button class="lp-visibility-btn" title="Toggle password visibility"
                    on:click=move |_| set_show_password.update(|v| *v = !*v)>
                    {move || if show_password.get() { "◉" } else { "○" }}
                </button>
                {move || {
                    let vis = show_password.get();
                    view! {
                        <input
                            type={if vis { "text" } else { "password" }}
                            class="lp-field-input"
                            placeholder="Password"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    }
                }}
            </div>

            // ── ERROR ──
            {move || {
                let err = error.get();
                if err.is_empty() { ().into_any() } else {
                    view! { <div class="lp-error">{err}</div> }.into_any()
                }
            }}

            // ── REGISTER / LOGIN BUTTONS ──
            <div class="lp-action-row">
                <button class="lp-action-btn lp-register" on:click=move |_| set_show_signup.set(true)>
                    "REGISTER"
                </button>
                <button class="lp-action-btn lp-login" on:click=on_login>
                    "LOGIN"
                </button>
            </div>

            // ── OAUTH / 2FA ROW ──
            <div class="lp-oauth-row">
                <button class="lp-oauth-btn">"GMAIL"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn">"LINKEDIN"</button>
                <div class="lp-oauth-divider" />
                <button class="lp-oauth-btn">"2FA"</button>
            </div>

            // ── CHANGELOG ACCORDION ──
            <div class="lp-changelog">
                <div class="lp-changelog-header" on:click=move |_| set_changelog_open.update(|v| *v = !*v)>
                    <span>"CHANGELOG"</span>
                    <span>{move || if changelog_open.get() { "▲" } else { "▼" }}</span>
                </div>
                {move || if changelog_open.get() {
                    view! {
                        <div class="lp-changelog-body">
                            <div class="lp-changelog-summary">"Brief summary of notable changes"</div>
                            <ul class="lp-changelog-list">
                                <li>"Changes"</li>
                                <li>"Changes"</li>
                                <li>"Changes"</li>
                                <li>"Changes"</li>
                                <li>"Changes"</li>
                            </ul>
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            // ── FOOTER ──
            <div class="lp-footer">
                <button class="lp-footer-btn">"NEWS"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn">"ABOUT"</button>
                <div class="lp-footer-divider" />
                <button class="lp-footer-btn">"HELP"</button>
            </div>

        </div>

        // ── REGISTER SCREEN (lp-style) ──
        {move || {
            if show_signup.get() {
                view! {
                    <div class="lp-screen lp-register-screen">
                        <div class="lp-header">
                            <div class="lp-logo">"C"</div>
                            <div class="lp-title">"REGISTER"</div>
                            <button class="lp-close-btn" on:click=move |_| set_show_signup.set(false)>"✕"</button>
                        </div>

                        <div class="lp-saved-row" style="cursor:default;">
                            <div class="lp-saved-label">"Create a new account"</div>
                        </div>

                        // Username
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"◉"</div>
                            <input type="text" class="lp-field-input" placeholder="Username"
                                on:input=move |ev| set_su_username.set(event_target_value(&ev)) />
                        </div>

                        // Email
                        <div class="lp-field-row">
                            <div class="lp-visibility-btn" style="cursor:default;opacity:0.5;">"@"</div>
                            <input type="email" class="lp-field-input" placeholder="Email"
                                on:input=move |ev| set_su_email.set(event_target_value(&ev)) />
                        </div>

                        // Password
                        <div class="lp-field-row">
                            <button class="lp-visibility-btn" on:click=move |_| set_su_show_pass.update(|v| *v = !*v)>
                                {move || if su_show_pass.get() { "◉" } else { "○" }}
                            </button>
                            {move || {
                                let vis = su_show_pass.get();
                                view! {
                                    <input type={if vis { "text" } else { "password" }}
                                        class="lp-field-input" placeholder="Password"
                                        on:input=move |ev| set_su_password.set(event_target_value(&ev)) />
                                }
                            }}
                        </div>

                        // Confirm password
                        <div class="lp-field-row">
                            <button class="lp-visibility-btn" on:click=move |_| set_su_show_confirm.update(|v| *v = !*v)>
                                {move || if su_show_confirm.get() { "◉" } else { "○" }}
                            </button>
                            {move || {
                                let vis = su_show_confirm.get();
                                view! {
                                    <input type={if vis { "text" } else { "password" }}
                                        class="lp-field-input" placeholder="Confirm Password"
                                        on:input=move |ev| set_su_confirm.set(event_target_value(&ev)) />
                                }
                            }}
                        </div>

                        // Error / success
                        {move || {
                            let err = su_error.get();
                            if err.is_empty() { ().into_any() } else {
                                view! { <div class="lp-error">{err}</div> }.into_any()
                            }
                        }}
                        {move || {
                            let s = su_success.get();
                            if s.is_empty() { ().into_any() } else {
                                view! { <div class="lp-success">{s}</div> }.into_any()
                            }
                        }}

                        // Action row
                        <div class="lp-action-row">
                            <button class="lp-action-btn lp-register"
                                on:click=move |_| set_show_signup.set(false)>
                                "BACK"
                            </button>
                            <button class="lp-action-btn lp-login" on:click=on_signup>
                                "CREATE"
                            </button>
                        </div>

                        <div class="lp-footer">
                            <button class="lp-footer-btn">"NEWS"</button>
                            <div class="lp-footer-divider" />
                            <button class="lp-footer-btn">"ABOUT"</button>
                            <div class="lp-footer-divider" />
                            <button class="lp-footer-btn">"HELP"</button>
                        </div>
                    </div>
                }.into_any()
            } else { ().into_any() }
        }}
    }
}
