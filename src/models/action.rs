use crate::types::ActionType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Action record for complete undo/redo system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub action_type: ActionType,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub description: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_role: String,
    pub organization_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub previous_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub tab_context: Option<String>,
    pub navigated_from: Option<String>,
    pub navigated_to: Option<String>,
    pub metadata: serde_json::Value,
}

impl Action {
    pub fn new(
        action_type: ActionType,
        entity_type: String,
        entity_id: Uuid,
        description: String,
        user_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type,
            entity_type,
            entity_id,
            description,
            user_id,
            user_name: String::new(),
            user_role: String::new(),
            organization_id: None,
            timestamp: Utc::now(),
            previous_state: None,
            new_state: None,
            tab_context: None,
            navigated_from: None,
            navigated_to: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_user(mut self, name: String, role: String, org_id: Option<Uuid>) -> Self {
        self.user_name = name;
        self.user_role = role;
        self.organization_id = org_id;
        self
    }

    pub fn with_state(
        mut self,
        previous: serde_json::Value,
        new: serde_json::Value,
    ) -> Self {
        self.previous_state = Some(previous);
        self.new_state = Some(new);
        self
    }

    pub fn with_navigation(mut self, from: String, to: String) -> Self {
        self.navigated_from = Some(from);
        self.navigated_to = Some(to);
        self.action_type = ActionType::Navigate;
        self
    }

    pub fn with_tab_context(mut self, tab: String) -> Self {
        self.tab_context = Some(tab);
        self
    }
}

// Action history entry with user info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionHistoryEntry {
    pub action: Action,
    pub user_name: String,
    pub user_role: String,
    pub organization_id: Option<Uuid>,
}

// Filter for history queries
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActionFilter {
    pub action_types: Vec<ActionType>,
    pub entity_types: Vec<String>,
    pub user_ids: Vec<Uuid>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub tab_context: Option<String>,
}
