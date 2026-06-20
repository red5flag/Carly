use crate::types::{Currency, TransactionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Transaction record for financial tracking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: Currency,
    pub description: Option<String>,
    pub from_entity: EntityReference,
    pub to_entity: EntityReference,
    pub related_portfolio_id: Option<Uuid>,
    pub related_asset_group_id: Option<Uuid>,
    pub related_asset_id: Option<Uuid>,
    pub executed_by: Uuid,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    User,
    Organization,
    Portfolio,
    AssetGroup,
    Asset,
    External,
    System,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        amount: f64,
        currency: Currency,
        from_entity: EntityReference,
        to_entity: EntityReference,
        executed_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            currency,
            description: None,
            from_entity,
            to_entity,
            related_portfolio_id: None,
            related_asset_group_id: None,
            related_asset_id: None,
            executed_by,
            status: TransactionStatus::Draft,
            created_at: Utc::now(),
            executed_at: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn approve(&mut self) {
        self.status = TransactionStatus::Approved;
    }

    pub fn reject(&mut self, reason: Option<String>) {
        self.status = TransactionStatus::Rejected;
        if let Some(r) = reason {
            self.metadata["rejection_reason"] = serde_json::json!(r);
        }
    }

    pub fn execute(&mut self) {
        self.status = TransactionStatus::Executed;
        self.executed_at = Some(Utc::now());
    }
}

// Transaction summary for reports
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_transactions: u64,
    pub total_inflow: f64,
    pub total_outflow: f64,
    pub net_flow: f64,
    pub by_type: Vec<(TransactionType, f64, u64)>,
    pub by_currency: Vec<(Currency, f64)>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

impl TransactionSummary {
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            total_transactions: 0,
            total_inflow: 0.0,
            total_outflow: 0.0,
            net_flow: 0.0,
            by_type: Vec::new(),
            by_currency: Vec::new(),
            period_start,
            period_end,
        }
    }
}

// Quick sale request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuickSaleRequest {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub asset_group_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub requested_price: f64,
    pub urgency: SaleUrgency,
    pub notes: Option<String>,
    pub requested_by: Uuid,
    pub status: QuickSaleStatus,
    pub created_at: DateTime<Utc>,
    pub notifications_sent: Vec<(NotificationChannel, DateTime<Utc>)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaleUrgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickSaleStatus {
    Draft,
    Submitted,
    UnderReview,
    Approved,
    Listed,
    Sold,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Push,
    Sms,
    InApp,
}

impl QuickSaleRequest {
    pub fn new(requested_by: Uuid, requested_price: f64, urgency: SaleUrgency) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: None,
            asset_group_id: None,
            portfolio_id: None,
            requested_price,
            urgency,
            notes: None,
            requested_by,
            status: QuickSaleStatus::Draft,
            created_at: Utc::now(),
            notifications_sent: Vec::new(),
        }
    }

    pub fn submit(&mut self) {
        self.status = QuickSaleStatus::Submitted;
    }

    pub fn record_notification(&mut self, channel: NotificationChannel) {
        self.notifications_sent.push((channel, Utc::now()));
    }
}
