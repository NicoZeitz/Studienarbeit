/// A base trait for all players.
pub trait Player {
    type Game: crate::Game;

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
    fn get_action(&mut self, game: &Self::Game) -> <Self::Game as crate::Game>::Action;
}
