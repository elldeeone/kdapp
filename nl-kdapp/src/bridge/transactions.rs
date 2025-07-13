//! Transaction building for Episode commands
//! 
//! Creates Kaspa transactions that embed Episode commands using kdapp patterns.

use anyhow::Result;
use kaspa_consensus_core::tx::{Transaction, TransactionOutput};

/// Builds transactions for Episode commands
pub struct TransactionBuilder {
    // Future: pattern configuration, nonce management
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Create a transaction with Episode command payload
    pub fn build_command_transaction(
        &self,
        episode_id: &str,
        command_data: Vec<u8>,
    ) -> Result<TransactionOutput> {
        // For POC, we create a simple output with the command data
        // In production, this would use kdapp's TransactionGenerator patterns
        
        // Create script that embeds the command
        let script = self.create_command_script(episode_id, command_data)?;
        
        Ok(TransactionOutput {
            value: 0, // Data-only output
            script_public_key: script,
        })
    }
    
    /// Create script embedding Episode command
    fn create_command_script(
        &self,
        episode_id: &str,
        command_data: Vec<u8>,
    ) -> Result<kaspa_consensus_core::tx::ScriptPublicKey> {
        // Build script with Episode ID and command
        let mut payload = Vec::new();
        
        // Add Episode ID marker
        payload.extend_from_slice(b"KDAPP:");
        payload.extend_from_slice(episode_id.as_bytes());
        payload.push(b':');
        
        // Add command data
        payload.extend_from_slice(&command_data);
        
        // Create OP_RETURN script
        Ok(kaspa_txscript::pay_to_script_hash_script(&payload))
    }
    
    /// Extract Episode command from transaction
    pub fn extract_command(tx: &Transaction) -> Option<Vec<u8>> {
        // Look for OP_RETURN output with kdapp pattern
        for output in &tx.outputs {
            if output.value == 0 {
                // Check if this is a kdapp command output
                // For POC, simplified extraction
                return Some(output.script_public_key.script().to_vec());
            }
        }
        None
    }
}