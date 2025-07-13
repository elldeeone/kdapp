//! UTXO management for server wallet

use anyhow::Result;
use kaspa_consensus_core::tx::TransactionOutpoint;
use kaspa_txscript::ScriptPublicKey;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Utxo {
    pub outpoint: TransactionOutpoint,
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub is_spent: bool,
}

pub struct UtxoManager {
    utxos: HashMap<TransactionOutpoint, Utxo>,
}

impl UtxoManager {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }
    
    /// Update UTXO set from RPC response
    pub fn update_utxos(&mut self, entries: Vec<kaspa_rpc_core::api::UtxosByAddressesEntry>) -> Result<()> {
        // Clear existing UTXOs
        self.utxos.clear();
        
        // Add new UTXOs
        for entry in entries {
            let outpoint = TransactionOutpoint::new(
                entry.outpoint.transaction_id,
                entry.outpoint.index,
            );
            
            let utxo = Utxo {
                outpoint: outpoint.clone(),
                amount: entry.utxo_entry.amount,
                script_public_key: entry.utxo_entry.script_public_key,
                is_spent: false,
            };
            
            self.utxos.insert(outpoint, utxo);
        }
        
        log::info!("Updated UTXO set: {} UTXOs available", self.utxos.len());
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