//! Integration with core kdapp Engine and Proxy
//! 
//! This module manages the kdapp Engine instances and Proxy listeners
//! for Episodes created through the natural language interface.

use anyhow::Result;
use kdapp::{
    engine::{Engine, EpisodeMessage},
    episode::{Episode, EpisodeEventHandler, EpisodeId},
    generator::{PatternType, PrefixType, TransactionGenerator},
    proxy::{connect_client, run_listener},
    pki::{PubKey, generate_keypair},
};
use kaspa_consensus_core::network::{NetworkId, NetworkType};
use kaspa_addresses::{Address, Prefix, Version};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, broadcast, Mutex, RwLock};
use std::collections::HashMap;
use secp256k1::Keypair;

use crate::episodes::TicTacToe;

// For now, use the same pattern/prefix as the tic-tac-toe example
const PATTERN: PatternType = [(7, 0), (32, 1), (45, 0), (99, 1), (113, 0), (126, 1), (189, 0), (200, 1), (211, 0), (250, 1)];
const PREFIX: PrefixType = 858598618;

/// Manages kdapp Engine instances and Proxy connections
pub struct KdappManager {
    engines: Arc<Mutex<HashMap<String, EngineHandle>>>,
    network: String,
    wrpc_url: Option<String>,
    exit_signal: Arc<AtomicBool>,
    event_sender: Arc<RwLock<Option<broadcast::Sender<EpisodeEvent>>>>,
    server_pubkey: Option<PubKey>,
    player1_pubkey: Option<PubKey>,
    player2_pubkey: Option<PubKey>,
    // Store latest states for Episodes
    episode_states: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

struct EngineHandle {
    episode_type: String,
    // For now, we'll handle only TicTacToe
    // Future: make this generic over Episode types
}

impl KdappManager {
    pub fn new(
        network: String, 
        wrpc_url: Option<String>, 
        server_keypair: Option<&Keypair>,
        player1_keypair: Option<&Keypair>,
        player2_keypair: Option<&Keypair>,
    ) -> Self {
        let server_pubkey = server_keypair.map(|kp| PubKey(kp.public_key()));
        let player1_pubkey = player1_keypair.map(|kp| PubKey(kp.public_key()));
        let player2_pubkey = player2_keypair.map(|kp| PubKey(kp.public_key()));
        
        Self {
            engines: Arc::new(Mutex::new(HashMap::new())),
            network,
            wrpc_url,
            exit_signal: Arc::new(AtomicBool::new(false)),
            event_sender: Arc::new(RwLock::new(None)),
            server_pubkey,
            player1_pubkey,
            player2_pubkey,
            episode_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the kdapp proxy listener
    pub async fn start_proxy(&self) -> Result<()> {
        // Parse network type and create NetworkId
        let network_id = match self.network.as_str() {
            "mainnet" => NetworkId::new(NetworkType::Mainnet),
            "testnet-10" | "testnet" => NetworkId::with_suffix(NetworkType::Testnet, 10),
            "testnet-11" => NetworkId::with_suffix(NetworkType::Testnet, 11),
            _ => NetworkId::with_suffix(NetworkType::Testnet, 10),
        };
        
        // Connect to Kaspa node
        let kaspad = connect_client(network_id, self.wrpc_url.clone()).await?;
        
        // Create channel for proxy messages
        let (sender, receiver) = std::sync::mpsc::channel();
        
        // Create broadcast channel for state updates
        let (event_tx, _event_rx) = broadcast::channel(1000);
        
        // Store the sender for later use
        {
            let mut sender_lock = self.event_sender.write().await;
            *sender_lock = Some(event_tx.clone());
        }
        
        let episode_states = self.episode_states.clone();
        
        // Start a TicTacToe engine to handle messages
        std::thread::spawn(move || {
            let mut engine = Engine::<TicTacToe, NLHandler>::new(receiver);
            
            // Create a handler that will forward events
            let handler = NLHandler {
                event_sender: event_tx,
                episode_states,
            };
            
            log::info!("Starting TicTacToe engine");
            engine.start(vec![handler]);
        });
        
        // Start the proxy listener in a background task
        let exit_signal = self.exit_signal.clone();
        tokio::spawn(async move {
            let patterns = std::iter::once((PREFIX, (PATTERN, sender))).collect();
            run_listener(kaspad, patterns, exit_signal).await;
        });
        
        log::info!("kdapp proxy started for network: {}", self.network);
        Ok(())
    }
    
    /// Initialize a new TicTacToe Episode by sending NewEpisode message
    pub async fn initialize_episode(
        &self, 
        episode_id: u32,
        server_wallet: &crate::wallet::ServerWallet,
    ) -> Result<()> {
        log::info!("Initializing TicTacToe Episode: {}", episode_id);
        
        // For TicTacToe, we need two players
        // Use test player keys if available, otherwise use server key and generate dummy
        let player1 = self.player1_pubkey
            .or(self.server_pubkey)
            .ok_or_else(|| anyhow::anyhow!("No player 1 key available"))?;
        
        let player2 = self.player2_pubkey
            .unwrap_or_else(|| {
                // Generate dummy player 2 if not configured
                let (_, pk) = generate_keypair();
                log::warn!("No PLAYER2_PRIVATE_KEY configured, using dummy player 2");
                pk
            });
        
        // Create NewEpisode message
        let new_episode = EpisodeMessage::<TicTacToe>::NewEpisode {
            episode_id,
            participants: vec![player1, player2],
        };
        
        // Create and submit the initialization transaction
        let tx = server_wallet.create_new_episode_transaction(
            episode_id,
            vec![player1, player2],
        ).await?;
        
        server_wallet.submit_transaction(tx).await?;
        
        log::info!("Submitted NewEpisode transaction for Episode {}", episode_id);
        Ok(())
    }
    
    pub async fn shutdown(&self) {
        self.exit_signal.store(true, Ordering::Relaxed);
    }
}

/// Event handler for NL-created Episodes
struct NLHandler {
    event_sender: broadcast::Sender<EpisodeEvent>,
    episode_states: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

#[derive(Debug, Clone)]
pub enum EpisodeEvent {
    StateUpdate { episode_id: String, state: Vec<u8> },
    GameOver { episode_id: String, winner: Option<String> },
}

impl KdappManager {
    /// Get the event sender for subscribing to Episode events
    pub async fn get_event_sender(&self) -> Option<broadcast::Sender<EpisodeEvent>> {
        self.event_sender.read().await.clone()
    }
    
    /// Get the current state of an Episode
    pub async fn get_episode_state(&self, episode_id: &str) -> Option<Vec<u8>> {
        self.episode_states.read().await.get(episode_id).cloned()
    }
}

impl EpisodeEventHandler<TicTacToe> for NLHandler {
    fn on_initialize(&self, episode_id: EpisodeId, episode: &TicTacToe) {
        log::info!("Episode {} initialized", episode_id);
        
        // Send initial state
        let state = episode.poll();
        log::debug!("Sending initial state for Episode {}: {:?}", episode_id, state);
        let state_bytes = borsh::to_vec(&state).unwrap_or_default();
        
        // Store the state
        {
            let mut states = self.episode_states.blocking_write();
            states.insert(episode_id.to_string(), state_bytes.clone());
        }
        
        // Try to broadcast (may fail if no receivers yet)
        let _ = self.event_sender.send(EpisodeEvent::StateUpdate {
            episode_id: episode_id.to_string(),
            state: state_bytes,
        });
    }

    fn on_command(
        &self,
        episode_id: EpisodeId,
        episode: &TicTacToe,
        _cmd: &<TicTacToe as Episode>::Command,
        _authorization: Option<PubKey>,
        _metadata: &kdapp::episode::PayloadMetadata,
    ) {
        log::info!("Episode {} command executed", episode_id);
        
        // Send updated state
        let state = episode.poll();
        log::debug!("Sending updated state for Episode {}: {:?}", episode_id, state);
        let state_bytes = borsh::to_vec(&state).unwrap_or_default();
        
        // Store the state
        {
            let mut states = self.episode_states.blocking_write();
            states.insert(episode_id.to_string(), state_bytes.clone());
        }
        
        // Broadcast to all receivers
        let _ = self.event_sender.send(EpisodeEvent::StateUpdate {
            episode_id: episode_id.to_string(),
            state: state_bytes,
        });
        
        // Check if game is over
        if !matches!(state.status, crate::episodes::tictactoe::TTTGameStatus::InProgress(_)) {
            let winner = match state.status {
                crate::episodes::tictactoe::TTTGameStatus::Winner(pk) => Some(format!("{:?}", pk)),
                crate::episodes::tictactoe::TTTGameStatus::Draw => Some("Draw".to_string()),
                _ => None,
            };
            
            let _ = self.event_sender.send(EpisodeEvent::GameOver {
                episode_id: episode_id.to_string(),
                winner,
            });
        }
    }

    fn on_rollback(&self, episode_id: EpisodeId, episode: &TicTacToe) {
        log::warn!("Episode {} rolled back", episode_id);
        
        // Send updated state after rollback
        let state = episode.poll();
        let state_bytes = borsh::to_vec(&state).unwrap_or_default();
        
        // Store the state
        {
            let mut states = self.episode_states.blocking_write();
            states.insert(episode_id.to_string(), state_bytes.clone());
        }
        
        let _ = self.event_sender.send(EpisodeEvent::StateUpdate {
            episode_id: episode_id.to_string(),
            state: state_bytes,
        });
    }
}