//! Template engine for selecting appropriate Episode templates

use anyhow::{Result, bail};
use crate::nlp::GameType;
use crate::templates::TemplateRegistry;

pub struct TemplateEngine {
    registry: TemplateRegistry,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            registry: TemplateRegistry::new(),
        }
    }

    pub fn get_template(&self, game_type: &GameType) -> Result<String> {
        match game_type {
            GameType::TicTacToe => {
                if let Some(template) = self.registry.get("tictactoe") {
                    Ok(template.base_code())
                } else {
                    bail!("TicTacToe template not found")
                }
            }
            _ => bail!("Game type not yet implemented"),
        }
    }
}