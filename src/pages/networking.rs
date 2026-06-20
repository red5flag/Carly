use crate::models::{default_permissions_for_role, Payment, PaymentSettings, PaymentStatus, Permission, User};
use crate::stores::use_app_store;
use crate::types::{PaymentInterval, PaymentMethod, UserRole};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn NetworkingPage() -> impl IntoView {
    let app_store = use_app_store();

    // Users come from app store, fall back to default mock users if empty
    let users = Memo::new(move |_| {
        let store_users = app_store.get().organization_users.clone();
        if store_users.is_empty() {
            default_mock_users()
        } else {
            store_users
        }
    });

    // New user form state
    let (new_name, set_new_name) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_role, set_new_role) = signal(UserRole::Worker);

    let on_add_user = move |_| {
        let name = new_name.get().trim().to_string();
        let email = new_email.get().trim().to_string();
        if name.is_empty() || email.is_empty() {
            return;
        }
        let mut user = User::new(name, email, new_role.get());
        user.organization_id = app_store.get().current_user.organization_id;
        app_store.update(|s| s.add_organization_user(user));
        set_new_name.set(String::new());
        set_new_email.set(String::new());
        set_new_role.set(UserRole::Worker);
    };

    let on_update_role = move |id: Uuid, role: UserRole| {
        app_store.update(|s| {
            let _ = s.update_user_role(id, role);
        });
    };

    let on_remove_user = move |id: Uuid| {
        app_store.update(|s| {
            s.remove_organization_user(id);
        });
    };

    let on_toggle_permission = move |id: Uuid, permission: Permission| {
        app_store.update(|s| {
            s.toggle_user_permission(id, permission);
        });
    };

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

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Networking"</h1>
                <p>"Organization members and payments"</p>
            </div>

            // Organization Stats
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Organization Overview"</span>
                </div>
                <div class="card-stats">
                    <div class="stat-item">
                        <div class="stat-label">"Total Members"</div>
                        <div class="stat-value">{move || users.get().len()}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Active Now"</div>
                        <div class="stat-value">"2"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Pending Payments"</div>
                        <div class="stat-value">
                            {move || {
                                transactions.get()
                                    .iter()
                                    .filter(|t| t.status == PaymentStatus::Pending)
                                    .count()
                            }}
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">"Total Payouts"</div>
                        <div class="stat-value">
                            {move || {
                                let total: f64 = transactions.get()
                                    .iter()
                                    .filter(|t| t.status == PaymentStatus::Completed)
                                    .map(|t| t.amount)
                                    .sum();
                                format!("${:.0}K", total / 1000.0)
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Add User Form
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
                                "Manager" => UserRole::Manager,
                                "Worker" => UserRole::Worker,
                                _ => UserRole::Guest,
                            });
                        }
                    >
                        <option value="Owner">"Owner"</option>
                        <option value="Manager">"Manager"</option>
                        <option value="Worker">"Worker"</option>
                        <option value="Guest">"Guest"</option>
                    </select>
                </div>
                <button class="card-btn" on:click=on_add_user>"Add Member"</button>
            </div>

            // Users List with Role Management
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Team Members & Roles"</span>
                </div>
                <div class="data-list">
                    {move || {
                        users.get()
                            .into_iter()
                            .map(|user| {
                                let role_icon = match user.role {
                                    UserRole::Owner => "👑",
                                    UserRole::Manager => "⭐",
                                    UserRole::Worker => "👤",
                                    UserRole::Guest => "🔒",
                                };
                                view! {
                                    <div class="list-item role-management-item">
                                        <div class="list-item-left">
                                            <div class="list-item-title">
                                                {format!("{} {}", role_icon, user.name)}
                                            </div>
                                            <div class="list-item-subtitle">
                                                {user.email.clone()}
                                            </div>
                                            <div class="permission-list">
                                                {[
                                                    (Permission::ViewOrganization, "View"),
                                                    (Permission::EditOrganization, "Edit"),
                                                    (Permission::ManageUsers, "Users"),
                                                    (Permission::ManagePayments, "Payments"),
                                                ].into_iter().map(|(permission, label)| {
                                                    let has = user.has_permission(&permission);
                                                    let user_id = user.id;
                                                    view! {
                                                        <label class="permission-tag" class:active=has>
                                                            <input
                                                                type="checkbox"
                                                                checked=has
                                                                on:change=move |_| on_toggle_permission(user_id, permission.clone())
                                                            />
                                                            {label}
                                                        </label>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                        <div class="list-item-right role-actions">
                                            <select
                                                class="role-select"
                                                prop:value={move || format!("{:?}", user.role)}
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    let role = match value.as_str() {
                                                        "Owner" => UserRole::Owner,
                                                        "Manager" => UserRole::Manager,
                                                        "Worker" => UserRole::Worker,
                                                        _ => UserRole::Guest,
                                                    };
                                                    on_update_role(user.id, role);
                                                }
                                            >
                                                <option value="Owner">"Owner"</option>
                                                <option value="Manager">"Manager"</option>
                                                <option value="Worker">"Worker"</option>
                                                <option value="Guest">"Guest"</option>
                                            </select>
                                            <button
                                                class="card-btn danger"
                                                on:click=move |_| on_remove_user(user.id)
                                            >
                                                "Remove"
                                            </button>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Payment History
            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Payments"</span>
                </div>
                <div class="data-list">
                    {move || {
                        transactions.get()
                            .into_iter()
                            .map(|payment| {
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
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
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
            email: "john@company.com".to_string(),
            role: UserRole::Owner,
            organization_id: Some(org_id),
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
            email: "sarah@company.com".to_string(),
            role: UserRole::Manager,
            organization_id: Some(org_id),
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
            email: "mike@company.com".to_string(),
            role: UserRole::Worker,
            organization_id: Some(org_id),
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
            email: "guest@company.com".to_string(),
            role: UserRole::Guest,
            organization_id: Some(org_id),
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
