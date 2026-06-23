use crate::models::{default_permissions_for_role, Payment, PaymentSettings, PaymentStatus, Permission, User, UserActivity, UserAssignment};
use crate::stores::use_app_store;
use crate::types::{PaymentInterval, PaymentMethod, UserRole};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let app_store = use_app_store();

    // Users come from app store, fall back to default mock users if empty, sorted by role
    let users = Memo::new(move |_| {
        let store = app_store.get();
        let store_users = store.organization_users.clone();
        let current_org = store.current_organization_id.or(store.current_user.organization_id);
        let mut users: Vec<_> = if store_users.is_empty() {
            default_mock_users()
        } else {
            store_users
        }
        .into_iter()
        .filter(|u| current_org.is_none() || u.organization_id == current_org)
        .collect();
        users.sort_by(|a, b| b.role.level().cmp(&a.role.level()));
        users
    });

    let _on_update_role = move |id: Uuid, role: UserRole| {
        app_store.update(|s| {
            let _ = s.update_user_role(id, role);
        });
    };

    let _on_remove_user = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_organization_user(id);
        });
    };

    let _on_toggle_permission = move |id: Uuid, permission: Permission| {
        app_store.update(|s| {
            s.toggle_user_permission(id, permission);
        });
    };

    let (edit_mode, set_edit_mode) = signal(false);
    let (_people_open, _set_people_open) = signal(false);
    let (active_tab, set_active_tab) = signal("organizations");
    let (member_view, set_member_view) = signal("list");

    // Mock transactions
    let transactions = Memo::new(move |_| {
        vec![
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 5000.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Monthly salary payment".to_string()),
                related_asset_id: None,
                related_portfolio_id: None,
                status: PaymentStatus::Completed,
                scheduled_date: None,
                executed_date: Some(Utc::now()),
                created_at: Utc::now(),
                is_recurring: true,
                recurrence_rule: Some("monthly".to_string()),
            },
            Payment {
                id: Uuid::new_v4(),
                from_user_id: Uuid::new_v4(),
                to_user_id: Uuid::new_v4(),
                amount: 2500.0,
                currency: crate::types::Currency::USD,
                payment_method: PaymentMethod::BankTransfer,
                description: Some("Asset performance bonus".to_string()),
                related_asset_id: Some(Uuid::new_v4()),
                related_portfolio_id: None,
                status: PaymentStatus::Pending,
                scheduled_date: Some(Utc::now()),
                executed_date: None,
                created_at: Utc::now(),
                is_recurring: false,
                recurrence_rule: None,
            },
        ]
    });

    let render_tab_content = move || {
        let tab = active_tab.get();
        if tab == "organizations" {
            render_organizations(users).into_any()
        } else if tab == "members" {
            render_members(users, member_view, set_member_view).into_any()
        } else if tab == "activity" {
            render_activity().into_any()
        } else {
            render_payments(transactions).into_any()
        }
    };

    view! {
        <div class="home-screen">
            // Quick tabs
            <div class="net-quick-tabs">
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "organizations"}
                    on:click=move |_| set_active_tab.set("organizations")
                >
                    "Organizations"
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "members"}
                    on:click=move |_| set_active_tab.set("members")
                >
                    {move || format!("Members {}", users.get().len())}
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "activity"}
                    on:click=move |_| set_active_tab.set("activity")
                >
                    "Activity"
                </button>
                <button
                    class="net-quick-tab"
                    class:active={move || active_tab.get() == "payments"}
                    on:click=move |_| set_active_tab.set("payments")
                >
                    "Payments"
                </button>
            </div>

            // Organization Stats
            <div class="org-metrics-bar">
                <div class="org-metric">
                    <div class="org-metric-value">{move || app_store.get().organizations.len()}</div>
                    <div class="org-metric-label">"Organizations"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">{move || users.get().len()}</div>
                    <div class="org-metric-label">"Members"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">
                        {move || {
                            transactions.get()
                                .iter()
                                .filter(|t| t.status == PaymentStatus::Pending)
                                .count()
                        }}
                    </div>
                    <div class="org-metric-label">"Pending"</div>
                </div>
                <div class="org-metric">
                    <div class="org-metric-value">
                        {move || {
                            let total: f64 = transactions.get()
                                .iter()
                                .filter(|t| t.status == PaymentStatus::Completed)
                                .map(|t| t.amount)
                                .sum();
                            format!("${:.0}K", total / 1000.0)
                        }}
                    </div>
                    <div class="org-metric-label">"Payouts"</div>
                </div>
            </div>

            // Edit action bar
            <div class="net-action-bar">
                <button
                    class="net-action-btn"
                    class:active={move || edit_mode.get()}
                    on:click=move |_| set_edit_mode.update(|v| *v = !*v)
                >
                    "Edit"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| app_store.update(|s| s.toggle_networking_add_member())
                >
                    "Add"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| {
                        if let Some(first) = users.get().first() {
                            app_store.update(|s| { let _ = s.remove_organization_user(first.id); });
                        }
                    }
                >
                    "Remove"
                </button>
                <button
                    class="net-action-btn"
                    on:click=move |_| app_store.update(|s| s.open_search())
                >
                    "Search"
                </button>
            </div>

            // Tab content
            {render_tab_content}
        </div>
    }
}

