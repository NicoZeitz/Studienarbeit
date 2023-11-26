use crate::Game;

/// A game evaluator for a 2 player Game.
///
/// # Type Parameters
///
/// * `Game` - The type representing a game.
/// * `Evaluation` - The type representing an evaluation.
pub trait Evaluator: Sync + Send {
    type Game: crate::Game;

    /// Returns the evaluation of the given intermediate state.
    /// An intermediate state is a state that is not terminal.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_intermediate_node(&self, game: &Self::Game) -> f64;

    /// Returns the evaluation of the given terminal state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_terminal_node(&self, game: &Self::Game) -> f64;

    /// Returns the evaluation of the given state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_node(&self, game: &Self::Game) -> f64 {
        if game.is_terminated() {
            self.evaluate_terminal_node(game)
        } else {
            self.evaluate_intermediate_node(game)
        }
    }

    /// Interprets the evaluation for the opponent player.
    ///
    /// # Arguments
    ///
    /// * `evaluation` - The evaluation to interpret.
    ///
    /// # Returns
    ///
    /// The interpreted evaluation.
    fn interpret_evaluation_as_opponent(&self, evaluation: f64) -> f64 {
        -evaluation
    }
}
