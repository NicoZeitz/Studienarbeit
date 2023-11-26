use game::Player;
use patchwork_core::Patchwork;

/// A computer player that uses the Minimax algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MinimaxPlayer {
    /// The name of the player.
    pub name: String,
}

impl MinimaxPlayer {
    /// Creates a new [`MinimaxPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        MinimaxPlayer { name: name.into() }
    }
}

impl Default for MinimaxPlayer {
    fn default() -> Self {
        Self::new("Minimax Player".to_string())
    }
}

impl Player for MinimaxPlayer {
    type Game = Patchwork;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, _game: &Self::Game) -> <Self::Game as game::Game>::Action {
        // TODO:
        todo!()
    }
}
