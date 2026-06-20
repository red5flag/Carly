use crate::models::Action;
use crate::types::ActionType;
use leptos::prelude::*;
use std::collections::VecDeque;
use uuid::Uuid;

const MAX_HISTORY_SIZE: usize = 100;

// Helper to create an action with user info populated from AppStore
pub fn create_action(
    action_type: ActionType,
    entity_type: &str,
    description: &str,
    user_id: Uuid,
    user_name: &str,
    user_role: &str,
    org_id: Option<Uuid>,
) -> Action {
    Action::new(
        action_type,
        entity_type.to_string(),
        Uuid::new_v4(),
        description.to_string(),
        user_id,
    )
    .with_user(user_name.to_string(), user_role.to_string(), org_id)
}

// Undo/Redo store for complete action history
#[derive(Clone, Debug)]
pub struct UndoRedoStore {
    // History of actions - past actions that can be undone
    pub past: VecDeque<Action>,
    // Future actions that can be redone (after undo)
    pub future: VecDeque<Action>,
    // Current state reference
    pub current_action_id: Option<Uuid>,
}

impl Default for UndoRedoStore {
    fn default() -> Self {
        Self {
            past: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            future: VecDeque::new(),
            current_action_id: None,
        }
    }
}

impl UndoRedoStore {
    pub fn new() -> Self {
        Self::default()
    }

    // Record a new action, clearing the redo history
    pub fn record_action(&mut self, action: Action) {
        // Clear redo history when new action is performed
        self.future.clear();

        // Add to past history
        self.past.push_back(action);

        // Maintain max history size
        if self.past.len() > MAX_HISTORY_SIZE {
            self.past.pop_front();
        }
    }

    // Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    // Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    // Undo the last action
    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.past.pop_back() {
            self.future.push_front(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Redo the next action
    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.future.pop_front() {
            self.past.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    // Get the last action without removing it
    pub fn peek_last(&self) -> Option<&Action> {
        self.past.back()
    }

    // Get recent actions for history view
    pub fn get_recent_actions(&self, count: usize) -> Vec<Action> {
        self.past.iter().rev().take(count).cloned().collect()
    }

    // Clear all history
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.current_action_id = None;
    }

    // Get action by ID
    pub fn get_action(&self, id: Uuid) -> Option<&Action> {
        self.past.iter().find(|a| a.id == id)
    }

    // Filter actions by type
    pub fn filter_by_type(&self, action_type: ActionType) -> Vec<&Action> {
        self.past
            .iter()
            .filter(|a| a.action_type == action_type)
            .collect()
    }

    // Filter actions by entity
    pub fn filter_by_entity(&self, entity_type: &str, entity_id: Uuid) -> Vec<&Action> {
        self.past
            .iter()
            .filter(|a| a.entity_type == entity_type && a.entity_id == entity_id)
            .collect()
    }

    // Get count of actions
    pub fn len(&self) -> usize {
        self.past.len()
    }

    pub fn is_empty(&self) -> bool {
        self.past.is_empty()
    }
}

// Create a signal-based store for Leptos
pub fn create_undo_redo_store() -> RwSignal<UndoRedoStore> {
    RwSignal::new(UndoRedoStore::new())
}

// Helper to format action for display
pub fn format_action_description(action: &Action) -> String {
    match action.action_type {
        ActionType::Create => format!("Created {}: {}", action.entity_type, action.description),
        ActionType::Update => format!("Updated {}: {}", action.entity_type, action.description),
        ActionType::Delete => format!("Deleted {}: {}", action.entity_type, action.description),
        ActionType::View => format!("Viewed {}: {}", action.entity_type, action.description),
        ActionType::Navigate => format!(
            "Navigated from {} to {}",
            action.navigated_from.as_deref().unwrap_or("unknown"),
            action.navigated_to.as_deref().unwrap_or("unknown")
        ),
        ActionType::Setting => format!("Changed setting: {}", action.description),
        ActionType::Payment => format!("Payment: {}", action.description),
        ActionType::Notification => format!("Notification: {}", action.description),
        ActionType::Search => format!("Search: {}", action.description),
        ActionType::Undo => format!("Undo: {}", action.description),
        ActionType::Redo => format!("Redo: {}", action.description),
        ActionType::Login => format!("Login: {}", action.description),
        ActionType::Logout => format!("Logout: {}", action.description),
    }
}

// Undo handler trait for different action types
pub trait UndoHandler: Send + Sync {
    fn can_undo(&self, action: &Action) -> bool;
    fn undo(&self, action: &Action) -> Result<(), String>;
    fn redo(&self, action: &Action) -> Result<(), String>;
}

// Registry of undo handlers
pub struct UndoHandlerRegistry {
    handlers: Vec<Box<dyn UndoHandler>>,
}

impl Default for UndoHandlerRegistry {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl UndoHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Box<dyn UndoHandler>) {
        self.handlers.push(handler);
    }

    pub fn find_handler(&self, action: &Action) -> Option<&dyn UndoHandler> {
        self.handlers
            .iter()
            .find(|h| h.can_undo(action))
            .map(|h| h.as_ref())
    }
}
