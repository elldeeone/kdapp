//! Server wallet that funds all Episode transactions during POC
//! 
//! This is a temporary solution for testnet. When moving to mainnet,
//! users will connect their own wallets (e.g., Kaspa Kastle Web Wallet).

use anyhow::{Result, Context};
use kaspa_consensus_core::tx::{Transaction, TransactionOutpoint, UtxoEntry};
use kaspa_consensus_core::network::{NetworkType, NetworkId};
use kaspa_addresses::{Address, Prefix, Version};
use kaspa_rpc_core::api::rpc::RpcApi;
use kdapp::{
    generator::{TransactionGenerator, PatternType, PrefixType, get_first_output_utxo},
    proxy::connect_client,
    engine::EpisodeMessage,
    episode::EpisodeId,
    pki::{PubKey, sign_message, to_message},
};
use secp256k1::Keypair;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use super::{UtxoManager, RateLimiter, Utxo};

// Use the same pattern/prefix as defined in kdapp_integration
const PATTERN: PatternType = [(7, 0), (32, 1), (45, 0), (99, 1), (113, 0), (126, 1), (189, 0), (200, 1), (211, 0), (250, 1)];
const PREFIX: PrefixType = 858598618;
const FEE: u64 = 5000;

pub struct ServerWallet {
    keypair: Keypair,
    pub address: String,
    utxo_manager: Arc<Mutex<UtxoManager>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    network_id: NetworkType,
    kaspad: Arc<Mutex<Option<Arc<dyn RpcApi>>>>,
}

impl ServerWallet {
    /// Create server wallet from environment variables
    pub async fn from_env() -> Result<Self> {
        Self::from_env_with_key("SERVER_PRIVATE_KEY").await
    }
    
    /// Create wallet from specific environment variable key
    pub async fn from_env_with_key(key_name: &str) -> Result<Self> {
        let private_key_str = env::var(key_name)
            .with_context(|| format!("{} not set in environment", key_name))?;
        
        let network = env::var("KASPA_NETWORK")
            .unwrap_or_else(|_| "testnet-10".to_string());
        
        let network_id = match network.as_str() {
            "mainnet" => NetworkType::Mainnet,
            "testnet-10" | "testnet" => NetworkType::Testnet,
            "devnet" => NetworkType::Devnet,
            "simnet" => NetworkType::Simnet,
            _ => NetworkType::Testnet,
        };
        
        // Parse hex private key
        let mut private_key_bytes = [0u8; 32];
        faster_hex::hex_decode(private_key_str.as_bytes(), &mut private_key_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid hex private key in {}", key_name))?;
        
        let keypair = Keypair::from_seckey_slice(secp256k1::SECP256K1, &private_key_bytes)
            .context("Invalid private key")?;
        
        // Convert to address
        let public_key = keypair.x_only_public_key().0;
        let address = Address::new(
            Prefix::try_from(network_id)?,
            Version::PubKey,
            &public_key.serialize(),
        );
        
        let utxo_manager = Arc::new(Mutex::new(UtxoManager::new()));
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new()));
        
        log::info!("Wallet ({}) initialized for {} on {}", key_name, address, network);
        
        let wallet = Self {
            keypair,
            address: address.to_string(),
            utxo_manager,
            rate_limiter,
            network_id,
            kaspad: Arc::new(Mutex::new(None)),
        };
        
        // Fetch initial UTXOs from network
        wallet.refresh_utxos_from_network().await?;
        
