use crate::models::{Organization, User};
use crate::stores::use_app_store;
use crate::types::UserRole;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn OrganizationPage() -> impl IntoView {
    let app_store = use_app_store();

    let organizations = Memo::new(move |_| app_store.get().organizations.clone());
    let org_users = Memo::new(move |_| app_store.get().organization_users.clone());

    let can_manage = move || {
        let role = app_store.get().current_user.role.clone();
        role == UserRole::Owner || role == UserRole::Manager
    };

    // Add org form
    let (show_add_org, set_show_add_org) = signal(false);
    let (new_org_name, set_new_org_name) = signal(String::new());
    let (new_org_desc, set_new_org_desc) = signal(String::new());

    // Add member form
    let (show_add_member, set_show_add_member) = signal(Option::<Uuid>::None);
    let (member_name, set_member_name) = signal(String::new());
    let (member_email, set_member_email) = signal(String::new());
    let (member_role, set_member_role) = signal(UserRole::Worker);

    // Edit org form
    let (editing_org, set_editing_org) = signal(Option::<Uuid>::None);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_desc, set_edit_desc) = signal(String::new());

    let on_add_org = move |_| {
        let name = new_org_name.get();
        if name.trim().is_empty() {
            return;
        }
        let owner_id = app_store.get().current_user.id;
        let mut org = Organization::new(name, owner_id);
        org.description = if new_org_desc.get().trim().is_empty() {
            None
        } else {
            Some(new_org_desc.get())
        };
        app_store.update(|s| s.add_organization(org));
        set_new_org_name.set(String::new());
        set_new_org_desc.set(String::new());
        set_show_add_org.set(false);
    };

    let on_delete_org = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_organization(id);
        });
    };

    let on_start_edit = move |id: Uuid, name: String, desc: Option<String>| {
        set_edit_name.set(name);
        set_edit_desc.set(desc.unwrap_or_default());
        set_editing_org.set(Some(id));
    };

    let on_save_edit = move |id: Uuid| {
        let name = edit_name.get();
        if name.trim().is_empty() {
            return;
        }
        app_store.update(|s| {
            if let Some(org) = s.get_organization_mut(id) {
                org.name = name;
                org.description = if edit_desc.get().trim().is_empty() {
                    None
                } else {
                    Some(edit_desc.get())
                };
                org.updated_at = chrono::Utc::now();
            }
        });
        set_editing_org.set(None);
    };

    let on_add_member = move |org_id: Uuid| {
        let name = member_name.get();
        let email = member_email.get();
        if name.trim().is_empty() || email.trim().is_empty() {
            return;
        }
        let mut user = User::new(name, email, member_role.get());
        user.organization_id = Some(org_id);
        app_store.update(|s| {
            s.add_organization_user(user.clone());
            if let Some(org) = s.get_organization_mut(org_id) {
                org.add_member(user.id);
            }
        });
        set_member_name.set(String::new());
        set_member_email.set(String::new());
        set_member_role.set(UserRole::Worker);
        set_show_add_member.set(None);
    };

    let on_remove_member = move |org_id: Uuid, user_id: Uuid| {
        app_store.update(|s| {
            s.remove_organization_user(user_id);
            if let Some(org) = s.get_organization_mut(org_id) {
                org.remove_member(user_id);
            }
        });
    };

    let on_update_member_role = move |user_id: Uuid, new_role: UserRole| {
        app_store.update(|s| {
            let _ = s.update_user_role(user_id, new_role);
        });
    };

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Organization"</h1>
                <p>"Manage your organizations and team members"</p>
            </div>

            // Add Organization button
            {move || if can_manage() {
                view! {
                    <div class="view-toggle">
                        <button
                            class="view-btn"
                            class:active=show_add_org
                            on:click=move |_| set_show_add_org.update(|v| *v = !*v)
                        >
                            "+ Add Organization"
                        </button>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            // Add Org Form
            {move || show_add_org.get().then(|| view! {
                <div class="add-form">
                    <input class="login-input" type="text" placeholder="Organization name"
                        on:input=move |ev| set_new_org_name.set(event_target_value(&ev)) />
                    <input class="login-input" type="text" placeholder="Description (optional)"
                        on:input=move |ev| set_new_org_desc.set(event_target_value(&ev)) />
                    <button class="login-btn" on:click=on_add_org>"Create Organization"</button>
                </div>
            })}

            // Organizations List
            {move || {
                let orgs = organizations.get();
                let users = org_users.get();
                let can = can_manage();
                let editing = editing_org.get();

                if orgs.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state">
                                <div class="empty-text">"No organizations yet. Create one to get started."</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="data-list">
                            {orgs.into_iter().map(|org| {
                                let oid = org.id;
                                let oid_del = org.id;
                                let oid_member = org.id;
                                let oid_edit = org.id;
                                let org_name = org.name.clone();
                                let org_desc = org.description.clone();
                                let is_editing = editing == Some(oid);
                                let member_ids = org.members.clone();
                                let org_members: Vec<User> = users.iter()
                                    .filter(|u| member_ids.contains(&u.id))
                                    .cloned()
                                    .collect();

                                view! {
                                    <div class="data-card">
                                        {if is_editing {
                                            view! {
                                                <div class="add-form">
                                                    <input class="login-input" type="text" placeholder="Organization name"
                                                        prop:value=edit_name
                                                        on:input=move |ev| set_edit_name.set(event_target_value(&ev)) />
                                                    <input class="login-input" type="text" placeholder="Description"
                                                        prop:value=edit_desc
                                                        on:input=move |ev| set_edit_desc.set(event_target_value(&ev)) />
                                                    <button class="login-btn" on:click=move |_| on_save_edit(oid_edit)>
                                                        "Save Changes"
                                                    </button>
                                                    <button class="view-btn" on:click=move |_| set_editing_org.set(None)>
                                                        "Cancel"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="card-header">
                                                    <span class="card-title">{org.name.clone()}</span>
                                                    <div class="card-actions">
                                                        {if can {
                                                            let on_edit = org_name.clone();
                                                            let on_edit_desc = org_desc.clone();
                                                            view! {
                                                                <button class="card-btn"
                                                                    on:click=move |_| on_start_edit(oid_edit, on_edit.clone(), on_edit_desc.clone())>
                                                                    "✎ Edit"
                                                                </button>
                                                                <button class="card-btn sell"
                                                                    on:click=move |_| on_delete_org(oid_del)>
                                                                    "🗑 Delete"
                                                                </button>
                                                            }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                </div>
                                                <div class="card-stats">
                                                    <div class="stat-item">
                                                        <div class="stat-label">"Members"</div>
                                                        <div class="stat-value">{org.members.len()}</div>
                                                    </div>
                                                    <div class="stat-item">
                                                        <div class="stat-label">"Created"</div>
                                                        <div class="stat-value">
                                                            {org.created_at.format("%Y-%m-%d").to_string()}
                                                        </div>
                                                    </div>
                                                </div>
                                                {move || org.description.as_ref().map(|d| view! {
                                                    <div style="padding: 8px 0; color: var(--text-secondary); font-size: 13px;">
                                                        {d.clone()}
                                                    </div>
                                                })}
                                            }.into_any()
                                        }}

                                        // Members section
                                        <div style="margin-top: 12px; padding-top: 12px; border-top: 2px solid var(--border-color);">
                                            <div class="asset-section-title">
                                                "Members"
                                                {move || if can {
                                                    let oid_m = oid_member;
                                                    view! {
                                                        <button class="add-btn-small"
                                                            on:click=move |_| set_show_add_member.set(Some(oid_m))>
                                                            "+ Add Member"
                                                        </button>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>

                                            {move || show_add_member.get().map(|gp| {
                                                if gp == oid_member {
                                                    view! {
                                                        <div class="add-form">
                                                            <input class="login-input" type="text" placeholder="Member name"
                                                                on:input=move |ev| set_member_name.set(event_target_value(&ev)) />
                                                            <input class="login-input" type="email" placeholder="Email"
                                                                on:input=move |ev| set_member_email.set(event_target_value(&ev)) />
                                                            <select class="login-input"
                                                                on:change=move |ev| {
                                                                    let v = event_target_value(&ev);
                                                                    let r = match v.as_str() {
                                                                        "Owner" => UserRole::Owner,
                                                                        "Manager" => UserRole::Manager,
                                                                        "Worker" => UserRole::Worker,
                                                                        _ => UserRole::Guest,
                                                                    };
                                                                    set_member_role.set(r);
                                                                }
                                                            >
                                                                <option value="Worker">"Worker"</option>
                                                                <option value="Manager">"Manager"</option>
                                                                <option value="Owner">"Owner"</option>
                                                                <option value="Guest">"Guest"</option>
                                                            </select>
                                                            <button class="login-btn"
                                                                on:click=move |_| on_add_member(oid_member)>
                                                                "Add Member"
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else { ().into_any() }
                                            })}

                                            {if org_members.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <div class="empty-text">"No members"</div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="asset-list">
                                                        {org_members.into_iter().map(|user| {
                                                            let _uid = user.id;
                                                            let uid_del = user.id;
                                                            let uid_role = user.id;
                                                            let role_str = format!("{:?}", user.role);
                                                            view! {
                                                                <div class="list-item">
                                                                    <div class="list-item-left">
                                                                        <div class="list-item-title">{user.name.clone()}</div>
                                                                        <div class="list-item-subtitle">{user.email.clone()}</div>
                                                                    </div>
                                                                    <div class="list-item-right">
                                                                        {if can {
                                                                            let uid_r = uid_role;
                                                                            view! {
                                                                                <select class="login-input"
                                                                                    style="width: auto; height: 32px; font-size: 12px;"
                                                                                    on:change=move |ev| {
                                                                                        let v = event_target_value(&ev);
                                                                                        let r = match v.as_str() {
                                                                                            "Owner" => UserRole::Owner,
                                                                                            "Manager" => UserRole::Manager,
                                                                                            "Worker" => UserRole::Worker,
                                                                                            _ => UserRole::Guest,
                                                                                        };
                                                                                        on_update_member_role(uid_r, r);
                                                                                    }
                                                                                >
                                                                                    <option value="Worker" selected={user.role == UserRole::Worker}>"Worker"</option>
                                                                                    <option value="Manager" selected={user.role == UserRole::Manager}>"Manager"</option>
                                                                                    <option value="Owner" selected={user.role == UserRole::Owner}>"Owner"</option>
                                                                                    <option value="Guest" selected={user.role == UserRole::Guest}>"Guest"</option>
                                                                                </select>
                                                                                <button class="card-btn sell"
                                                                                    on:click=move |_| on_remove_member(oid_member, uid_del)>
                                                                                    "Remove"
                                                                                </button>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! {
                                                                                <span class="stat-value">{role_str}</span>
                                                                            }.into_any()
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            }.into_any()
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
