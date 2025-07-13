//! UI command processing
//! 
//! Converts high-level UI actions into Episode-specific commands.

use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

/// High-level commands from the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiCommand {
    /// Initialize a new Episode
    Initialize {
        episode_type: String,
        config: serde_json::Value,
    },
    
    /// Game move (for game Episodes)
    GameMove {
        position: Vec<u32>,
        player_id: String,
    },
    
    /// Cast a vote (for voting Episodes)
    CastVote {
        option_id: String,
        voter_id: String,
    },
    
    /// Place a bid (for auction Episodes)
    PlaceBid {
        amount: u64,
        bidder_id: String,
    },
    
    /// Generic action for custom Episodes
    CustomAction {
        action_type: String,
        params: serde_json::Value,
    },
}

/// Episode-specific command format
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum EpisodeCommand {
    Initialize(InitCommand),
    Action(ActionCommand),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitCommand {
    pub episode_type: String,
    pub config: Vec<u8>, // Serialized config
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ActionCommand {
    pub action_type: u8,
    pub data: Vec<u8>,
}

/// Processes UI commands into Episode commands
pub struct CommandProcessor {
    // Future: command validation, authorization
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Process a UI command into Episode command bytes
    pub fn process(&self, ui_command: UiCommand) -> Result<Vec<u8>> {
        // Special handling for TicTacToe moves
        if let UiCommand::GameMove { position, .. } = &ui_command {
            if let Some(pos) = position.first() {
                // Convert position (0-8) to row/col for TTTMove
                let row = (pos / 3) as usize;
                let col = (pos % 3) as usize;
                
                use crate::episodes::tictactoe::TTTMove;
                let ttt_move = TTTMove { row, col };
                
                return borsh::to_vec(&ttt_move).map_err(Into::into);
            }
        }
        
        // Generic handling for other commands
        let episode_command = match ui_command {
            UiCommand::Initialize { episode_type, config } => {
                EpisodeCommand::Initialize(InitCommand {
                    episode_type,
                    config: serde_json::to_vec(&config)?,
                })
            }
            
            UiCommand::GameMove { position, player_id } => {
                let data = GameMoveData {
                    position,
                    player_id,
                };
                EpisodeCommand::Action(ActionCommand {
                    action_type: ActionType::GameMove as u8,
                    data: borsh::to_vec(&data)?,
                })
            }
            
            UiCommand::CastVote { option_id, voter_id } => {
                let data = VoteData {
                    option_id,
                    voter_id,
                };
                EpisodeCommand::Action(ActionCommand {
                    action_type: ActionType::Vote as u8,
                    data: borsh::to_vec(&data)?,
                })
            }
            
            UiCommand::PlaceBid { amount, bidder_id } => {
                let data = BidData {
                    amount,
                    bidder_id,
                };
                EpisodeCommand::Action(ActionCommand {
                    action_type: ActionType::Bid as u8,
                    data: borsh::to_vec(&data)?,
                })
            }
            
            UiCommand::CustomAction { action_type, params } => {
                // For custom Episodes, pass through the params
                EpisodeCommand::Action(ActionCommand {
                    action_type: ActionType::Custom as u8,
                    data: serde_json::to_vec(&params)?,
                })
            }
        };
        
        borsh::to_vec(&episode_command).map_err(Into::into)
    }
}

/// Action types for Episodes
#[repr(u8)]
enum ActionType {
    GameMove = 1,
    Vote = 2,
    Bid = 3,
    Custom = 255,
}

/// Game move data
#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct GameMoveData {
    position: Vec<u32>,
    player_id: String,
}

/// Vote data
#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct VoteData {
    option_id: String,
    voter_id: String,
}

/// Bid data
#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct BidData {
    amount: u64,
    bidder_id: String,
}