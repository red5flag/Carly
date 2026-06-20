use crate::models::{Organization, Permission, Portfolio, User};
use crate::stores::credentials::CredentialStore;
use crate::types::{TabType, Theme, UserProfile, UserRole};
use crate::utils::crypto;
use leptos::prelude::*;
use uuid::Uuid;

// Main application state store
#[derive(Clone, Debug)]
pub struct AppStore {
    // Current user
    pub current_user: UserProfile,
    // Currently active tab
    pub active_tab: Option<TabType>,
    // Currently expanded tab (if any)
    pub expanded_tab: Option<TabType>,
    // All portfolios
    pub portfolios: Vec<Portfolio>,
    // Selected portfolio/asset IDs
    pub selected_portfolio_id: Option<Uuid>,
    pub selected_asset_group_id: Option<Uuid>,
    pub selected_asset_id: Option<Uuid>,
    // UI state
    pub is_search_open: bool,
    pub search_query: String,
    pub theme: Theme,
    // Notifications
    pub notifications: Vec<Notification>,
    // Modal state
    pub active_modal: Option<ModalType>,
    // Loading states
    pub is_loading: bool,
    // Network users (for networking tab) with role and privilege management
    pub organization_users: Vec<User>,
    // View mode for portfolios
    pub portfolio_view_mode: crate::types::ViewMode,
    // Authentication state
    pub is_authenticated: bool,
    // Organizations
    pub organizations: Vec<Organization>,
    // Credential store for password verification
    pub credentials: CredentialStore,
    // Encryption key for caching (derived from password hash)
    pub cache_key: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalType {
    CreatePortfolio,
    EditPortfolio(Uuid),
    CreateAssetGroup(Uuid), // portfolio_id
    EditAssetGroup(Uuid),
    CreateAsset(Uuid), // group_id
    EditAsset(Uuid),
    DeleteConfirmation {
        entity_type: String,
        entity_id: Uuid,
        entity_name: String,
    },
    QuickSale(Uuid), // asset_id
    Payout {
        asset_ids: Vec<Uuid>,
        recipients: Vec<Uuid>,
    },
    Notify {
        portfolio_ids: Vec<Uuid>,
        asset_ids: Vec<Uuid>,
    },
    UserDetails(Uuid),
    PaymentSetup(Uuid),
    SettingsEditor,
}

impl Default for AppStore {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut credentials = CredentialStore::with_defaults();

        #[cfg(feature = "hydrate")]
        credentials.merge_from_local_storage();

        Self {
            current_user: UserProfile::default(),
            active_tab: None,
            expanded_tab: None,
            portfolios: Vec::new(),
            selected_portfolio_id: None,
            selected_asset_group_id: None,
            selected_asset_id: None,
            is_search_open: false,
            search_query: String::new(),
            theme: Theme::default(),
            notifications: Vec::new(),
            active_modal: None,
            is_loading: false,
            organization_users: Vec::new(),
            portfolio_view_mode: crate::types::ViewMode::List,
            is_authenticated: false,
            organizations: Vec::new(),
            credentials,
            cache_key: Vec::new(),
        }
    }
}

