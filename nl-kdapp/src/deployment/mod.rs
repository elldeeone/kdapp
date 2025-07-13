//! Deployment module
//! 
//! Handles compilation and deployment of generated Episodes to Kaspa network.

use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

pub mod compiler;
pub mod deployer;

pub use compiler::Compiler;
pub use deployer::Deployer;

/// Manages the deployment pipeline for generated Episodes
pub struct Manager {
    network: String,
    wrpc_url: Option<String>,
    compiler: Compiler,
    deployer: Deployer,
}

impl Manager {
    pub fn new(network: &str, wrpc_url: Option<String>) -> Self {
        Self {
            network: network.to_string(),
            wrpc_url: wrpc_url.clone(),
            compiler: Compiler::new(),
            deployer: Deployer::new(network, wrpc_url),
        }
    }

    pub async fn deploy(&self, code: &str) -> Result<DeploymentResult> {
        // Generate unique ID for this deployment
        let deployment_id = Uuid::new_v4().to_string();
        
        // Compile the code
        let compiled_path = self.compiler.compile(&deployment_id, code).await?;
        
        // Deploy to network
        let episode_info = self.deployer.deploy(&deployment_id, &compiled_path).await?;
        
        Ok(DeploymentResult {
            deployment_id,
            episode_id: episode_info.id,
            share_link: episode_info.share_link,
            network: self.network.clone(),
        })
    }
}

pub struct DeploymentResult {
    pub deployment_id: String,
    pub episode_id: String,
    pub share_link: String,
    pub network: String,
}

pub struct EpisodeInfo {
    pub id: String,
    pub share_link: String,
}