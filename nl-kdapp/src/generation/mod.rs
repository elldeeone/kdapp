//! Code Generation module
//! 
//! Generates kdapp Episode implementations from structured game requests.

use anyhow::Result;
use crate::nlp::{GameRequest, GameType};

pub mod template_engine;
pub mod episode_builder;

pub use template_engine::TemplateEngine;
pub use episode_builder::EpisodeBuilder;

/// Main code generator that produces kdapp Episodes
pub struct Generator {
    template_engine: TemplateEngine,
    episode_builder: EpisodeBuilder,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            episode_builder: EpisodeBuilder::new(),
        }
    }

    pub async fn generate(&self, request: &GameRequest) -> Result<GeneratedCode> {
        // Select appropriate template based on game type
        let template = self.template_engine.get_template(&request.game_type)?;
        
        // Build the Episode code
        let code = self.episode_builder.build(template, request)?;
        
        Ok(GeneratedCode {
            episode_code: code,
            game_type: request.game_type.clone(),
            deployment_ready: true,
        })
    }
}

pub struct GeneratedCode {
    pub episode_code: String,
    pub game_type: GameType,
    pub deployment_ready: bool,
}