impl AppStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Tab management
    pub fn expand_tab(&mut self, tab: TabType) {
        self.expanded_tab = Some(tab.clone());
        self.active_tab = Some(tab);
    }

    pub fn collapse_tab(&mut self) {
        self.expanded_tab = None;
        self.active_tab = None;
    }

    pub fn is_tab_expanded(&self, tab: &TabType) -> bool {
        self.expanded_tab.as_ref() == Some(tab)
    }

    // Portfolio management
    pub fn add_portfolio(&mut self, portfolio: Portfolio) {
        self.portfolios.push(portfolio);
    }

    pub fn get_portfolio(&self, id: Uuid) -> Option<&Portfolio> {
        self.portfolios.iter().find(|p| p.id == id)
    }

    pub fn get_portfolio_mut(&mut self, id: Uuid) -> Option<&mut Portfolio> {
        self.portfolios.iter_mut().find(|p| p.id == id)
    }

    pub fn remove_portfolio(&mut self, id: Uuid) -> Option<Portfolio> {
        if let Some(pos) = self.portfolios.iter().position(|p| p.id == id) {
            Some(self.portfolios.remove(pos))
        } else {
            None
        }
    }

    // Organization user management
    pub fn add_organization_user(&mut self, user: User) {
        self.organization_users.push(user);
    }

    pub fn remove_organization_user(&mut self, id: Uuid) -> Option<User> {
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            Some(self.organization_users.remove(pos))
        } else {
            None
        }
    }

    pub fn update_user_role(&mut self, id: Uuid, new_role: UserRole) -> Result<(), String> {
        let current_user_id = self.current_user.id;
        if let Some(pos) = self.organization_users.iter().position(|u| u.id == id) {
            let current_user = self
                .organization_users
                .iter()
                .find(|u| u.id == current_user_id)
                .cloned()
                .unwrap_or_else(|| User::new("Current".to_string(), String::new(), UserRole::Owner));
            let user = &mut self.organization_users[pos];
            user.update_role(new_role, &current_user)
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn toggle_user_permission(&mut self, id: Uuid, permission: Permission) {
        if let Some(user) = self.organization_users.iter_mut().find(|u| u.id == id) {
            user.toggle_permission(permission);
        }
    }

    // Search functionality
    pub fn open_search(&mut self) {
        self.is_search_open = true;
    }

    pub fn close_search(&mut self) {
        self.is_search_open = false;
        self.search_query.clear();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    // Theme management
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    // Notification management
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notifications.push(Notification {
            id: Uuid::new_v4(),
            message,
            notification_type,
            timestamp: chrono::Utc::now(),
        });

        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    pub fn remove_notification(&mut self, id: Uuid) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    // Modal management
    pub fn open_modal(&mut self, modal_type: ModalType) {
        self.active_modal = Some(modal_type);
    }

    pub fn close_modal(&mut self) {
        self.active_modal = None;
    }

    // Loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    // Authentication
    pub fn login_with_credentials(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(String, String), String> {
        let cred = self
            .credentials
            .verify(username, password)
            .ok_or("Invalid username or password")?;

        if !cred.validated {
            return Err("Account not validated. Please check your email or validate via /emailvalid.".to_string());
        }

        let display_name = cred.display_name.clone();
        let email = cred.email.clone();

        // Derive encryption key from password hash
        self.cache_key = crypto::derive_key(&cred.password_hash);

        // Set user profile
        self.is_authenticated = true;
        self.current_user.name = display_name.clone();
        self.current_user.email = email.clone();
        self.current_user.role = UserRole::Owner;

        // Seed a default portfolio if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            let mut p = Portfolio::new("Commercial Real Estate".to_string(), owner_id, crate::types::Currency::USD);
            p.description = Some("Office buildings and retail spaces".to_string());
            p.tags = vec!["real-estate".to_string(), "commercial".to_string()];

            let mut hq = crate::models::Asset::new("Headquarters".to_string(), crate::types::AssetType::RealEstate, 5000000.0);
            hq.update_value(6200000.0);
            p.assets.push(hq);

            let mut group1 = crate::models::AssetGroup::new("Downtown Properties".to_string());
            let mut a1 = crate::models::Asset::new("Main Office Building".to_string(), crate::types::AssetType::RealEstate, 2500000.0);
            a1.update_value(3200000.0);
            let mut a2 = crate::models::Asset::new("Retail Plaza".to_string(), crate::types::AssetType::RealEstate, 1200000.0);
            a2.update_value(1450000.0);
            group1.assets = vec![a1, a2];
            group1.recalculate_values();

            let mut group2 = crate::models::AssetGroup::new("Suburban Offices".to_string());
            let mut a3 = crate::models::Asset::new("Tech Park Building A".to_string(), crate::types::AssetType::RealEstate, 1800000.0);
            a3.update_value(2100000.0);
            let mut a4 = crate::models::Asset::new("Tech Park Building B".to_string(), crate::types::AssetType::RealEstate, 1600000.0);
            a4.update_value(1850000.0);
            group2.assets = vec![a3, a4];
            group2.recalculate_values();

            p.asset_groups = vec![group1, group2];
            p.recalculate_values();

            self.portfolios.push(p);
        }

        // Pre-cache home page with PQC encryption
        let home_data = serde_json::json!({
            "user": display_name,
            "email": email,
            "portfolios": self.portfolios.len(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string();

        let _ = crypto::cache_to_local("farley_home_cache", &home_data, &self.cache_key);

        // Go to home page (expand Overview tab)
        self.expand_tab(TabType::Overview);

        Ok((display_name, format!("{:?}", self.current_user.role)))
    }

    pub fn login(&mut self, name: String, email: String, role: UserRole) {
        self.is_authenticated = true;
        self.current_user.name = name;
        self.current_user.email = email;
        self.current_user.role = role;

        // Seed a default portfolio if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            let mut p = Portfolio::new("Commercial Real Estate".to_string(), owner_id, crate::types::Currency::USD);
            p.description = Some("Office buildings and retail spaces".to_string());
            p.tags = vec!["real-estate".to_string(), "commercial".to_string()];

            let mut hq = crate::models::Asset::new("Headquarters".to_string(), crate::types::AssetType::RealEstate, 5000000.0);
            hq.update_value(6200000.0);
            p.assets.push(hq);

            let mut group1 = crate::models::AssetGroup::new("Downtown Properties".to_string());
            let mut a1 = crate::models::Asset::new("Main Office Building".to_string(), crate::types::AssetType::RealEstate, 2500000.0);
            a1.update_value(3200000.0);
            let mut a2 = crate::models::Asset::new("Retail Plaza".to_string(), crate::types::AssetType::RealEstate, 1200000.0);
            a2.update_value(1450000.0);
            group1.assets = vec![a1, a2];
            group1.recalculate_values();

            let mut group2 = crate::models::AssetGroup::new("Suburban Offices".to_string());
            let mut a3 = crate::models::Asset::new("Tech Park Building A".to_string(), crate::types::AssetType::RealEstate, 1800000.0);
            a3.update_value(2100000.0);
            let mut a4 = crate::models::Asset::new("Tech Park Building B".to_string(), crate::types::AssetType::RealEstate, 1600000.0);
            a4.update_value(1850000.0);
            group2.assets = vec![a3, a4];
            group2.recalculate_values();

            p.asset_groups = vec![group1, group2];
            p.recalculate_values();

            self.portfolios.push(p);
        }
    }

    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.current_user = UserProfile::default();
        self.collapse_tab();
        self.close_search();
        self.selected_portfolio_id = None;
        self.selected_asset_group_id = None;
        self.selected_asset_id = None;
        self.cache_key = Vec::new();
        self.portfolios.clear();
        let _ = crypto::clear_cached("farley_home_cache");
    }

    /// Register a new user in the credential store
    pub fn register_user(
        &mut self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<(), String> {
        let display_name = username;
        let result = self.credentials.register_user(username, password, display_name, email);

        #[cfg(feature = "hydrate")]
        if result.is_ok() {
            self.credentials.save_to_local_storage();
        }

        result
    }

    /// Check if password matches locally (regardless of validation)
    pub fn check_password(&self, username: &str, password: &str) -> bool {
        self.credentials.verify_password_only(username, password)
    }

    /// Check if user is validated locally
    pub fn is_user_validated(&self, username: &str) -> bool {
        self.credentials.is_validated(username)
    }

    /// Mark a user as validated locally
    pub fn mark_user_validated(&mut self, username: &str) {
        self.credentials.mark_validated(username);
    }

    /// Check if a user exists in local credentials
    pub fn user_exists(&self, username: &str) -> bool {
        self.credentials.user_exists(username)
    }

    /// Login a server-validated user (from /api/login after email validation)
    pub fn login_server_validated(&mut self, display_name: &str, email: &str) {
        // Derive encryption key from display name + email as a simple key
        self.cache_key = crypto::derive_key(&format!("{}:{}", display_name, email));

        // Set user profile
        self.is_authenticated = true;
        self.current_user.name = display_name.to_string();
        self.current_user.email = email.to_string();
        self.current_user.role = UserRole::Owner;

        // Also mark this user as validated locally so future local logins work
        if !display_name.is_empty() {
            self.credentials.mark_validated(display_name);
            #[cfg(feature = "hydrate")]
            self.credentials.save_to_local_storage();
        }

        // Seed a default portfolio if none exist
        if self.portfolios.is_empty() {
            let owner_id = self.current_user.id;
            let mut p = Portfolio::new("Commercial Real Estate".to_string(), owner_id, crate::types::Currency::USD);
            p.description = Some("Office buildings and retail spaces".to_string());
            p.tags = vec!["real-estate".to_string(), "commercial".to_string()];

            let mut hq = crate::models::Asset::new("Headquarters".to_string(), crate::types::AssetType::RealEstate, 5000000.0);
            hq.update_value(6200000.0);
            p.assets.push(hq);

            let mut group1 = crate::models::AssetGroup::new("Downtown Properties".to_string());
            let mut a1 = crate::models::Asset::new("Main Office Building".to_string(), crate::types::AssetType::RealEstate, 2500000.0);
            a1.update_value(3200000.0);
            let mut a2 = crate::models::Asset::new("Retail Plaza".to_string(), crate::types::AssetType::RealEstate, 1200000.0);
            a2.update_value(1450000.0);
            group1.assets = vec![a1, a2];
            group1.recalculate_values();

            let mut group2 = crate::models::AssetGroup::new("Suburban Offices".to_string());
            let mut a3 = crate::models::Asset::new("Tech Park Building A".to_string(), crate::types::AssetType::RealEstate, 1800000.0);
            a3.update_value(2100000.0);
            let mut a4 = crate::models::Asset::new("Tech Park Building B".to_string(), crate::types::AssetType::RealEstate, 1600000.0);
            a4.update_value(1850000.0);
            group2.assets = vec![a3, a4];
            group2.recalculate_values();

            p.asset_groups = vec![group1, group2];
            p.recalculate_values();

            self.portfolios.push(p);
        }

        // Pre-cache home page with PQC encryption
        let home_data = serde_json::json!({
            "user": display_name,
            "email": email,
            "portfolios": self.portfolios.len(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string();

        let _ = crypto::cache_to_local("farley_home_cache", &home_data, &self.cache_key);

        // Go to home page (expand Overview tab)
        self.expand_tab(TabType::Overview);
    }

    // Organization management
    pub fn add_organization(&mut self, org: Organization) {
        self.organizations.push(org);
    }

    pub fn get_organization(&self, id: Uuid) -> Option<&Organization> {
        self.organizations.iter().find(|o| o.id == id)
    }

    pub fn get_organization_mut(&mut self, id: Uuid) -> Option<&mut Organization> {
        self.organizations.iter_mut().find(|o| o.id == id)
    }

    pub fn remove_organization(&mut self, id: Uuid) -> Option<Organization> {
        if let Some(pos) = self.organizations.iter().position(|o| o.id == id) {
            Some(self.organizations.remove(pos))
        } else {
            None
        }
    }

    // Get location name for navbar
    pub fn get_current_location(&self) -> String {
        if let Some(ref tab) = self.expanded_tab {
            match tab {
                TabType::Overview => "Overview".to_string(),
                TabType::Portfolios => {
                    if let Some(id) = self.selected_portfolio_id {
                        if let Some(p) = self.get_portfolio(id) {
                            return format!("Portfolio: {}", p.name);
                        }
                    }
                    "Portfolios".to_string()
                }
                TabType::Networking => "Networking".to_string(),
                TabType::Organization => "Organization".to_string(),
                TabType::Transactions => "Transactions".to_string(),
                TabType::History => "History".to_string(),
                TabType::Settings => "Settings".to_string(),
                TabType::Agent => "Agent".to_string(),
            }
        } else {
            "Home".to_string()
        }
    }
}

// Create a signal-based store for Leptos
pub fn create_app_store() -> RwSignal<AppStore> {
    RwSignal::new(AppStore::new())
}

// Context provider for the app store
pub fn provide_app_store() -> RwSignal<AppStore> {
    let store = create_app_store();
    provide_context(store);
    store
}

// Hook to use the app store
pub fn use_app_store() -> RwSignal<AppStore> {
    expect_context::<RwSignal<AppStore>>()
}
