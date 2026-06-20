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
    let is_advanced_search_open = move || search_store.get().is_advanced_search_open;
    let current_location = move || app_store.get().get_current_location();
    let profile_name = move || app_store.get().current_user.name.clone();

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
        search_store.update(|store| {
            store.close_advanced_search();
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

    let on_advanced_search_toggle = move |_| {
        let (uid, name, role, org) = user_info(&app_store);
        search_store.update(|store| {
            let was_open = store.is_advanced_search_open;
            store.toggle_advanced_search();
            let description = if was_open {
                "Closed advanced search"
            } else {
                "Opened advanced search"
            };
            let action = create_action(
                ActionType::Search,
                "Search",
                description,
                uid,
                &name,
                &role,
                org,
            );
            undo_store.update(|u| u.record_action(action));
        });
    };

    view! {
        // Search Panel - slides in from the right
        <div
            class="search-overlay"
            class:active=is_search_open
        >
            <div class="search-panel-header">
                <input
                    type="text"
                    class="search-input"
                    placeholder="Search across all data..."
                    prop:value={move || search_store.get().query}
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        search_store.update(|store| store.set_query(value));
                    }
                />
                <button
                    class="search-close-btn"
                    on:click=on_search_close
                >
                    "✕"
                </button>
            </div>

            {move || is_search_open().then(|| view! {
                <div
                    class="search-arrow-bar"
                    class:active=is_advanced_search_open
                    on:click=on_advanced_search_toggle
                    title="Toggle advanced search"
                >
                    <span class="search-arrow-icon">
                        {move || if is_advanced_search_open() { "▲" } else { "▼" }}
                    </span>
                </div>
            })}

            <div class="search-panel-body">
                {move || is_search_open().then(|| view! {
                    <Show when=move || is_advanced_search_open()>
                        <SearchFilters />
                    </Show>
                })}
            </div>
        </div>

        // Main Navbar - Fixed at top
        <nav class="navbar">
            // Left section: Home, Redo
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    on:click=on_home
                    title="Home"
                >
                    "⌂"
                </button>
                <button
                    class="nav-btn"
                    on:click=on_redo
                    disabled={move || !can_redo()}
                    title="Redo"
                >
                    "↻"
                </button>
            </div>

            // Middle: Location and Profile
            <div class="nav-location">
                <div class="nav-location-text">{current_location}</div>
                <div class="nav-profile-name">{profile_name}</div>
            </div>

            // Left of search: Undo button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    on:click=on_undo
                    disabled={move || !can_undo()}
                    title="Undo"
                >
                    "↺"
                </button>
            </div>

            // Right: Search button
            <div class="navbar-section">
                <button
                    class="nav-btn"
                    class:active=is_search_open
                    on:click=on_search_click
                    title="Search"
                >
                    "🔍"
                </button>
            </div>
        </nav>
    }
}
