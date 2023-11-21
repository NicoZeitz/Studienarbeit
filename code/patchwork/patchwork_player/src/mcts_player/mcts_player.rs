use patchwork_core::{Action, Patchwork, Player};

use crate::MCTSOptions;

pub struct MCTSPlayer {
    pub options: MCTSOptions,
    pub name: String,
}

impl MCTSPlayer {
    /// Creates a new [`MCTSPlayer`].
    pub fn new(name: String, options: Option<MCTSOptions>) -> Self {
        MCTSPlayer {
            name,
            options: options.unwrap_or(MCTSOptions {}),
        }
    }
}

impl Player for MCTSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, _game: &Patchwork) -> Action {
        todo!() // TODO:
    }
}
