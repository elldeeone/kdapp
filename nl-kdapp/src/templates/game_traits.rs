//! Common traits for game templates

use serde::{Serialize, Deserialize};

pub trait GameTemplate: Send + Sync {
    fn name(&self) -> &str;
    fn base_code(&self) -> String;
    fn supported_player_counts(&self) -> Vec<usize>;
}

pub trait GameCommand: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    fn validate(&self) -> bool;
}