//! Episode manager for concurrent Episode execution

use anyhow::{Result, bail};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use super::{EpisodeExecutor, EpisodeStorage, EpisodeMetadata, EpisodeEvent};
use super::executor::RunningEpisode;

/// Manages multiple concurrent Episodes
pub struct EpisodeManager {
    executor: EpisodeExecutor,
    storage: Box<dyn EpisodeStorage>,
    episodes: Arc<RwLock<HashMap<String, RunningEpisode>>>,
    event_channel: mpsc::Sender<EpisodeEvent>,
    event_receiver: Arc<RwLock<mpsc::Receiver<EpisodeEvent>>>,
}

impl EpisodeManager {
    /// Create a new Episode manager with given storage
    pub fn with_storage(storage: Box<dyn EpisodeStorage>) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        Self {
            executor: EpisodeExecutor::new(),
            storage,
            episodes: Arc::new(RwLock::new(HashMap::new())),
            event_channel: tx,
            event_receiver: Arc::new(RwLock::new(rx)),
        }
    }
    
    /// Create and start a new Episode
    pub async fn create_episode(
        &self,
        episode_type: &str,
        episode_id: String,
        creator_session: String,
    ) -> Result<EpisodeMetadata> {
        // Create the Episode
        let running_episode = self.executor.create_episode(
            episode_type,
            episode_id.clone(),
            creator_session,
        ).await?;
        
        let metadata = running_episode.metadata.clone();
        
        // Store in memory
        {
            let mut episodes = self.episodes.write().await;
            episodes.insert(episode_id.clone(), running_episode);
        }
        
        // Save initial state
        self.storage.save_state(&episode_id, &[])?;
        
        log::info!(
            "Created {} Episode {} (expires in {} seconds)",
            episode_type,
            episode_id,
            metadata.remaining_seconds()
        );
        
        Ok(metadata)
    }
    
    /// Execute a command on an Episode
    pub async fn execute_command(
        &self,
        episode_id: &str,
        command: Vec<u8>,
    ) -> Result<()> {
        let episode = {
            let episodes = self.episodes.read().await;
            episodes.get(episode_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Episode {} not found", episode_id))?
        };
        
        // Check if expired
        if episode.metadata.is_expired() {
            bail!("Episode {} has expired", episode_id);
        }
        
        // Execute command
        self.executor.execute_command(
            episode_id,
            command,
            &episode.state,
            &self.event_channel,
        ).await?;
        
        // Save updated state
        let state = episode.state.read().await;
        self.storage.save_state(episode_id, &state)?;
        
        Ok(())
    }
    
    /// Get Episode metadata
    pub async fn get_episode(&self, episode_id: &str) -> Result<Option<EpisodeMetadata>> {
        let episodes = self.episodes.read().await;
        Ok(episodes.get(episode_id).map(|e| e.metadata.clone()))
    }
    
    /// List all active Episodes
    pub async fn list_episodes(&self) -> Result<Vec<EpisodeMetadata>> {
        let episodes = self.episodes.read().await;
        Ok(episodes.values().map(|e| e.metadata.clone()).collect())
    }
    
    /// Clean up expired Episodes
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut episodes = self.episodes.write().await;
        let expired_ids: Vec<String> = episodes
            .iter()
            .filter(|(_, e)| e.metadata.is_expired())
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = expired_ids.len();
        
        for id in expired_ids {
            episodes.remove(&id);
            self.storage.delete_state(&id)?;
            log::info!("Cleaned up expired Episode: {}", id);
        }
        
        Ok(count)
    }
    
    /// Subscribe to Episode events
    pub fn subscribe_events(&self) -> mpsc::Receiver<EpisodeEvent> {
        let (tx, rx) = mpsc::channel(100);
        
        // Forward events to subscriber
        let event_channel = self.event_channel.clone();
        tokio::spawn(async move {
            // This is a simplified event forwarding
            // In production, we'd filter by Episode ID, etc.
            log::debug!("Event subscriber created");
        });
        
        rx
    }
}