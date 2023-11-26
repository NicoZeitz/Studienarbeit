use game::Player;
use patchwork_core::Patchwork;

/// A computer player that uses the AlphaZero algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlphaZeroPlayer {
    /// The name of the player.
    pub name: String,
}

impl AlphaZeroPlayer {
    /// Creates a new [`AlphaZeroPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        AlphaZeroPlayer { name: name.into() }
    }
}

impl Default for AlphaZeroPlayer {
    fn default() -> Self {
        Self::new("AlphaZero Player".to_string())
    }
}

impl Player for AlphaZeroPlayer {
    type Game = Patchwork;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, _game: &Self::Game) -> <Self::Game as game::Game>::Action {
        // TODO:
        todo!()
    }
}
