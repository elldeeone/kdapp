//! Natural Language Processing module
//! 
//! Converts user prompts into structured game requests.

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod simple_parser;

pub use simple_parser::SimpleParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRequest {
    pub game_type: GameType,
    pub player_count: usize,
    pub config: GameConfig,
    pub raw_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameType {
    TicTacToe,
    Chess,
    Checkers,
    Poker { variant: PokerVariant },
    Custom { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PokerVariant {
    TexasHoldem,
    Omaha,
    FiveCard,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameConfig {
    pub time_limit: Option<u32>, // seconds per turn
    pub buy_in: Option<u64>,      // in KAS
    pub max_players: Option<usize>,
    pub custom_rules: Vec<String>,
}

/// Main NLP processor that converts natural language to structured requests
pub struct Processor {
    parser: SimpleParser,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            parser: SimpleParser::new(),
        }
    }

    pub fn process(&self, prompt: &str) -> Result<GameRequest> {
        // For POC, use simple pattern matching
        // Later, integrate AI model here
        self.parser.parse(prompt)
    }
}