        Ok(wallet)
    }
    
    /// Refresh UTXOs from the Kaspa network
    async fn refresh_utxos_from_network(&self) -> Result<()> {
        // Parse network ID
        let network_id = match self.network_id {
            NetworkType::Mainnet => NetworkId::new(NetworkType::Mainnet),
            NetworkType::Testnet => NetworkId::with_suffix(NetworkType::Testnet, 10),
            NetworkType::Devnet => NetworkId::new(NetworkType::Devnet),
            NetworkType::Simnet => NetworkId::new(NetworkType::Simnet),
        };
        
        // Connect to Kaspa node
        let wrpc_url = env::var("KASPA_WRPC_URL").ok();
        let kaspad = Arc::new(connect_client(network_id, wrpc_url).await?);
        
        // Store the client for later use
        {
            let mut client_lock = self.kaspad.lock().await;
            *client_lock = Some(kaspad.clone());
        }
        
        // Parse address
        let address = Address::try_from(self.address.as_str())?;
        
        // Get UTXOs from network
        let entries = kaspad.get_utxos_by_addresses(vec![address]).await?;
        
        if entries.is_empty() {
            log::warn!("No UTXOs found for server wallet address: {}", self.address);
            log::warn!("Please fund the address with testnet KAS");
            anyhow::bail!("No UTXOs available in server wallet");
        }
        
        // Update UTXO manager
        let mut manager = self.utxo_manager.lock().await;
        manager.utxos.clear();
        
        let mut total_balance = 0u64;
        for entry in entries {
            let outpoint = TransactionOutpoint::from(entry.outpoint);
            let utxo_entry = UtxoEntry::from(entry.utxo_entry);
            
            let utxo = Utxo {
                outpoint: outpoint.clone(),
                amount: utxo_entry.amount,
                script_public_key: utxo_entry.script_public_key.clone(),
                is_spent: false,
            };
            
            total_balance += utxo.amount;
            manager.utxos.insert(outpoint, utxo);
        }
        
        let kas_balance = total_balance as f64 / 100_000_000.0;
        log::info!("Server wallet UTXOs refreshed - balance: {} KAS", kas_balance);
        
        Ok(())
    }
    
    /// Create a transaction on behalf of a user session
    pub async fn create_episode_transaction(
        &self,
        episode_id: &str,
        command_payload: Vec<u8>,
        session_id: &str,
    ) -> Result<Transaction> {
        // Check rate limits
        {
            let mut limiter = self.rate_limiter.lock().await;
            limiter.check_and_update(session_id)?;
        }
        
        // Get available UTXOs
        let utxos = {
            let manager = self.utxo_manager.lock().await;
            manager.get_available_utxos()?
        };
        
        if utxos.is_empty() {
            anyhow::bail!("No UTXOs available in server wallet");
        }
        
        // For POC, use first UTXO
        let utxo = &utxos[0];
        let utxo_tuple = (utxo.outpoint.clone(), UtxoEntry::new(
            utxo.amount,
            utxo.script_public_key.clone(),
            0, // block daa score
            false, // is_coinbase
        ));
        
        // Create TransactionGenerator
        let generator = TransactionGenerator::new(self.keypair.clone(), PATTERN, PREFIX);
        
        // Parse address
        let address = Address::try_from(self.address.as_str())?;
        
        // Parse the numeric Episode ID from the string
        let episode_id_num: u32 = episode_id.parse()
            .map_err(|_| anyhow::anyhow!("Invalid episode ID: {}", episode_id))?;
        
        // Create a SignedCommand message with the payload
        // This wraps our command payload in the kdapp EpisodeMessage format
        use crate::episodes::TicTacToe;
        let cmd: <TicTacToe as kdapp::episode::Episode>::Command = borsh::from_slice(&command_payload)?;
        
        // Sign the command with the server's private key
        let sk = self.keypair.secret_key();
        let pk = PubKey(self.keypair.public_key());
        let msg = to_message(&cmd);
        let sig = sign_message(&sk, &msg);
        
        let episode_message = EpisodeMessage::<TicTacToe>::SignedCommand {
            episode_id: episode_id_num,
            cmd,
            pubkey: pk,
            sig,
        };
        
        // Build the transaction using TransactionGenerator
        let tx = generator.build_command_transaction(
            utxo_tuple,
            &address,
            &episode_message,
            FEE,
        );
        
        log::info!(
            "Created kdapp transaction {} for Episode {} from session {}",
            tx.id(),
            episode_id,
            session_id
        );
        
        Ok(tx.as_ref().clone())
    }
    
    /// Get server wallet balance (for monitoring)
    pub async fn get_balance(&self) -> Result<u64> {
        let manager = self.utxo_manager.lock().await;
        Ok(manager.get_total_balance())
    }
    
    /// Get the keypair (for kdapp integration)
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
    
    /// Create a NewEpisode transaction to initialize an Episode
    pub async fn create_new_episode_transaction(
        &self,
        episode_id: u32,
        participants: Vec<kdapp::pki::PubKey>,
    ) -> Result<Transaction> {
        // Get available UTXOs
        let utxos = {
            let manager = self.utxo_manager.lock().await;
            manager.get_available_utxos()?
        };
        
        if utxos.is_empty() {
            anyhow::bail!("No UTXOs available in server wallet");
        }
        
        // For POC, use first UTXO
        let utxo = &utxos[0];
        let utxo_tuple = (utxo.outpoint.clone(), UtxoEntry::new(
            utxo.amount,
            utxo.script_public_key.clone(),
            0, // block daa score
            false, // is_coinbase
        ));
        
        // Create TransactionGenerator
        let generator = TransactionGenerator::new(self.keypair.clone(), PATTERN, PREFIX);
        
        // Parse address
        let address = Address::try_from(self.address.as_str())?;
        
        // Create NewEpisode message
        use crate::episodes::TicTacToe;
        let episode_message = EpisodeMessage::<TicTacToe>::NewEpisode {
            episode_id,
            participants,
        };
        
        // Build the transaction using TransactionGenerator
        let tx = generator.build_command_transaction(
            utxo_tuple,
            &address,
            &episode_message,
            FEE,
        );
        
        log::info!(
            "Created NewEpisode transaction {} for Episode {}",
            tx.id(),
            episode_id
        );
        
        Ok(tx.as_ref().clone())
    }
    
    /// Submit a transaction to the Kaspa network
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<()> {
        let kaspad_lock = self.kaspad.lock().await;
        let kaspad = kaspad_lock.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Kaspa client not connected"))?;
        
        let tx_id = tx.id();
        log::info!("Submitting transaction {} to Kaspa network", tx_id);
        
        // Convert to RPC transaction and submit
        kaspad.submit_transaction((&tx).into(), false).await?;
        
        log::info!("Transaction {} submitted successfully", tx_id);
        
        // Update UTXO set: mark old one as spent and add new output
        if let Some(input) = tx.inputs.first() {
            let mut manager = self.utxo_manager.lock().await;
            let _ = manager.mark_spent(&input.previous_outpoint);
            
            // kdapp transactions have a single output that becomes the new UTXO
            let (new_outpoint, new_entry) = get_first_output_utxo(&tx);
            let new_utxo = Utxo {
                outpoint: new_outpoint,
                amount: new_entry.amount,
                script_public_key: new_entry.script_public_key,
                is_spent: false,
            };
            manager.utxos.insert(new_utxo.outpoint.clone(), new_utxo);
            log::debug!("Added new UTXO from tx {}: {} sompi", tx_id, new_entry.amount);
        }
        
        Ok(())
    }
}