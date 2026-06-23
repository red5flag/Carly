use crate::models::{Asset, Portfolio};
use crate::types::{SearchFilters, SortMode, TabType};
use leptos::prelude::*;

// Search store with Meilisearch integration
#[derive(Clone, Debug)]
pub struct SearchStore {
    // Current search query
    pub query: String,
    // Active search filters
    pub filters: SearchFilters,
    // Search results
    pub results: SearchResults,
    // Is search loading
    pub is_loading: bool,
    // Recent searches
    pub recent_searches: Vec<String>,
    // Search suggestions
    pub suggestions: Vec<String>,
    // Context-aware priority
    pub current_tab: Option<TabType>,
    // Whether the advanced search panel is open
    pub is_advanced_search_open: bool,
    // Available filter tags
    pub available_tags: Vec<String>,
    // Selected filter tags
    pub selected_tags: Vec<String>,
    // Search history for current session
    pub search_history: Vec<SearchQuery>,
    // Sort mode for search results
    pub sort_mode: SortMode,
}

#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub query: String,
    pub filters: SearchFilters,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SearchResults {
    pub portfolios: Vec<Portfolio>,
    pub assets: Vec<Asset>,
    pub total_count: usize,
    pub search_time_ms: u64,
}

impl Default for SearchStore {
    fn default() -> Self {
        Self {
            query: String::new(),
            filters: SearchFilters::default(),
            results: SearchResults::default(),
            is_loading: false,
            recent_searches: Vec::new(),
            suggestions: Vec::new(),
            current_tab: None,
            is_advanced_search_open: false,
            available_tags: vec![
                "Real Estate".to_string(),
                "Vehicle".to_string(),
                "Equipment".to_string(),
                "Stock".to_string(),
                "Active".to_string(),
                "For Sale".to_string(),
                "High Value".to_string(),
                "Trending".to_string(),
            ],
            selected_tags: Vec::new(),
            search_history: Vec::new(),
            sort_mode: SortMode::Recent,
        }
    }
}

impl SearchStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Set search query
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.update_suggestions();
    }

    // Clear search
    pub fn clear(&mut self) {
        self.query.clear();
        self.filters = SearchFilters::default();
        self.results = SearchResults::default();
        self.selected_tags.clear();
        self.sort_mode = SortMode::Recent;
    }

    // Set sort mode
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }

    // Add tag filter
    pub fn toggle_tag(&mut self, tag: String) {
        if self.selected_tags.contains(&tag) {
            self.selected_tags.retain(|t| t != &tag);
        } else {
            self.selected_tags.push(tag);
        }
        self.update_filters_from_tags();
    }

    // Update filters based on selected tags
    fn update_filters_from_tags(&mut self) {
        // Convert tags to filters
        self.filters.tags = self.selected_tags.clone();

        // Map common tags to asset types
        self.filters.asset_types = self
            .selected_tags
            .iter()
            .filter_map(|tag| match tag.as_str() {
                "Real Estate" => Some(crate::types::AssetType::RealEstate),
                "Vehicle" => Some(crate::types::AssetType::Vehicle),
                "Equipment" => Some(crate::types::AssetType::Equipment),
                "Stock" => Some(crate::types::AssetType::Stock),
                _ => None,
            })
            .collect();
    }

    // Update suggestions based on query
    fn update_suggestions(&mut self) {
        if self.query.len() < 2 {
            self.suggestions.clear();
            return;
        }

        // Generate suggestions based on query and context
        let query_lower = self.query.to_lowercase();
        self.suggestions = self
            .available_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&query_lower))
            .take(5)
            .cloned()
            .collect();
    }

    // Perform search (mock implementation - would integrate with Meilisearch)
    pub async fn perform_search(&mut self) {
        self.is_loading = true;

        // Simulate search delay
        gloo_timers::future::TimeoutFuture::new(300).await;

        // In a real implementation, this would call Meilisearch API
        // For now, we'll return empty results
        self.results = SearchResults {
            portfolios: Vec::new(),
            assets: Vec::new(),
            total_count: 0,
            search_time_ms: 0,
        };

        // Record search in history
        self.search_history.push(SearchQuery {
            query: self.query.clone(),
            filters: self.filters.clone(),
            timestamp: chrono::Utc::now(),
            result_count: self.results.total_count,
        });

        // Keep only last 20 searches
        if self.search_history.len() > 20 {
            self.search_history.remove(0);
        }

        self.is_loading = false;
    }

    // Toggle advanced search panel
    pub fn toggle_advanced_search(&mut self) {
        self.is_advanced_search_open = !self.is_advanced_search_open;
    }

    pub fn open_advanced_search(&mut self) {
        self.is_advanced_search_open = true;
    }

    pub fn close_advanced_search(&mut self) {
        self.is_advanced_search_open = false;
    }

    // Set context tab for priority searching
    pub fn set_context_tab(&mut self, tab: TabType) {
        self.current_tab = Some(tab.clone());

        // Adjust search priorities based on tab
        match tab {
            TabType::Overview => {
                // Prioritize trending/recent
            }
            TabType::Portfolios => {
                // Prioritize portfolios and assets
            }
            TabType::Networking | TabType::NetworkingAddMember => {
                // Prioritize users
            }
            TabType::Organization => {
                // Prioritize organizations and members
            }
            TabType::Reporting => {
                // Prioritize documents, payslips, hours, deeds
            }
            TabType::Transactions => {
                // Prioritize transactions
            }
            TabType::History => {
                // Prioritize actions
            }
            TabType::Settings => {
                // Prioritize settings
            }
            TabType::Agent => {
                // No specific priority
            }
        }
    }

    // Get search suggestions with context
    pub fn get_contextual_suggestions(&self) -> Vec<String> {
        let mut suggestions = self.suggestions.clone();

        // Add context-specific suggestions
        if let Some(ref tab) = self.current_tab {
            match tab {
                TabType::Portfolios => {
                    suggestions.push("portfolio:".to_string());
                    suggestions.push("asset:".to_string());
                }
                TabType::Networking | TabType::NetworkingAddMember => {
                    suggestions.push("user:".to_string());
                    suggestions.push("role:".to_string());
                }
                TabType::Organization => {
                    suggestions.push("org:".to_string());
                    suggestions.push("member:".to_string());
                    suggestions.push("department:".to_string());
                }
                TabType::Reporting => {
                    suggestions.push("doc:".to_string());
                    suggestions.push("payslip:".to_string());
                    suggestions.push("deed:".to_string());
                    suggestions.push("registration:".to_string());
                }
                TabType::Transactions => {
                    suggestions.push("txn:".to_string());
                    suggestions.push("status:".to_string());
                }
                TabType::History => {
                    suggestions.push("action:".to_string());
                    suggestions.push("date:".to_string());
                }
                _ => {}
            }
        }

        suggestions.truncate(8);
        suggestions
    }

    // Export search as URL query string
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if !self.query.is_empty() {
            params.push(format!("q={}", self.query));
        }

        for tag in &self.selected_tags {
            params.push(format!("tag={}", tag));
        }

        params.join("&")
    }
}

