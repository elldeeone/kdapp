//! Server wallet that funds all Episode transactions during POC
//! 
//! This is a temporary solution for testnet. When moving to mainnet,
//! users will connect their own wallets (e.g., Kaspa Kastle Web Wallet).

use anyhow::{Result, Context};
use kaspa_consensus_core::signing::secp256k1::XOnlyPublicKey;
use kaspa_wallet_keys::{keypair::Keypair, privatekey::PrivateKey};
use kaspa_consensus_core::tx::{Transaction, TransactionOutput, TransactionInput};
use kaspa_consensus_core::hashing::sighash::SigHashReusedValues;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use super::{UtxoManager, RateLimiter};

pub struct ServerWallet {
    keypair: Keypair,
    pub address: String,
    utxo_manager: Arc<Mutex<UtxoManager>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    network_id: kaspa_addresses::NetworkId,
}

impl ServerWallet {
    /// Create server wallet from environment variables
    pub async fn from_env() -> Result<Self> {
        let private_key_str = env::var("SERVER_PRIVATE_KEY")
            .context("SERVER_PRIVATE_KEY not set in environment")?;
        
        let network = env::var("KASPA_NETWORK")
            .unwrap_or_else(|_| "testnet-10".to_string());
        
        let network_id = match network.as_str() {
            "mainnet" => kaspa_addresses::NetworkId::Mainnet,
            "testnet-10" | "testnet" => kaspa_addresses::NetworkId::Testnet,
            "devnet" => kaspa_addresses::NetworkId::Devnet,
            "simnet" => kaspa_addresses::NetworkId::Simnet,
            _ => kaspa_addresses::NetworkId::Testnet,
        };
        
        let private_key = PrivateKey::try_from(private_key_str.as_str())
            .context("Invalid private key format")?;
        
        let keypair = Keypair::from_private_key(private_key);
        let public_key = keypair.public_key();
        
        // Convert to address
        let address = kaspa_addresses::Address::new(
            kaspa_addresses::Prefix::try_from(network_id)?,
            kaspa_addresses::Version::PubKey,
            public_key.as_bytes(),
        );
        
        let utxo_manager = Arc::new(Mutex::new(UtxoManager::new()));
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new()));
        
        log::info!("Server wallet initialized for {} on {}", address, network);
        
        Ok(Self {
            keypair,
            address: address.to_string(),
            utxo_manager,
            rate_limiter,
            network_id,
        })
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
        
        // Create transaction
        let mut tx = Transaction::new(
            0, // version
            vec![TransactionInput {
                previous_outpoint: utxo.outpoint.clone(),
                signature_script: vec![],
                sequence: u64::MAX,
                sig_op_count: 1,
            }],
            vec![],
            0, // lock time
            kaspa_consensus_core::subnets::SUBNETWORK_ID_NATIVE,
            0, // gas
            vec![],
        );
        
        // Add output with Episode command payload
        let fee = 5000; // 5000 sompi fee for POC
        let change_amount = utxo.amount.saturating_sub(fee);
        
        tx.outputs.push(TransactionOutput {
            value: 0,
            script_public_key: kaspa_txscript::pay_to_script_hash_script(&command_payload),
        });
        
        // Add change output back to server wallet
        if change_amount > 0 {
            tx.outputs.push(TransactionOutput {
                value: change_amount,
                script_public_key: kaspa_txscript::pay_to_address_script(&self.address.parse()?),
            });
        }
        
        // Sign the transaction
        let mut reused_values = SigHashReusedValues::new();
        let sig_hash = kaspa_consensus_core::hashing::sighash::calc_schnorr_signature_hash(
            &tx,
            0,
            kaspa_consensus_core::hashing::sighash_type::SIG_HASH_ALL,
            &utxo.script_public_key,
            &mut reused_values,
        );
        
        let signature = self.keypair.sign_schnorr(sig_hash)?;
        
        // Create signature script
        tx.inputs[0].signature_script = kaspa_txscript::builder::Builder::new()
            .add_data(&signature.to_vec())?
            .add_data(&self.keypair.public_key().to_vec())?
            .drain();
        
        log::info!(
            "Created transaction for Episode {} from session {}",
            episode_id,
            session_id
        );
        
        Ok(tx)
    }
    
    /// Get server wallet balance (for monitoring)
    pub async fn get_balance(&self) -> Result<u64> {
        let manager = self.utxo_manager.lock().await;
        Ok(manager.get_total_balance())
    }
    
    /// Update UTXO set from network
    pub async fn refresh_utxos(&self, utxos: Vec<kaspa_rpc_core::api::UtxosByAddressesEntry>) -> Result<()> {
        let mut manager = self.utxo_manager.lock().await;
        manager.update_utxos(utxos)
    }
}