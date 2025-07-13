//! Server wallet module for POC transaction funding
//! 
//! During POC phase, the server pays for all Episode transactions.
//! Future versions will use user wallets for authentication and payment.

mod server_wallet;
mod utxo_manager;
mod rate_limiter;

pub use server_wallet::ServerWallet;
pub use utxo_manager::{UtxoManager, Utxo};
pub use rate_limiter::{RateLimiter, SessionLimits};

use anyhow::Result;
use std::env;

/// Initialize the server wallet for POC usage
pub async fn init_server_wallet() -> Result<ServerWallet> {
    ServerWallet::from_env().await
}

/// Initialize test player wallets if configured
pub async fn init_test_player_wallets() -> Result<(Option<ServerWallet>, Option<ServerWallet>)> {
    let player1 = if env::var("PLAYER1_PRIVATE_KEY").is_ok() {
        Some(ServerWallet::from_env_with_key("PLAYER1_PRIVATE_KEY").await?)
    } else {
        None
    };
    
    let player2 = if env::var("PLAYER2_PRIVATE_KEY").is_ok() {
        Some(ServerWallet::from_env_with_key("PLAYER2_PRIVATE_KEY").await?)
    } else {
        None
    };
    
    Ok((player1, player2))
}