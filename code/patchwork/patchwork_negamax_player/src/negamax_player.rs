use game::Player;
use patchwork_core::Patchwork;

/// A computer player that uses the Negamax algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NegamaxPlayer {
    /// The name of the player.
    pub name: String,
}

impl NegamaxPlayer {
    /// Creates a new [`NegamaxPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        NegamaxPlayer { name: name.into() }
    }
}

impl Default for NegamaxPlayer {
    fn default() -> Self {
        Self::new("Negamax Player".to_string())
    }
}

impl Player for NegamaxPlayer {
    type Game = Patchwork;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, _game: &Self::Game) -> <Self::Game as game::Game>::Action {
        // TODO:
        todo!()
    }
}
