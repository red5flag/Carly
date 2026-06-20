use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();

    let on_logout = move |_| {
        let store = app_store.get();
        let user_id = store.current_user.id;
        let user_name = store.current_user.name.clone();
        let user_role = format!("{:?}", store.current_user.role);
        let org_id = store.current_user.organization_id;
        drop(store);

        undo_store.update(|u| {
            u.record_action(create_action(
                ActionType::Logout,
                "Auth",
                &format!("User '{}' logged out", user_name),
                user_id,
                &user_name,
                &user_role,
                org_id,
            ));
        });
        app_store.update(|store| store.logout());
    };

    let on_portfolio_click = move |_| {
        app_store.update(|store| {
            store.expand_tab(TabType::Portfolios);
        });
    };

    let on_contact = move |_| {
        app_store.update(|store| {
            store.add_notification(
                "Contact: support@farley.app".to_string(),
                crate::stores::NotificationType::Info,
            );
        });
    };

    view! {
        <footer class="app-footer">
            <div class="footer-section">
                <button class="footer-btn" on:click=on_logout title="Logout">
                    "⏻ Logout"
                </button>
            </div>
            <div class="footer-section">
                <button class="footer-btn" on:click=on_contact title="Contact">
                    "✉ Contact"
                </button>
            </div>
            <div class="footer-section">
                <button class="footer-btn" on:click=on_portfolio_click title="Quick access to Portfolios">
                    "📁 Portfolios"
                </button>
            </div>
        </footer>
    }
}