// Create a signal-based store for Leptos
pub fn create_search_store() -> RwSignal<SearchStore> {
    RwSignal::new(SearchStore::new())
}

// Context provider for the search store
pub fn provide_search_store() -> RwSignal<SearchStore> {
    let store = create_search_store();
    provide_context(store);
    store
}

// Hook to use the search store
pub fn use_search_store() -> RwSignal<SearchStore> {
    expect_context::<RwSignal<SearchStore>>()
}

// Meilisearch client configuration
#[derive(Clone, Debug)]
pub struct MeilisearchConfig {
    pub host: String,
    pub api_key: String,
    pub index_name: String,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost:7700".to_string(),
            api_key: String::new(),
            index_name: "farley".to_string(),
        }
    }
}

// Meilisearch search client (SSR only - uses reqwest)
#[cfg(feature = "ssr")]
pub struct MeilisearchClient {
    config: MeilisearchConfig,
    client: reqwest::Client,
}

#[cfg(feature = "ssr")]
impl MeilisearchClient {
    pub fn new(config: MeilisearchConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<SearchResults, reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/search",
            self.config.host, self.config.index_name
        );

        let search_request = serde_json::json!({
            "q": query,
            "filter": self.build_filter_string(filters),
            "limit": 20,
            "attributesToHighlight": ["name", "description"],
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&search_request)
            .send()
            .await?;

        // Parse response and convert to SearchResults
        // This is simplified - full implementation would parse the Meilisearch response
        let _search_response: serde_json::Value = response.json().await?;

        Ok(SearchResults::default())
    }

    fn build_filter_string(&self, filters: &SearchFilters) -> String {
        let mut filter_parts = Vec::new();

        if !filters.asset_types.is_empty() {
            let types: Vec<String> = filters
                .asset_types
                .iter()
                .map(|t| format!("asset_type = {:?}", t))
                .collect();
            filter_parts.push(format!("({})", types.join(" OR ")));
        }

        if let Some((min, max)) = filters.value_range {
            filter_parts.push(format!("current_value >= {} AND current_value <= {}", min, max));
        }

        filter_parts.join(" AND ")
    }

    pub async fn add_document(&self, document: &serde_json::Value) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/documents",
            self.config.host, self.config.index_name
        );

        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(document)
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_document(
        &self,
        document: &serde_json::Value,
    ) -> Result<(), reqwest::Error> {
        // Same as add - Meilisearch upserts
        self.add_document(document).await
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/indexes/{}/documents/{}",
            self.config.host, self.config.index_name, document_id
        );

        self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        Ok(())
    }
}
