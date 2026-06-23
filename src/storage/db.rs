use crate::models::Portfolio;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

const DB_PATH: &str = "./data/portfolios";

/// Global singleton: sled DB handle + DashMap write-through cache.
pub struct PortfolioStore {
    db: sled::Db,
    cache: DashMap<Uuid, Portfolio>,
}

static STORE: OnceLock<Arc<PortfolioStore>> = OnceLock::new();

/// Initialise (or retrieve) the global store. Call once at server start.
pub fn portfolio_store() -> Arc<PortfolioStore> {
    STORE
        .get_or_init(|| Arc::new(PortfolioStore::open().expect("Failed to open portfolio DB")))
        .clone()
}

impl PortfolioStore {
    fn open() -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(DB_PATH)?;
        let db = sled::open(DB_PATH)?;

        let store = Self {
            db,
            cache: DashMap::new(),
        };

        store.warm_cache()?;
        Ok(store)
    }

    fn warm_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        for item in self.db.iter() {
            let (_k, v) = item?;
            if let Ok(p) = serde_json::from_slice::<Portfolio>(&v) {
                self.cache.insert(p.id, p);
            }
        }
        Ok(())
    }

    /// Persist a single portfolio (write-through to sled + update cache).
    pub fn save(&self, portfolio: &Portfolio) -> Result<(), Box<dyn std::error::Error>> {
        let key = portfolio.id.to_string();
        let value = serde_json::to_vec(portfolio)?;
        self.db.insert(key.as_bytes(), value)?;
        self.cache.insert(portfolio.id, portfolio.clone());
        Ok(())
    }

    /// Load a single portfolio by ID (cache-first).
    pub fn load(&self, id: Uuid) -> Option<Portfolio> {
        if let Some(p) = self.cache.get(&id) {
            return Some(p.clone());
        }
        let key = id.to_string();
        self.db
            .get(key.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice::<Portfolio>(&v).ok())
            .inspect(|p| {
                self.cache.insert(p.id, p.clone());
            })
    }

    /// Load all portfolios for a given owner (from cache).
    pub fn load_all_for_owner(&self, owner_id: Uuid) -> Vec<Portfolio> {
        self.cache
            .iter()
            .filter(|e| e.owner_id == owner_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Delete a portfolio from DB and cache.
    pub fn delete(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let key = id.to_string();
        self.db.remove(key.as_bytes())?;
        self.cache.remove(&id);
        Ok(())
    }
}
