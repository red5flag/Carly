use crate::stores::{format_action_description, use_app_store, use_undo_redo_store};
use crate::types::ActionType;
use leptos::prelude::*;

fn action_type_badge(action_type: &ActionType) -> (&'static str, &'static str) {
    match action_type {
        ActionType::Create => ("Create", "badge-create"),
        ActionType::Update => ("Update", "badge-update"),
        ActionType::Delete => ("Delete", "badge-delete"),
        ActionType::View => ("View", "badge-view"),
        ActionType::Navigate => ("Navigate", "badge-nav"),
        ActionType::Setting => ("Setting", "badge-setting"),
        ActionType::Payment => ("Payment", "badge-payment"),
        ActionType::Notification => ("Notification", "badge-notif"),
        ActionType::Search => ("Search", "badge-search"),
        ActionType::Undo => ("Undo", "badge-undo"),
        ActionType::Redo => ("Redo", "badge-redo"),
        ActionType::Login => ("Login", "badge-login"),
        ActionType::Logout => ("Logout", "badge-logout"),
    }
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let undo_store = use_undo_redo_store();
    let app_store = use_app_store();

    let action_count = move || undo_store.get().past.len();
    let recent_actions = move || undo_store.get().get_recent_actions(50);
    let current_user_name = move || app_store.get().current_user.name.clone();
    let current_user_role = move || format!("{:?}", app_store.get().current_user.role);

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"History"</h1>
                <p>{move || format!("{} recorded actions", action_count())}</p>
            </div>

            // Current user info
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Current User"</span>
                </div>
                <div class="card-stats">
                    <div class="stat-item">
                        <div class="stat-label">"Name"</div>
                        <div class="stat-value">{current_user_name}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Role"</div>
                        <div class="stat-value">{current_user_role}</div>
                    </div>
                </div>
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Actions"</span>
                </div>
                {move || {
                    let actions = recent_actions();
                    if actions.is_empty() {
                        view! {
                            <div style="padding: 20px; text-align: center; color: var(--text-secondary);">
                                <p>"No actions recorded yet"</p>
                                <div style="margin-top: 20px; font-size: 48px;">"📜"</div>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="timeline">
                                {actions
                                    .into_iter()
                                    .map(|action| {
                                        let description = format_action_description(&action);
                                        let time = action.timestamp.format("%H:%M:%S").to_string();
                                        let date = action.timestamp.format("%Y-%m-%d").to_string();
                                        let (type_label, badge_class) = action_type_badge(&action.action_type);
                                        let user_name = if action.user_name.is_empty() {
                                            "Unknown".to_string()
                                        } else {
                                            action.user_name.clone()
                                        };
                                        let user_role = if action.user_role.is_empty() {
                                            "—".to_string()
                                        } else {
                                            action.user_role.clone()
                                        };

                                        view! {
                                            <div class="timeline-item">
                                                <div class="timeline-time">{time}</div>
                                                <div class="timeline-content">
                                                    <div class="timeline-action">
                                                        <span class={format!("action-badge {}", badge_class)}>{type_label}</span>
                                                        {description}
                                                    </div>
                                                    <div class="timeline-meta">
                                                        <span class="timeline-user">{user_name}</span>
                                                        <span class="timeline-role">{user_role}</span>
                                                        <span class="timeline-date">{date}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
