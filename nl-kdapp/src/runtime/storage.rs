//! Storage abstraction for Episodes
//! 
//! Supports both ephemeral (current) and persistent (future) storage modes.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage trait for Episode state
pub trait EpisodeStorage: Send + Sync {
    /// Save Episode state
    fn save_state(&self, id: &str, state: &[u8]) -> Result<()>;
    
    /// Load Episode state
    fn load_state(&self, id: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete Episode state
    fn delete_state(&self, id: &str) -> Result<()>;
    
    /// List all Episode IDs
    fn list_episodes(&self) -> Result<Vec<String>>;
}

/// Ephemeral (in-memory) storage for POC
pub struct EphemeralStorage {
    episodes: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl EphemeralStorage {
    pub fn new() -> Self {
        Self {
            episodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl EpisodeStorage for EphemeralStorage {
    fn save_state(&self, id: &str, state: &[u8]) -> Result<()> {
        // This is sync but we're in an async context
        let episodes = self.episodes.clone();
        let id = id.to_string();
        let state = state.to_vec();
        
        tokio::task::block_in_place(move || {
            let mut episodes = tokio::runtime::Handle::current()
                .block_on(episodes.write());
            episodes.insert(id, state);
        });
        
        Ok(())
    }
    
    fn load_state(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let episodes = self.episodes.clone();
        let id = id.to_string();
        
        let state = tokio::task::block_in_place(move || {
            let episodes = tokio::runtime::Handle::current()
                .block_on(episodes.read());
            episodes.get(&id).cloned()
        });
        
        Ok(state)
    }
    
    fn delete_state(&self, id: &str) -> Result<()> {
        let episodes = self.episodes.clone();
        let id = id.to_string();
        
        tokio::task::block_in_place(move || {
            let mut episodes = tokio::runtime::Handle::current()
                .block_on(episodes.write());
            episodes.remove(&id);
        });
        
        Ok(())
    }
    
    fn list_episodes(&self) -> Result<Vec<String>> {
        let episodes = self.episodes.clone();
        
        let ids = tokio::task::block_in_place(move || {
            let episodes = tokio::runtime::Handle::current()
                .block_on(episodes.read());
            episodes.keys().cloned().collect()
        });
        
        Ok(ids)
    }
}

// Future implementation placeholder
#[allow(dead_code)]
pub struct PersistentStorage {
    // Will use RocksDB or similar
    _path: String,
}

#[allow(dead_code)]
impl PersistentStorage {
    pub fn new(path: &str) -> Self {
        Self {
            _path: path.to_string(),
        }
    }
}