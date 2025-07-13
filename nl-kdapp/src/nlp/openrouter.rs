//! OpenRouter integration for flexible LLM selection
//! 
//! This module provides a unified interface to multiple LLMs through OpenRouter,
//! allowing easy switching between models like Claude, GPT-4, Llama, etc.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;

#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    api_key: String,
    client: Client,
    model: String,
}

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Available models on OpenRouter
#[derive(Debug, Clone)]
pub enum Model {
    Claude3Opus,
    Claude3Sonnet,
    GPT4,
    GPT4Turbo,
    Llama3_70B,
    Mixtral8x7B,
    Custom(String),
}

impl Model {
    fn to_string(&self) -> String {
        match self {
            Model::Claude3Opus => "anthropic/claude-3-opus".to_string(),
            Model::Claude3Sonnet => "anthropic/claude-3-sonnet".to_string(),
            Model::GPT4 => "openai/gpt-4".to_string(),
            Model::GPT4Turbo => "openai/gpt-4-turbo".to_string(),
            Model::Llama3_70B => "meta-llama/llama-3-70b-instruct".to_string(),
            Model::Mixtral8x7B => "mistralai/mixtral-8x7b-instruct".to_string(),
            Model::Custom(s) => s.clone(),
        }
    }
}

impl OpenRouterClient {
    pub fn new(model: Model) -> Result<Self> {
        let api_key = env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY environment variable not set")?;
        
        Ok(Self {
            api_key,
            client: Client::new(),
            model: model.to_string(),
        })
    }

    pub fn with_api_key(api_key: String, model: Model) -> Self {
        Self {
            api_key,
            client: Client::new(),
            model: model.to_string(),
        }
    }

    pub async fn process_game_prompt(&self, prompt: &str) -> Result<super::GameRequest> {
        let system_prompt = r#"
You are a game specification parser. Convert natural language game requests into structured JSON.

Output format:
{
  "game_type": "tictactoe" | "chess" | "checkers" | "poker",
  "player_count": <number>,
  "config": {
    "time_limit": <seconds or null>,
    "buy_in": <amount in KAS or null>,
    "custom_rules": [<array of rule strings>]
  }
}

Examples:
"Make me a chess game" -> {"game_type": "chess", "player_count": 2, "config": {}}
"Create poker for 6 players with 100 KAS buy-in" -> {"game_type": "poker", "player_count": 6, "config": {"buy_in": 100}}
"#;

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.2, // Low temperature for consistent parsing
            max_tokens: Some(500),
        };

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/elldeeone/kdapp")
            .header("X-Title", "Natural Language kdapp Interface")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OpenRouter API error: {}", error_text);
        }

        let api_response: OpenRouterResponse = response.json().await
            .context("Failed to parse OpenRouter response")?;

        let content = api_response.choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No response from OpenRouter"))?
            .message
            .content
            .clone();

        // Parse the JSON response into our GameRequest structure
        self.parse_llm_response(&content, prompt)
    }

    fn parse_llm_response(&self, content: &str, original_prompt: &str) -> Result<super::GameRequest> {
        // Try to extract JSON from the response
        let json_start = content.find('{');
        let json_end = content.rfind('}');
        
        let json_str = match (json_start, json_end) {
            (Some(start), Some(end)) => &content[start..=end],
            _ => content,
        };

        #[derive(Deserialize)]
        struct LLMResponse {
            game_type: String,
            player_count: Option<usize>,
            config: Option<LLMConfig>,
        }

        #[derive(Deserialize)]
        struct LLMConfig {
            time_limit: Option<u32>,
            buy_in: Option<u64>,
            custom_rules: Option<Vec<String>>,
        }

        let parsed: LLMResponse = serde_json::from_str(json_str)
            .context("Failed to parse LLM response as JSON")?;

        let game_type = match parsed.game_type.as_str() {
            "tictactoe" | "tic-tac-toe" => super::GameType::TicTacToe,
            "chess" => super::GameType::Chess,
            "checkers" => super::GameType::Checkers,
            "poker" => super::GameType::Poker { 
                variant: super::PokerVariant::TexasHoldem 
            },
            other => super::GameType::Custom { 
                name: other.to_string() 
            },
        };

        let config = super::GameConfig {
            time_limit: parsed.config.as_ref().and_then(|c| c.time_limit),
            buy_in: parsed.config.as_ref().and_then(|c| c.buy_in),
            max_players: Some(parsed.player_count.unwrap_or(2)),
            custom_rules: parsed.config
                .and_then(|c| c.custom_rules)
                .unwrap_or_default(),
        };

        Ok(super::GameRequest {
            game_type,
            player_count: parsed.player_count.unwrap_or(2),
            config,
            raw_prompt: original_prompt.to_string(),
        })
    }

    pub fn switch_model(&mut self, model: Model) {
        self.model = model.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_strings() {
        assert_eq!(Model::Claude3Opus.to_string(), "anthropic/claude-3-opus");
        assert_eq!(Model::GPT4.to_string(), "openai/gpt-4");
        assert_eq!(Model::Custom("custom/model".to_string()).to_string(), "custom/model");
    }
}