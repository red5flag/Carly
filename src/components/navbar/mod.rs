use crate::components::search::SearchFilters;
use crate::stores::{create_action, use_app_store, use_search_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    let search_store = use_search_store();

    // Derived signals
    let can_undo = move || undo_store.get().can_undo();
    let can_redo = move || undo_store.get().can_redo();
    let is_search_open = move || app_store.get().is_search_open;
    let current_location = move || app_store.get().get_current_location();
    let profile_name = move || app_store.get().current_user.name.clone();
    let profile_role = move || format!("{:?}", app_store.get().current_user.role);
    let notification_count = move || app_store.get().notifications.len();

    // Helper to get user info tuple
    fn user_info(app_store: &leptos::prelude::RwSignal<crate::stores::AppStore>) -> (uuid::Uuid, String, String, Option<uuid::Uuid>) {
        let store = app_store.get();
        (store.current_user.id, store.current_user.name.clone(), format!("{:?}", store.current_user.role), store.current_user.organization_id)
    }

    // Handlers
    let on_home = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        let action = create_action(
            ActionType::Navigate,
            "App",
            "Returned home",
            uid,
            &name,
            &role,
            org,
        );
        undo_store.update(|u| u.record_action(action));
        app_store.update(|store| {
            store.collapse_tab();
            store.close_search();
        });
    };

    let on_redo = move |_| {
        if let Some(redone) = undo_store.get().redo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Redo,
                "Action",
                &format!("Redid: {}", redone.description),
                uid,
                &name,
                &role,
                org,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Redo: {:?}", redone);
        }
    };

    let on_undo = move |_| {
        if let Some(undone) = undo_store.get().undo() {
            let (uid, name, role, org) = user_info(&app_store);
            let action = create_action(
                ActionType::Undo,
                "Action",
                &format!("Undid: {}", undone.description),
                uid,
                &name,
                &role,
                org,
            );
            undo_store.update(|u| u.record_action(action));
            tracing::info!("Undo: {:?}", undone);

            if let Some(ref from) = undone.navigated_from {
                tracing::info!("Navigating back to: {}", from);
            }
        }
    };

    let on_search_click = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        app_store.update(|store| {
            if store.is_search_open {
                store.close_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Closed search",
                    uid,
                    &name,
                    &role,
                    org,
                );
                undo_store.update(|u| u.record_action(action));
            } else {
                store.open_search();
                let action = create_action(
                    ActionType::Search,
                    "Search",
                    "Opened search",
                    uid,
                    &name,
                    &role,
                    org,
                );
                undo_store.update(|u| u.record_action(action));
            }
        });
        search_store.update(|store| {
            store.set_context_tab(
                app_store
                    .get()
                    .active_tab
                    .clone()
                    .unwrap_or(TabType::Overview),
            );
        });
    };

    let on_search_close = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        app_store.update(|store| {
            store.close_search();
        });
        let action = create_action(
            ActionType::Search,
            "Search",
            "Closed search",
            uid,
            &name,
            &role,
            org,
        );
        undo_store.update(|u| u.record_action(action));
    };

    view! {
        // Main Navbar - Fixed at top, two rows
        <nav class="navbar">
            // ROW 1: buttons OR search bar when open
            <div class="navbar-row navbar-row-1">
                {move || if is_search_open() {
                    view! {
                        <div class="nav-search-bar-wrap">
                            <input
                                type="text"
                                class="nav-search-bar-input"
                                placeholder="Search..."
                                prop:value={move || search_store.get().query}
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    search_store.update(|s| s.set_query(v));
                                }
                            />
                            <button class="nav-search-close-btn" on:click=on_search_close>"✕"</button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="nav-row1-left">
                            <button class="nav-btn" on:click=on_home title="Home">"⌂"</button>
                            <button class="nav-btn" on:click=on_redo
                                disabled={move || !can_redo()} title="Redo">"↻"</button>
                        </div>
                        <div class="nav-row1-centre">
                            <span class="nav-profile-name-top">{profile_name}</span>
                        </div>
                        <div class="nav-row1-right">
                            <button class="nav-btn" on:click=on_undo
                                disabled={move || !can_undo()} title="Undo">"↺"</button>
                            <button class="nav-btn nav-search-btn" on:click=on_search_click title="Search">"🔍"</button>
                        </div>
                    }.into_any()
                }}
            </div>

            // ROW 2: Avatar | [centre: Location above Role] | Notifications
            <div class="navbar-row navbar-row-2">
                <div class="nav-avatar">"⚙"</div>
                <div class="nav-centre">
                    <div class="nav-centre-top">
                        <span class="nav-location-label">{current_location}</span>
                    </div>
                    <div class="nav-profile-role">{profile_role}</div>
                </div>
                <div class="nav-notif-wrap">
                    <div class="nav-notif-icon">"✉"</div>
                    {move || {
                        let count = notification_count();
                        if count > 0 {
                            view! { <div class="nav-notif-badge">{count}</div> }.into_any()
                        } else { ().into_any() }
                    }}
                </div>
            </div>
        </nav>

        // Search panel - drops below navbar when open
        {move || if is_search_open() {
            view! {
                <div class="search-drop-panel">
                    <SearchFilters />
                </div>
            }.into_any()
        } else { ().into_any() }}
    }
}
