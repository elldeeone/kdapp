//! Episode executor for running kdapp Episodes on the server

use anyhow::{Result};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use super::EpisodeMetadata;

/// Represents a running Episode instance
pub struct RunningEpisode {
    pub metadata: EpisodeMetadata,
    pub state: Arc<RwLock<Vec<u8>>>, // Serialized Episode state
    pub event_sender: mpsc::Sender<EpisodeEvent>,
}

/// Events emitted by Episodes
#[derive(Debug, Clone)]
pub enum EpisodeEvent {
    StateUpdate { episode_id: String, state: Vec<u8> },
    ParticipantJoined { episode_id: String, participant_id: String },
    ParticipantLeft { episode_id: String, participant_id: String },
    Error { episode_id: String, error: String },
    Completed { episode_id: String },
}

/// Executes Episodes on the server
pub struct EpisodeExecutor {
    // In the future, this will compile and dynamically load Episode code
    // For POC, we'll use pre-compiled Episodes
}

impl EpisodeExecutor {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Execute a command on an Episode
    /// This is a placeholder - actual implementation will:
    /// 1. Deserialize the Episode state
    /// 2. Apply the command
    /// 3. Serialize the new state
    /// 4. Emit events
    pub async fn execute_command(
        &self,
        episode_id: &str,
        command: Vec<u8>,
        state: &Arc<RwLock<Vec<u8>>>,
        event_sender: &mpsc::Sender<EpisodeEvent>,
    ) -> Result<()> {
        // For POC, we'll simulate command execution
        log::info!("Executing command on Episode {}", episode_id);
        
        // Lock and update state
        let mut state_guard = state.write().await;
        
        // Simulate state change
        state_guard.extend_from_slice(&command);
        
        // Emit state update event
        let _ = event_sender.send(EpisodeEvent::StateUpdate {
            episode_id: episode_id.to_string(),
            state: state_guard.clone(),
        }).await;
        
        Ok(())
    }
    
    /// Create a new Episode instance
    /// For POC, this returns a mock Episode
    pub async fn create_episode(
        &self,
        episode_type: &str,
        episode_id: String,
        creator_session: String,
    ) -> Result<RunningEpisode> {
        let metadata = EpisodeMetadata::new(
            episode_id.clone(),
            episode_type.to_string(),
            creator_session,
        );
        
        // Create event channel
        let (event_sender, _event_receiver) = mpsc::channel(100);
        
        // Initialize empty state
        let state = Arc::new(RwLock::new(Vec::new()));
        
        Ok(RunningEpisode {
            metadata,
            state,
            event_sender,
        })
    }
}

// Future: Dynamic Episode loading
#[allow(dead_code)]
struct DynamicEpisodeLoader {
    // Will compile and load Episodes at runtime
}