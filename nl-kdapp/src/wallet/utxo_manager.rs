//! UTXO management for server wallet

use anyhow::Result;
use kaspa_consensus_core::tx::{TransactionOutpoint, ScriptPublicKey};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Utxo {
    pub outpoint: TransactionOutpoint,
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub is_spent: bool,
}

pub struct UtxoManager {
    pub utxos: HashMap<TransactionOutpoint, Utxo>,
}

impl UtxoManager {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }
    
    /// Update UTXO set from RPC response
    pub fn update_utxos(&mut self, response: Vec<kaspa_rpc_core::GetUtxosByAddressesResponse>) -> Result<()> {
        // Clear existing UTXOs
        self.utxos.clear();
        
        // For POC, we'll handle a simplified case
        // In production, this would parse the actual RPC response
        log::info!("UTXO update requested - POC implementation");
        Ok(())
    }
    
    /// Get available (unspent) UTXOs
    pub fn get_available_utxos(&self) -> Result<Vec<Utxo>> {
        Ok(self.utxos
            .values()
            .filter(|utxo| !utxo.is_spent)
            .cloned()
            .collect())
    }
    
    /// Mark UTXO as spent
    pub fn mark_spent(&mut self, outpoint: &TransactionOutpoint) -> Result<()> {
        if let Some(utxo) = self.utxos.get_mut(outpoint) {
            utxo.is_spent = true;
            Ok(())
        } else {
            anyhow::bail!("UTXO not found: {:?}", outpoint)
        }
    }
    
    /// Get total balance
    pub fn get_total_balance(&self) -> u64 {
        self.utxos
            .values()
            .filter(|utxo| !utxo.is_spent)
            .map(|utxo| utxo.amount)
            .sum()
    }
}