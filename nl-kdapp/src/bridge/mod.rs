//! Command bridge between web UI and blockchain Episodes
//! 
//! Converts UI actions into blockchain transactions and manages the flow
//! of commands from users to Episodes running on the server.

mod commands;
mod transactions;

pub use commands::{UiCommand, CommandProcessor};
pub use transactions::TransactionBuilder;

use anyhow::Result;
use std::sync::Arc;
use crate::{runtime::EpisodeManager, wallet::ServerWallet};

/// Bridge between UI and blockchain
pub struct CommandBridge {
    episode_manager: Arc<EpisodeManager>,
    server_wallet: Arc<ServerWallet>,
    player1_wallet: Option<Arc<ServerWallet>>,
    player2_wallet: Option<Arc<ServerWallet>>,
    command_processor: CommandProcessor,
    tx_builder: TransactionBuilder,
}

impl CommandBridge {
    pub fn new(
        episode_manager: Arc<EpisodeManager>,
        server_wallet: Arc<ServerWallet>,
    ) -> Self {
        let command_processor = CommandProcessor::new();
        let tx_builder = TransactionBuilder::new();
        
        Self {
            episode_manager,
            server_wallet,
            player1_wallet: None,
            player2_wallet: None,
            command_processor,
            tx_builder,
        }
    }
    
    pub fn with_player_wallets(
        mut self,
        player1_wallet: Option<Arc<ServerWallet>>,
        player2_wallet: Option<Arc<ServerWallet>>,
    ) -> Self {
        self.player1_wallet = player1_wallet;
        self.player2_wallet = player2_wallet;
        self
    }
    
    /// Process a command from the UI (JSON format)
    pub async fn process_ui_command(
        &self,
        episode_id: &str,
        command_json: serde_json::Value,
        player_number: Option<u8>,
    ) -> Result<()> {
        // Parse the JSON command
        let command: UiCommand = serde_json::from_value(command_json)?;
        
        // Select the appropriate wallet based on player number
        let wallet = match player_number {
            Some(1) if self.player1_wallet.is_some() => {
                log::info!("Using Player 1 wallet for command");
                self.player1_wallet.as_ref().unwrap()
            }
            Some(2) if self.player2_wallet.is_some() => {
                log::info!("Using Player 2 wallet for command");
                self.player2_wallet.as_ref().unwrap()
            }
            _ => {
                log::info!("Using server wallet for command");
                &self.server_wallet
            }
        };
        
        // Get session ID from the player number
        let session_id = match player_number {
            Some(n) => format!("player_{}", n),
            None => "web_session".to_string(),
        };
        
        // Convert UI command to Episode command
        let episode_command = self.command_processor.process(command)?;
        
        // Create blockchain transaction
        let tx = wallet.create_episode_transaction(
            episode_id,
            episode_command,
            &session_id,
        ).await?;
        
        // Submit transaction to Kaspa network
        // The proxy will detect it and the Engine will process it
        wallet.submit_transaction(tx).await?;
        
        log::info!("Transaction submitted to Kaspa network for Episode {} by {}", episode_id, session_id);
        
        Ok(())
    }
}