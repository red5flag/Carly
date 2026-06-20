use crate::pages::{AgentPage, HistoryPage, NetworkingPage, OrganizationPage, OverviewPage, PortfoliosPage, SettingsPage, TransactionsPage};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

fn use_tab_toggle(tab_type: TabType) -> Callback<()> {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    Callback::new(move |_| {
        let current_tab = tab_type.clone();
        let store = app_store.get();
        let user_id = store.current_user.id;
        let user_name = store.current_user.name.clone();
        let user_role = format!("{:?}", store.current_user.role);
        let org_id = store.current_user.organization_id;
        drop(store);

        app_store.update(|store| {
            if store.is_tab_expanded(&current_tab) {
                let action = create_action(
                    ActionType::View,
                    "Tab",
                    &format!("Closed {:?} tab", current_tab),
                    user_id,
                    &user_name,
                    &user_role,
                    org_id,
                );
                undo_store.update(|u| u.record_action(action));
                store.collapse_tab();
            } else {
                let action = create_action(
                    ActionType::View,
                    "Tab",
                    &format!("Opened {:?} tab", current_tab),
                    user_id,
                    &user_name,
                    &user_role,
                    org_id,
                );
                undo_store.update(|u| u.record_action(action));
                store.expand_tab(current_tab);
            }
        });
    })
}

fn use_open_settings_tab() -> Callback<()> {
    let app_store = use_app_store();
    let undo_store = use_undo_redo_store();
    Callback::new(move |_| {
        let store = app_store.get();
        let user_id = store.current_user.id;
        let user_name = store.current_user.name.clone();
        let user_role = format!("{:?}", store.current_user.role);
        let org_id = store.current_user.organization_id;
        drop(store);

        app_store.update(|store| {
            store.expand_tab(TabType::Settings);
        });
        undo_store.update(|u| {
            u.record_action(create_action(
                ActionType::Setting,
                "Tab",
                "Opened Settings tab",
                user_id,
                &user_name,
                &user_role,
                org_id,
            ))
        });
    })
}

#[component]
fn TabItem<F, IV>(
    tab_type: TabType,
    title: &'static str,
    page: F,
) -> impl IntoView
where
    F: Fn() -> IV + Send + 'static,
    IV: IntoView + Send + 'static,
{
    let app_store = use_app_store();
    let on_toggle = use_tab_toggle(tab_type.clone());
    let on_settings = use_open_settings_tab();
    let tab_type_class = tab_type.clone();
    let tab_type_arrow = tab_type.clone();
    let tab_type_content = tab_type.clone();

    view! {
        <div class="tab-item" class:expanded=move || app_store.get().is_tab_expanded(&tab_type_class)>
            <div class="tab-header" on:click=move |_| on_toggle.run(())>
                <span class="tab-arrow">{move || if app_store.get().is_tab_expanded(&tab_type_arrow) { "▲" } else { "▼" }}</span>
                <span class="tab-title">{title}</span>
                <div class="tab-hot-options" on:click=|ev| ev.stop_propagation()>
                    <button class="hot-btn" title="Settings" on:click=move |_| on_settings.run(())>"⚙"</button>
                </div>
            </div>
            {move || {
                if app_store.get().is_tab_expanded(&tab_type_content) {
                    view! {
                        <div class="tab-content" on:click=|ev| ev.stop_propagation()>
                            {page()}
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn TabsContainer() -> impl IntoView {
    view! {
        <div class="tabs-container">
            <TabItem tab_type=TabType::Overview title="Overview" page=OverviewPage />
            <TabItem tab_type=TabType::Portfolios title="Portfolios" page=PortfoliosPage />
            <TabItem tab_type=TabType::Networking title="Networking" page=NetworkingPage />
            <TabItem tab_type=TabType::Organization title="Organization" page=OrganizationPage />
            <TabItem tab_type=TabType::Transactions title="Transactions" page=TransactionsPage />
            <TabItem tab_type=TabType::History title="History" page=HistoryPage />
            <TabItem tab_type=TabType::Settings title="Settings" page=SettingsPage />
            <TabItem tab_type=TabType::Agent title="Agent" page=AgentPage />
        </div>
    }
}
