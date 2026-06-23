use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User roles for organization hierarchy: Owner > Director > SeniorManager > Manager > Worker > Contractor > Guest
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Owner,
    Director,
    SeniorManager,
    Manager,
    Worker,
    Contractor,
    Guest,
}

impl UserRole {
    pub fn level(&self) -> u8 {
        match self {
            UserRole::Owner => 6,
            UserRole::Director => 5,
            UserRole::SeniorManager => 4,
            UserRole::Manager => 3,
            UserRole::Worker => 2,
            UserRole::Contractor => 1,
            UserRole::Guest => 0,
        }
    }

    pub fn can_manage(&self, other: &UserRole) -> bool {
        self.level() > other.level()
    }

    pub fn can_equal_or_manage(&self, other: &UserRole) -> bool {
        self.level() >= other.level()
    }
}

// Currency types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
    CNY,
    Custom(String),
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

// Payment intervals
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentInterval {
    Hourly,
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Annually,
    OneTime,
    Custom(String),
}

impl Default for PaymentInterval {
    fn default() -> Self {
        PaymentInterval::Monthly
    }
}

// Payment methods
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    BankTransfer,
    CreditCard,
    DebitCard,
    DirectDeposit,
    PayPal,
    Crypto,
    Cash,
    Check,
    Custom(String),
}

// Notification types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Push,
    Email,
    Sms,
    InApp,
}

// Notification triggers
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationTrigger {
    PriceChange { percentage: f64 },
    Sale,
    Auction,
    Rent,
    Unrent,
    NoSales { days: u32 },
    Custom(String),
}

// Asset types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    RealEstate,
    Vehicle,
    Equipment,
    Stock,
    Bond,
    Commodity,
    Digital,
    IntellectualProperty,
    Custom(String),
}

// Transaction types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Purchase,
    Sale,
    Rent,
    Lease,
    Payout,
    Dividend,
    Fee,
    Tax,
    Transfer,
    Adjustment,
}

// View modes for display
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    List,
    Grid,
    Chart,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::List
    }
}

// Sort modes for lists
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortMode {
    #[default]
    Recent,
    Oldest,
    HighestValue,
    LowestValue,
    HighestProfit,
    LowestProfit,
    HighestRevenue,
    LowestRevenue,
    ByOrganization,
}

// Search filters
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub asset_types: Vec<AssetType>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub value_range: Option<(f64, f64)>,
    pub tags: Vec<String>,
    pub status: Option<String>,
    pub owner: Option<Uuid>,
    pub portfolio: Option<Uuid>,
}

// Action types for undo/redo system
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Create,
    Update,
    Delete,
    View,
    Navigate,
    Setting,
    Payment,
    Notification,
    Search,
    Undo,
    Redo,
    Login,
    Logout,
}

// Theme/accessibility options
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Default,
    HighContrast,
    Deuteranopia, // Green-blind
    Protanopia,   // Red-blind
    Tritanopia,   // Blue-blind
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Default
    }
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Default => "default",
            Theme::HighContrast => "high-contrast",
            Theme::Deuteranopia => "deuteranopia",
            Theme::Protanopia => "protanopia",
            Theme::Tritanopia => "tritanopia",
        }
    }
}

// Tab types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabType {
    Overview,
    Portfolios,
    Networking,
    NetworkingAddMember,
    Organization,
    Reporting,
    Transactions,
    History,
    Settings,
    Agent,
}

impl TabType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TabType::Overview => "Overview",
            TabType::Portfolios => "Portfolios",
            TabType::Networking => "Networking",
            TabType::NetworkingAddMember => "Add Team",
            TabType::Organization => "Organization",
            TabType::Reporting => "Reporting",
            TabType::Transactions => "Transactions",
            TabType::History => "History",
            TabType::Settings => "Settings",
            TabType::Agent => "Agent",
        }
    }
}

// Organization settings
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub default_currency: Currency,
    pub default_payment_interval: PaymentInterval,
    pub notification_preferences: Vec<(NotificationTrigger, Vec<NotificationType>)>,
    pub theme: Theme,
    pub custom_fields: Vec<String>,
}

// User profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Guest".to_string(),
            email: String::new(),
            role: UserRole::Worker,
            organization_id: None,
            settings: OrganizationSettings::default(),
            created_at: Utc::now(),
        }
    }
}

// API response wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

// Pagination
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page,
            per_page,
            total: 0,
        }
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }
}
