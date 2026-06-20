use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredCredential {
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub email: String,
    pub validated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CredentialStore {
    pub credentials: HashMap<String, StoredCredential>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-seed with default red/red credential (validated)
    pub fn with_defaults() -> Self {
        let mut store = Self::new();
        store.add_user("red", "red", "Red", "red@farley.app", true);
        store
    }

    /// Load credentials from localStorage (hydrate only)
    #[cfg(feature = "hydrate")]
    pub fn load_from_local_storage() -> Self {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item("farley_credentials") {
                    if let Ok(store) = serde_json::from_str::<Self>(&json) {
                        return store;
                    }
                }
            }
        }
        Self::new()
    }

    /// Save credentials to localStorage (hydrate only)
    #[cfg(feature = "hydrate")]
    pub fn save_to_local_storage(&self) {
        use web_sys::window;
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item("farley_credentials", &json);
                }
            }
        }
    }

    /// Merge locally stored credentials into this store (hydrate only)
    #[cfg(feature = "hydrate")]
    pub fn merge_from_local_storage(&mut self) {
        let stored = Self::load_from_local_storage();
        for (username, cred) in stored.credentials {
            // Only add credentials that don't already exist locally,
            // so the default validated red user isn't overwritten by stale storage.
            self.credentials.entry(username).or_insert(cred);
        }
    }

    /// Hash a password using argon2
    pub fn hash_password(password: &str) -> Result<String, String> {
        let mut salt_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| format!("Failed to create salt: {}", e))?;
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Failed to hash password: {}", e))
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed_hash) => {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .is_ok()
            }
            Err(_) => false,
        }
    }

    /// Add a user with username and password (password will be hashed)
    pub fn add_user(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
        validated: bool,
    ) {
        if let Ok(hash) = Self::hash_password(password) {
            self.credentials.insert(
                username.to_string(),
                StoredCredential {
                    username: username.to_string(),
                    password_hash: hash,
                    display_name: display_name.to_string(),
                    email: email.to_string(),
                    validated,
                },
            );
        }
    }

    /// Verify credentials against stored users
    pub fn verify(&self, username: &str, password: &str) -> Option<&StoredCredential> {
        self.credentials
            .get(username)
            .filter(|cred| Self::verify_password(password, &cred.password_hash))
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> bool {
        self.credentials.contains_key(username)
    }

    /// Register a new user. Returns Ok if successful, Err with message if user exists.
    pub fn register_user(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        email: &str,
    ) -> Result<(), String> {
        if self.user_exists(username) {
            return Err(format!("Username '{}' is already taken", username));
        }
        if password.len() < 3 {
            return Err("Password must be at least 3 characters".to_string());
        }
        if username.trim().is_empty() {
            return Err("Username is required".to_string());
        }
        if !email.contains('@') {
            return Err("A valid email is required".to_string());
        }
        self.add_user(username, password, display_name, email, false);
        Ok(())
    }

    /// Mark a user as validated
    pub fn mark_validated(&mut self, username: &str) {
        if let Some(cred) = self.credentials.get_mut(username) {
            cred.validated = true;
        }
    }

    /// Check if a user's email has been validated
    pub fn is_validated(&self, username: &str) -> bool {
        self.credentials.get(username).map(|c| c.validated).unwrap_or(false)
    }

    /// Verify password without checking validation status
    pub fn verify_password_only(&self, username: &str, password: &str) -> bool {
        self.credentials
            .get(username)
            .map(|cred| Self::verify_password(password, &cred.password_hash))
            .unwrap_or(false)
    }
}
