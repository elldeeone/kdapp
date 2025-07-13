//! Tic-tac-toe Episode template
//! Based on the kdapp examples/tictactoe implementation

use super::game_traits::GameTemplate;

pub struct TicTacToeTemplate;

impl TicTacToeTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl GameTemplate for TicTacToeTemplate {
    fn name(&self) -> &str {
        "tictactoe"
    }

    fn supported_player_counts(&self) -> Vec<usize> {
        vec![2]
    }

    fn base_code(&self) -> String {
        // Simplified template for POC
        // Real implementation will port from kdapp/examples/tictactoe
        r#"
use kdapp::{Episode, EpisodeEventHandler};
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct TicTacToeEpisode {
    board: [[Option<Player>; 3]; 3],
    current_player: Player,
    winner: Option<Player>,
}

#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize)]
pub enum Player {
    X,
    O,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum TicTacToeCommand {
    Move { row: usize, col: usize },
}

impl Episode for TicTacToeEpisode {
    type Command = TicTacToeCommand;
    type CommandError = String;
    type CommandRollback = ();

    fn initialize(&mut self) {
        self.current_player = Player::X;
    }

    fn execute(&mut self, command: Self::Command) -> Result<Self::CommandRollback, Self::CommandError> {
        // Game logic here
        Ok(())
    }

    fn rollback(&mut self, _rollback: Self::CommandRollback) {
        // Rollback logic here
    }
}
"#.to_string()
    }
}