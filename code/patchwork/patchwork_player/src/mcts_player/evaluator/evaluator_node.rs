/// A node in the game tree that is used for evaluation.
pub trait EvaluationNode {
    /// The type representing a game.
    type Game: patchwork_core::Game;

    /// Returns the game state of the node.
    ///
    /// # Returns
    ///
    /// The game state of the node.
    fn game(&self) -> Self::Game;

    /// Plays a random rollout from the given state and returns the resulting state.
    ///
    /// # Returns
    ///
    /// The resulting terminal node from the random rollout.
    fn random_rollout(&self) -> Self;
}
