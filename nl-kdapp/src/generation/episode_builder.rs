//! Episode builder that customizes templates based on game requests

use anyhow::Result;
use crate::nlp::GameRequest;

pub struct EpisodeBuilder;

impl EpisodeBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, template: String, request: &GameRequest) -> Result<String> {
        // For POC, just return the template with minimal modifications
        // In future, this will do sophisticated code generation
        
        let code = template
            .replace("{{PLAYER_COUNT}}", &request.player_count.to_string())
            .replace("{{GAME_NAME}}", &format!("{:?}", request.game_type));
        
        Ok(code)
    }
}