fn render_organizations(_users: Memo<Vec<User>>) -> impl IntoView {
    let app_store = use_app_store();
    view! {
        <div class="net-tab-content">
            {move || {
                let store = app_store.get();
                let orgs = store.organizations.clone();
                let all_users = if store.organization_users.is_empty() {
                    default_mock_users()
                } else {
                    store.organization_users.clone()
                };
                if orgs.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state">
                                <div class="empty-text">"No organizations yet"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    orgs.into_iter().map(|org| {
                        let mut org_users: Vec<User> = all_users.iter()
                            .filter(|u| u.organization_id == Some(org.id))
                            .cloned()
                            .collect();
                        org_users.sort_by(|a, b| b.role.level().cmp(&a.role.level()));
                        view! {
                            <div class="data-card">
                                <div class="card-header">
                                    <span class="card-title">{org.name.clone()}</span>
                                    <span class="stat-value">{format!("{} members", org_users.len())}</span>
                                </div>
                                <div class="net-org-members">
                                    {org_users.into_iter().map(|user| view! {
                                        <UserCard user={user} />
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>().into_any()
                }
            }}
        </div>
    }
}

fn render_members(
    users: Memo<Vec<User>>,
    member_view: ReadSignal<&'static str>,
    set_member_view: WriteSignal<&'static str>,
) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="view-toggle">
                <button
                    class="view-btn"
                    class:active={move || member_view.get() == "list"}
                    on:click=move |_| set_member_view.set("list")
                >
                    "List"
                </button>
                <button
                    class="view-btn"
                    class:active={move || member_view.get() == "grid"}
                    on:click=move |_| set_member_view.set("grid")
                >
                    "Grid"
                </button>
            </div>
            <div class={move || if member_view.get() == "grid" { "net-members-grid" } else { "net-members-list" }}>
                {move || {
                    users.get().into_iter().map(|user| view! {
                        <UserCard user={user} />
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

fn render_activity() -> impl IntoView {
    let app_store = use_app_store();
    view! {
        <div class="net-tab-content">
            {move || {
                let store = app_store.get();
                let all_users = if store.organization_users.is_empty() {
                    default_mock_users()
                } else {
                    store.organization_users.clone()
                };
                let activities: Vec<_> = all_users.iter()
                    .flat_map(|u| u.activity_log.iter().map(move |a| (u.clone(), a.clone())))
                    .collect();
                if activities.is_empty() {
                    view! {
                        <div class="data-card">
                            <div class="empty-state">
                                <div class="empty-text">"No activity yet"</div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    activities.into_iter().map(|(user, activity)| {
                        view! {
                            <div class="list-item">
                                <div class="list-item-left">
                                    <div class="list-item-title">{format!("{} {}", user.name, activity.action)}</div>
                                    <div class="list-item-subtitle">{format!("{}: {} - {}", activity.target_type, activity.target_name, activity.reason.as_deref().unwrap_or(""))}</div>
                                </div>
                                <div class="list-item-right">
                                    <div class="stat-value">{activity.timestamp.format("%Y-%m-%d").to_string()}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>().into_any()
                }
            }}
        </div>
    }
}

fn render_payments(transactions: Memo<Vec<Payment>>) -> impl IntoView {
    view! {
        <div class="net-tab-content">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Payments"</span>
                </div>
                <div class="data-list">
                    {move || {
                        transactions.get().into_iter().map(|payment| {
                            let status_class = match payment.status {
                                PaymentStatus::Completed => "positive",
                                PaymentStatus::Pending => "",
                                PaymentStatus::Failed => "negative",
                                _ => "",
                            };
                            let status_icon = match payment.status {
                                PaymentStatus::Completed => "✓",
                                PaymentStatus::Pending => "⏳",
                                PaymentStatus::Scheduled => "📅",
                                PaymentStatus::Processing => "⚙️",
                                PaymentStatus::Failed => "✗",
                                PaymentStatus::Cancelled => "⊘",
                            };
                            view! {
                                <div class="list-item">
                                    <div class="list-item-left">
                                        <div class="list-item-title">
                                            {format!("{} {}", status_icon,
                                                payment.description.as_deref().unwrap_or("Payment")
                                            )}
                                        </div>
                                        <div class="list-item-subtitle">
                                            {format!("{:?} - {:?}",
                                                payment.payment_method,
                                                payment.status
                                            )}
                                        </div>
                                    </div>
                                    <div class="list-item-right">
                                        <div class={format!("list-item-value {}", status_class)}>
                                            {format!("${:.0}", payment.amount)}
                                        </div>
                                        {payment.is_recurring.then(|| {
                                            view! {
                                                <div style="font-size: 10px; color: var(--text-secondary);">
                                                    "🔄 Recurring"
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn UserCard(user: User) -> impl IntoView {
    let avatar = user.avatar_url.clone().unwrap_or_else(|| {
        format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", user.id)
    });
    let role_label = format!("{:?}", user.role);
    let username = user.username.clone().unwrap_or_else(|| user.name.clone());
    let assignment = user.assignments.first().cloned();

    view! {
        <div class="net-user-card">
            <img class="net-user-avatar" src={avatar} alt={user.name.clone()} />
            <div class="net-user-info">
                <div class="net-user-name">{user.name.clone()}</div>
                <div class="net-user-meta">{format!("@{} • {}", username, user.email)}</div>
                <div class="net-user-role">{role_label}</div>
                {if let Some(a) = assignment {
                    view! {
                        <div class="net-user-assignment">
                            {format!("Assigned to {}: {}", a.target_type, a.target_name)}
                            {a.duration_days.map(|d| format!(" • {} days", d)).unwrap_or_default()}
                            {a.reason.map(|r| format!(" • {}", r)).unwrap_or_default()}
                        </div>
                    }.into_any()
                } else { ().into_any() }}
            </div>
        </div>
    }
}

#[component]
pub fn AddTeamMemberPage() -> impl IntoView {
    let app_store = use_app_store();

    let (new_name, set_new_name) = signal(String::new());
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal(UserRole::Worker);

    let on_add_user = move |_| {
        let name = new_name.get().trim().to_string();
        let email = new_email.get().trim().to_string();
        if name.is_empty() || email.is_empty() {
            return;
        }
        let username = new_username.get().trim().to_string();
        let avatar = if username.is_empty() {
            format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", name)
        } else {
            format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", username)
        };
        let mut user = User::new(name, email, new_role.get());
        user.username = if username.is_empty() { None } else { Some(username) };
        user.avatar_url = Some(avatar);
        let store = app_store.get();
        user.organization_id = store.current_organization_id.or(store.current_user.organization_id);
        drop(store);
        app_store.update(|s| s.add_organization_user(user));
        set_new_name.set(String::new());
        set_new_username.set(String::new());
        set_new_email.set(String::new());
        set_new_role.set(UserRole::Worker);
    };

    view! {
        <div class="home-screen">
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Add Team Member"</span>
                </div>
                <div class="form-group">
                    <label class="form-label">"Name"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Full name"
                        prop:value=new_name
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Username"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="Username"
                        prop:value=new_username
                        on:input=move |ev| set_new_username.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Email"</label>
                    <input
                        class="form-input"
                        type="email"
                        placeholder="Email address"
                        prop:value=new_email
                        on:input=move |ev| set_new_email.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">"Role"</label>
                    <select
                        class="form-select"
                        prop:value={move || format!("{:?}", new_role.get())}
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_new_role.set(match value.as_str() {
                                "Owner" => UserRole::Owner,
                                "Director" => UserRole::Director,
                                "SeniorManager" => UserRole::SeniorManager,
                                "Manager" => UserRole::Manager,
                                "Worker" => UserRole::Worker,
                                "Contractor" => UserRole::Contractor,
                                _ => UserRole::Guest,
                            });
                        }
                    >
                        <option value="Owner">"Owner"</option>
                        <option value="Director">"Director"</option>
                        <option value="SeniorManager">"Senior Manager"</option>
                        <option value="Manager">"Manager"</option>
                        <option value="Worker">"Worker"</option>
                        <option value="Contractor">"Contractor"</option>
                        <option value="Guest">"Guest"</option>
                    </select>
                </div>
                <button class="card-btn" on:click=on_add_user>"Add Member"</button>
            </div>
        </div>
    }
}

fn default_mock_users() -> Vec<User> {
    let org_id = Uuid::new_v4();
    vec![
        User {
            id: Uuid::new_v4(),
            name: "John Smith".to_string(),
            username: Some("jsmith".to_string()),
            email: "john@company.com".to_string(),
            role: UserRole::Owner,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=John".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Portfolio".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Downtown Properties".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(365),
                reason: Some("Property oversight".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Created".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Downtown Office".to_string(),
                timestamp: Utc::now(),
                reason: Some("New acquisition".to_string()),
            }],
            department: Some("Executive".to_string()),
            phone: Some("+1-555-0100".to_string()),
            address: Some("123 Main St".to_string()),
            hire_date: Some(Utc::now()),
            base_salary: Some(200000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****1234".to_string(),
                payment_interval: PaymentInterval::Monthly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Owner),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Sarah Johnson".to_string(),
            username: Some("sjohnson".to_string()),
            email: "sarah@company.com".to_string(),
            role: UserRole::Manager,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Sarah".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset Group".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Fleet Vehicles".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(180),
                reason: Some("Fleet coordinator".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Modified".to_string(),
                target_type: "Asset".to_string(),
                target_name: "Fleet Van #3".to_string(),
                timestamp: Utc::now(),
                reason: Some("Value update".to_string()),
            }],
            department: Some("Operations".to_string()),
            phone: Some("+1-555-0101".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(120000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::DirectDeposit,
                account_details: "****5678".to_string(),
                payment_interval: PaymentInterval::BiWeekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Manager),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Mike Williams".to_string(),
            username: Some("mwilliams".to_string()),
            email: "mike@company.com".to_string(),
            role: UserRole::Worker,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Mike".to_string()),
            assignments: vec![UserAssignment {
                target_type: "Asset".to_string(),
                target_id: Uuid::new_v4(),
                target_name: "Warehouse A".to_string(),
                assigned_at: Utc::now(),
                duration_days: Some(90),
                reason: Some("Maintenance rotation".to_string()),
            }],
            activity_log: vec![UserActivity {
                action: "Updated".to_string(),
                target_type: "Task".to_string(),
                target_name: "Roof repair".to_string(),
                timestamp: Utc::now(),
                reason: Some("Routine maintenance".to_string()),
            }],
            department: Some("Field Operations".to_string()),
            phone: Some("+1-555-0102".to_string()),
            address: None,
            hire_date: Some(Utc::now()),
            base_salary: Some(65000.0),
            payment_settings: PaymentSettings {
                payment_method: PaymentMethod::BankTransfer,
                account_details: "****9012".to_string(),
                payment_interval: PaymentInterval::Weekly,
                currency: crate::types::Currency::USD,
                automatic_payout: true,
                payout_threshold: None,
            },
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Worker),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: Some(Utc::now()),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Guest User".to_string(),
            username: Some("guest".to_string()),
            email: "guest@company.com".to_string(),
            role: UserRole::Guest,
            organization_id: Some(org_id),
            avatar_url: Some("https://api.dicebear.com/7.x/avataaars/svg?seed=Guest".to_string()),
            assignments: vec![],
            activity_log: vec![],
            department: Some("External".to_string()),
            phone: None,
            address: None,
            hire_date: None,
            base_salary: None,
            payment_settings: PaymentSettings::default(),
            notification_preferences: vec![],
            permissions: default_permissions_for_role(&UserRole::Guest),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            is_active: true,
        },
    ]
}
