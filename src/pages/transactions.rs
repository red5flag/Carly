use crate::models::{EntityReference, EntityType, Transaction, TransactionStatus};
use crate::stores::use_app_store;
use crate::types::{Currency, TransactionType};
use chrono::Utc;
use leptos::prelude::*;
use uuid::Uuid;

fn create_mock_transaction(
    transaction_type: TransactionType,
    amount: f64,
    description: &str,
    from: &str,
    to: &str,
    status: TransactionStatus,
) -> Transaction {
    Transaction {
        id: Uuid::new_v4(),
        transaction_type,
        amount,
        currency: Currency::USD,
        description: Some(description.to_string()),
        from_entity: EntityReference {
            entity_type: EntityType::Organization,
            entity_id: Uuid::new_v4(),
            name: from.to_string(),
        },
        to_entity: EntityReference {
            entity_type: EntityType::External,
            entity_id: Uuid::new_v4(),
            name: to.to_string(),
        },
        related_portfolio_id: None,
        related_asset_group_id: None,
        related_asset_id: None,
        executed_by: Uuid::new_v4(),
        status,
        created_at: Utc::now(),
        executed_at: Some(Utc::now()),
        metadata: serde_json::json!({}),
    }
}

fn status_label(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Draft => "Draft",
        TransactionStatus::Pending => "Pending",
        TransactionStatus::Approved => "Approved",
        TransactionStatus::Rejected => "Rejected",
        TransactionStatus::Executed => "Executed",
        TransactionStatus::Cancelled => "Cancelled",
    }
}

fn type_icon(transaction_type: &TransactionType) -> &'static str {
    match transaction_type {
        TransactionType::Purchase => "🛒",
        TransactionType::Sale => "💰",
        TransactionType::Rent => "🏠",
        TransactionType::Lease => "📄",
        TransactionType::Payout => "💵",
        TransactionType::Dividend => "📈",
        TransactionType::Fee => "⚠",
        TransactionType::Tax => "🏛",
        TransactionType::Transfer => "🔄",
        TransactionType::Adjustment => "🔧",
    }
}

#[component]
pub fn TransactionsPage() -> impl IntoView {
    let _app_store = use_app_store();

    let transactions = Memo::new(move |_| {
        vec![
            create_mock_transaction(
                TransactionType::Purchase,
                125000.0,
                "Office equipment purchase",
                "Main Org",
                "Tech Supplies Inc",
                TransactionStatus::Executed,
            ),
            create_mock_transaction(
                TransactionType::Sale,
                450000.0,
                "Property sale - downtown plaza",
                "Real Estate Holdings",
                "Buyer Corp",
                TransactionStatus::Approved,
            ),
            create_mock_transaction(
                TransactionType::Rent,
                8500.0,
                "Monthly warehouse rent",
                "Tenant LLC",
                "Property Manager",
                TransactionStatus::Executed,
            ),
            create_mock_transaction(
                TransactionType::Fee,
                1200.0,
                "Bank processing fee",
                "Main Org",
                "Banking Partner",
                TransactionStatus::Executed,
            ),
            create_mock_transaction(
                TransactionType::Transfer,
                50000.0,
                "Inter-portfolio transfer",
                "Portfolio A",
                "Portfolio B",
                TransactionStatus::Pending,
            ),
        ]
    });

    let total_volume = move || {
        transactions
            .get()
            .iter()
            .filter(|t| t.status == TransactionStatus::Executed || t.status == TransactionStatus::Approved)
            .map(|t| t.amount)
            .sum::<f64>()
    };

    view! {
        <div class="home-screen">
            <div class="welcome-header">
                <h1>"Transactions"</h1>
                <p>{move || format!("Total volume: ${:.2}", total_volume())}</p>
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Recent Transactions"</span>
                </div>
                {move || {
                    transactions
                        .get()
                        .into_iter()
                        .map(|transaction| {
                            let icon = type_icon(&transaction.transaction_type);
                            let status = status_label(&transaction.status);
                            let amount = format!("${:.2}", transaction.amount);
                            let description = transaction.description.unwrap_or_default();
                            let from = transaction.from_entity.name;
                            let to = transaction.to_entity.name;
                            view! {
                                <div class="list-item">
                                    <div class="list-item-left">
                                        <div class="list-item-title">{icon} " " {description}</div>
                                        <div class="list-item-subtitle">{from} " → " {to}</div>
                                    </div>
                                    <div class="list-item-right">
                                        <div class="list-item-value">{amount}</div>
                                        <div class="list-item-subtitle">{status}</div>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            <div class="data-card">
                <div class="card-header">
                    <span class="card-title">"Transaction Summary"</span>
                </div>
                <div class="card-stats">
                    <div class="stat-item">
                        <div class="stat-value">"5"</div>
                        <div class="stat-label">"Total"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-value">"3"</div>
                        <div class="stat-label">"Executed"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-value">"1"</div>
                        <div class="stat-label">"Pending"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-value">"1"</div>
                        <div class="stat-label">"Approved"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}
