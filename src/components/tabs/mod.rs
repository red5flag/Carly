use crate::pages::{AddTeamMemberPage, AgentPage, HistoryPage, NetworkingPage, OrganizationPage, OverviewPage, PortfoliosPage, ReportingPage, SettingsPage, TransactionsPage};
use crate::stores::{create_action, use_app_store, use_undo_redo_store};
use crate::types::{ActionType, TabType};
use leptos::prelude::*;

/// Per-tab edit mode signal provided as context to child pages.
#[derive(Clone, Copy)]
pub struct TabEditMode(pub ReadSignal<bool>);

pub fn use_tab_edit_mode() -> ReadSignal<bool> {
    use_context::<TabEditMode>().map(|c| c.0).unwrap_or_else(|| signal(false).0)
}

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
        let active = store.active_tab.clone();
        drop(store);

        app_store.update(|store| {
            if active == Some(TabType::Networking) {
                store.toggle_networking_add_member();
            } else {
                store.expand_tab(TabType::Settings);
            }
        });
        undo_store.update(|u| {
            let description = if active == Some(TabType::Networking) {
                "Toggled Networking add member"
            } else {
                "Opened Settings tab"
            };
            u.record_action(create_action(
                ActionType::Setting,
                "Tab",
                description,
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
    let tab_type_class = tab_type.clone();
    let tab_type_arrow = tab_type.clone();
    let tab_type_content = tab_type.clone();

    let (edit_mode, set_edit_mode) = signal(false);
    provide_context(TabEditMode(edit_mode));

    view! {
        <div class="tab-item" class:expanded=move || app_store.get().is_tab_expanded(&tab_type_class)>
            <div class="tab-header" on:click=move |_| on_toggle.run(())>
                <span class="tab-arrow">{move || if app_store.get().is_tab_expanded(&tab_type_arrow) { "▲" } else { "▼" }}</span>
                <span class="tab-title">{title}</span>
                <div class="tab-hot-options" on:click=|ev| ev.stop_propagation()>
                    <button
                        class="hot-btn"
                        class:hot-btn-active=move || edit_mode.get()
                        title="Toggle edit mode"
                        on:click=move |_| set_edit_mode.update(|v| *v = !*v)
                    >"✎"</button>
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
    let app_store = use_app_store();

    view! {
        <div class="tabs-container">
            <TabItem tab_type=TabType::Overview title="Overview" page=OverviewPage />
            <TabItem tab_type=TabType::Portfolios title="Portfolios" page=PortfoliosPage />
            <TabItem tab_type=TabType::Networking title="Networking" page=NetworkingPage />
            {move || if app_store.get().networking_add_member_open {
                view! {
                    <TabItem tab_type=TabType::NetworkingAddMember title="Add Team" page=AddTeamMemberPage />
                }.into_any()
            } else { ().into_any() }}
            <TabItem tab_type=TabType::Organization title="Organization" page=OrganizationPage />
            <TabItem tab_type=TabType::Reporting title="Reporting" page=ReportingPage />
            <TabItem tab_type=TabType::Transactions title="Transactions" page=TransactionsPage />
            <TabItem tab_type=TabType::History title="History" page=HistoryPage />
            <TabItem tab_type=TabType::Settings title="Settings" page=SettingsPage />
            <TabItem tab_type=TabType::Agent title="Agent" page=AgentPage />
        </div>
    }
}
