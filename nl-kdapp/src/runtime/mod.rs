//! Episode runtime module for server-side execution
//! 
//! Manages the lifecycle of Episodes running on kdapp.fun servers.
//! Episodes are ephemeral (in-memory) during POC phase.

mod executor;
mod manager;
mod storage;

pub use executor::EpisodeExecutor;
pub use manager::EpisodeManager;
pub use storage::{EpisodeStorage, EphemeralStorage};

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Episode metadata for tracking and display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetadata {
    pub id: String,
    pub episode_type: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub participant_count: usize,
    pub is_ephemeral: bool,
    pub creator_session: String,
}

impl EpisodeMetadata {
    pub fn new(id: String, episode_type: String, creator_session: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Episodes expire after 3 days (Kaspa pruning window)
        let expires_at = created_at + (3 * 24 * 60 * 60);
        
        Self {
            id,
            episode_type,
            created_at,
            expires_at,
            participant_count: 0,
            is_ephemeral: true, // Always true for POC
            creator_session,
        }
    }
    
    /// Check if Episode has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
    
    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.expires_at as i64 - now as i64
    }
}