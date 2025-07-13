//! Natural Language Processing module
//! 
//! Converts user prompts into structured game requests.

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod simple_parser;
pub mod openrouter;

pub use simple_parser::SimpleParser;
pub use openrouter::{OpenRouterClient, Model};

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

impl ToString for GameType {
    fn to_string(&self) -> String {
        match self {
            GameType::TicTacToe => "TicTacToe".to_string(),
            GameType::Chess => "Chess".to_string(),
            GameType::Checkers => "Checkers".to_string(),
            GameType::Poker { variant } => format!("Poker_{:?}", variant),
            GameType::Custom { name } => name.clone(),
        }
    }
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
    parser: ProcessorBackend,
}

enum ProcessorBackend {
    Simple(SimpleParser),
    OpenRouter(OpenRouterClient),
}

impl Processor {
    pub fn new() -> Self {
        // Check if OpenRouter is configured
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            // Default to Claude 3 Sonnet for good balance of speed and quality
            if let Ok(client) = OpenRouterClient::new(Model::Claude3Sonnet) {
                log::info!("Using OpenRouter with Claude 3 Sonnet");
                return Self {
                    parser: ProcessorBackend::OpenRouter(client),
                };
            }
        }
        
        // Fallback to simple parser
        log::info!("Using simple pattern matching (set OPENROUTER_API_KEY to use AI)");
        Self {
            parser: ProcessorBackend::Simple(SimpleParser::new()),
        }
    }

    pub fn with_model(model: Model) -> Result<Self> {
        let client = OpenRouterClient::new(model)?;
        Ok(Self {
            parser: ProcessorBackend::OpenRouter(client),
        })
    }

    pub async fn process(&self, prompt: &str) -> Result<GameRequest> {
        match &self.parser {
            ProcessorBackend::Simple(parser) => parser.parse(prompt),
            ProcessorBackend::OpenRouter(client) => client.process_game_prompt(prompt).await,
        }
    }
}