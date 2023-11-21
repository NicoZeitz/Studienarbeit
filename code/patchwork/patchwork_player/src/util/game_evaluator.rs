use patchwork_core::Game;

/// A game evaluator for a Game.
///
/// # Type Parameters
///
/// * `State` - The type representing a state.
/// * `Evaluation` - The type representing an evaluation.
///
/// # Example
///
/// implementation for Patchwork returning a constant evaluation of 0:
///
/// ```
/// pub struct Test;
/// impl GameEvaluator for Test {
///     type State = Patchwork;
///     type Evaluation = i32;
///
///     fn evaluate_state(
///         &self,
///         state: &Self::State,
///         player: &<<Self as GameEvaluator>::State as Game>::Player,
///     ) -> Self::Evaluation {
///         0
///     }
/// }
/// ```
pub trait GameEvaluator {
    type State: Game;
    type Evaluation: Into<f64>;

    /// Returns the evaluation of the given state for the given player.
    ///
    /// # Arguments
    ///
    /// * `game` - The game.
    /// * `state` - The state.
    /// * `player` - The player.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_state(
        &self,
        state: &Self::State,
        player: &<<Self as GameEvaluator>::State as Game>::Player,
    ) -> Self::Evaluation;
}
