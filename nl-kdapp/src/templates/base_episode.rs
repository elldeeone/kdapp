//! Base Episode template structure
//! Inspired by kasperience's kaspa-auth Episode patterns

pub struct BaseEpisode {
    pub name: String,
    pub player_count: usize,
}

impl BaseEpisode {
    pub fn new(name: String, player_count: usize) -> Self {
        Self {
            name,
            player_count,
        }
    }
}