//! Server wallet module for POC transaction funding
//! 
//! During POC phase, the server pays for all Episode transactions.
//! Future versions will use user wallets for authentication and payment.

mod server_wallet;
mod utxo_manager;
mod rate_limiter;

pub use server_wallet::ServerWallet;
pub use utxo_manager::UtxoManager;
pub use rate_limiter::{RateLimiter, SessionLimits};

use anyhow::Result;

/// Initialize the server wallet for POC usage
pub async fn init_server_wallet() -> Result<ServerWallet> {
    ServerWallet::from_env().await
}