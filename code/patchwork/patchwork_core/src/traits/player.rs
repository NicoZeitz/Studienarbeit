use anyhow::Result;

pub type PlayerResult<T> = Result<T>;

use crate::{ActionId, Patchwork};

/// A base trait for all players.
pub trait Player {
    /// Returns the name of the player.
    fn name(&self) -> &str;

    /// A method that returns the action that the player wants to take.
    ///
    /// # Arguments
    ///
    /// * `state` - The current state of the game.
    ///
    /// # Returns
    ///
    /// The action that the player wants to take.
    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId>;
}
