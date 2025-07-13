//! Simple pattern-based parser for POC
//! 
//! This will be replaced with AI model integration in later phases.

use anyhow::{Result, bail};
use super::{GameRequest, GameType, GameConfig, PokerVariant};

pub struct SimpleParser;

impl SimpleParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, prompt: &str) -> Result<GameRequest> {
        let lower = prompt.to_lowercase();
        
        // Detect game type
        let game_type = if lower.contains("tic-tac-toe") || lower.contains("tic tac toe") {
            GameType::TicTacToe
        } else if lower.contains("chess") {
            GameType::Chess
        } else if lower.contains("checkers") {
            GameType::Checkers
        } else if lower.contains("poker") || lower.contains("hold'em") || lower.contains("holdem") {
            let variant = if lower.contains("omaha") {
                PokerVariant::Omaha
            } else {
                PokerVariant::TexasHoldem
            };
            GameType::Poker { variant }
        } else {
            bail!("Unsupported game type. Try: tic-tac-toe, chess, checkers, or poker");
        };

        // Extract player count
        let player_count = self.extract_player_count(&lower).unwrap_or(2);

        // Extract config
        let config = self.extract_config(&lower);

        Ok(GameRequest {
            game_type,
            player_count,
            config,
            raw_prompt: prompt.to_string(),
        })
    }

    fn extract_player_count(&self, text: &str) -> Option<usize> {
        // Look for patterns like "for 4 players" or "4-player"
        let patterns = [
            r"(\d+)[\s-]?players?",
            r"for\s+(\d+)",
            r"(\d+)[\s-]?person",
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(text) {
                    if let Some(num_str) = captures.get(1) {
                        if let Ok(num) = num_str.as_str().parse::<usize>() {
                            return Some(num);
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_config(&self, text: &str) -> GameConfig {
        let mut config = GameConfig::default();

        // Extract time limit
        if let Ok(re) = regex::Regex::new(r"(\d+)\s*minute") {
            if let Some(captures) = re.captures(text) {
                if let Some(num_str) = captures.get(1) {
                    if let Ok(minutes) = num_str.as_str().parse::<u32>() {
                        config.time_limit = Some(minutes * 60);
                    }
                }
            }
        }

        // Extract buy-in
        if let Ok(re) = regex::Regex::new(r"(\d+)\s*kas") {
            if let Some(captures) = re.captures(text) {
                if let Some(num_str) = captures.get(1) {
                    if let Ok(kas) = num_str.as_str().parse::<u64>() {
                        config.buy_in = Some(kas);
                    }
                }
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tictactoe() {
        let parser = SimpleParser::new();
        let request = parser.parse("Make me a tic-tac-toe game").unwrap();
        matches!(request.game_type, GameType::TicTacToe);
    }

    #[test]
    fn test_parse_poker_with_players() {
        let parser = SimpleParser::new();
        let request = parser.parse("Create a poker game for 6 players").unwrap();
        matches!(request.game_type, GameType::Poker { .. });
        assert_eq!(request.player_count, 6);
    }
}