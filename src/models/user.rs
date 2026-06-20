use crate::types::{
    Currency, NotificationTrigger, NotificationType, OrganizationSettings, PaymentInterval,
    PaymentMethod, UserRole,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User/Organization member
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub hire_date: Option<DateTime<Utc>>,
    pub base_salary: Option<f64>,
    pub payment_settings: PaymentSettings,
    pub notification_preferences: Vec<(NotificationTrigger, Vec<NotificationType>)>,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentSettings {
    pub payment_method: PaymentMethod,
    pub account_details: String,
    pub payment_interval: PaymentInterval,
    pub currency: Currency,
    pub automatic_payout: bool,
    pub payout_threshold: Option<f64>,
}

impl Default for PaymentSettings {
    fn default() -> Self {
        Self {
            payment_method: PaymentMethod::BankTransfer,
            account_details: String::new(),
            payment_interval: PaymentInterval::Monthly,
            currency: Currency::USD,
            automatic_payout: true,
            payout_threshold: None,
        }
    }
}

impl User {
    pub fn new(name: String, email: String, role: UserRole) -> Self {
        let now = Utc::now();
        let permissions = default_permissions_for_role(&role);
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            role,
            organization_id: None,
            department: None,
            phone: None,
            address: None,
            hire_date: None,
            base_salary: None,
            payment_settings: PaymentSettings::default(),
            notification_preferences: Vec::new(),
            permissions,
            created_at: now,
            updated_at: now,
            last_login: None,
            is_active: true,
        }
    }

    pub fn can_manage(&self, other: &User) -> bool {
        // Must be in same organization
        if self.organization_id != other.organization_id {
            return false;
        }
        self.role.can_manage(&other.role)
    }

    pub fn update_role(&mut self, new_role: UserRole, changed_by: &User) -> Result<(), String> {
        if !changed_by.can_manage(self) {
            return Err("Insufficient permissions to change role".to_string());
        }
        self.role = new_role;
        self.permissions = default_permissions_for_role(&self.role);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn toggle_permission(&mut self, permission: Permission) {
        if self.permissions.contains(&permission) {
            self.permissions.retain(|p| p != &permission);
        } else {
            self.permissions.push(permission);
        }
        self.updated_at = Utc::now();
    }
}

pub fn default_permissions_for_role(role: &UserRole) -> Vec<Permission> {
    match role {
        UserRole::Owner => vec![
            Permission::ViewAll,
            Permission::CreateOrganization,
            Permission::EditAll,
            Permission::DeleteAll,
            Permission::ManageUsers,
            Permission::ManageRoles,
            Permission::ManagePayments,
            Permission::ManageSettings,
            Permission::ExportData,
            Permission::ImportData,
        ],
        UserRole::Manager => vec![
            Permission::ViewOrganization,
            Permission::CreateOrganization,
            Permission::EditOrganization,
            Permission::DeleteOrganization,
            Permission::ManageUsers,
            Permission::ManagePayments,
        ],
        UserRole::Worker => vec![
            Permission::ViewOwn,
            Permission::CreateOwn,
            Permission::EditOwn,
            Permission::DeleteOwn,
        ],
        UserRole::Guest => vec![Permission::ViewOwn],
    }
}

// Organization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub members: Vec<Uuid>,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn new(name: String, owner_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            owner_id,
            members: vec![owner_id],
            settings: OrganizationSettings::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_member(&mut self, user_id: Uuid) {
        if !self.members.contains(&user_id) {
            self.members.push(user_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_member(&mut self, user_id: Uuid) {
        self.members.retain(|&id| id != user_id);
        self.updated_at = Utc::now();
    }
}

// Payment/Transaction record
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: f64,
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub description: Option<String>,
    pub related_asset_id: Option<Uuid>,
    pub related_portfolio_id: Option<Uuid>,
    pub status: PaymentStatus,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub executed_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Scheduled,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Payment {
    pub fn new(
        from_user_id: Uuid,
        to_user_id: Uuid,
        amount: f64,
        currency: Currency,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            amount,
            currency,
            payment_method,
            description: None,
            related_asset_id: None,
            related_portfolio_id: None,
            status: PaymentStatus::Pending,
            scheduled_date: None,
            executed_date: None,
            created_at: Utc::now(),
            is_recurring: false,
            recurrence_rule: None,
        }
    }

    pub fn schedule(&mut self, date: DateTime<Utc>) {
        self.scheduled_date = Some(date);
        self.status = PaymentStatus::Scheduled;
    }

    pub fn mark_completed(&mut self) {
        self.status = PaymentStatus::Completed;
        self.executed_date = Some(Utc::now());
    }
}

// Role definition and permissions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: Uuid,
    pub name: String,
    pub role_type: UserRole,
    pub permissions: Vec<Permission>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ViewOwn,
    ViewOrganization,
    ViewAll,
    CreateOwn,
    CreateOrganization,
    EditOwn,
    EditOrganization,
    EditAll,
    DeleteOwn,
    DeleteOrganization,
    DeleteAll,
    ManageUsers,
    ManageRoles,
    ManagePayments,
    ManageSettings,
    ExportData,
    ImportData,
    Custom(String),
}
