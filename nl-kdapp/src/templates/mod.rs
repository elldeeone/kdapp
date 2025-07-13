//! Episode Templates
//! 
//! Pre-built Episode patterns inspired by kasperience's kaspa-auth structure.
//! These templates are used as the basis for AI-generated Episodes.

use anyhow::Result;

pub mod base_episode;
pub mod tictactoe;
pub mod game_traits;

pub use base_episode::BaseEpisode;
pub use game_traits::{GameTemplate, GameCommand};

/// Registry of available Episode templates
pub struct TemplateRegistry {
    templates: std::collections::HashMap<String, Box<dyn GameTemplate>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut templates = std::collections::HashMap::new();
        
        // Register built-in templates
        templates.insert("tictactoe".to_string(), Box::new(tictactoe::TicTacToeTemplate::new()) as Box<dyn GameTemplate>);
        
        Self { templates }
    }

    pub fn get(&self, name: &str) -> Option<&dyn GameTemplate> {
        self.templates.get(name).map(|t| t.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}