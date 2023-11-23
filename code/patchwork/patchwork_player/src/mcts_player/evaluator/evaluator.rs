use patchwork_core::Game;

use crate::EvaluationNode;

/// A game evaluator for a Game.
///
/// # Type Parameters
///
/// * `Game` - The type representing a game.
/// * `Evaluation` - The type representing an evaluation.
pub trait Evaluator: Sync + Send {
    type Game: patchwork_core::Game;
    type Evaluation: Into<f64> + Send;

    /// Returns the evaluation of the given intermediate state.
    /// An intermediate state is a state that is not terminal.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_intermediate_node<Node: EvaluationNode<Game = Self::Game>>(
        &self,
        node: Node,
    ) -> Self::Evaluation;

    /// Returns the evaluation of the given terminal state.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_terminal_node<Node: EvaluationNode<Game = Self::Game>>(
        &self,
        node: Node,
    ) -> Self::Evaluation;

    /// Returns the evaluation of the given state for the given player.
    ///
    /// # Arguments
    ///
    /// * `state` - The state to evaluate.
    /// * `player` - The player to evaluate the state for.
    /// * `evaluation` - The evaluation to interpret.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state for the given player.
    fn interpret_evaluation_for_player(
        &self,
        state: &Self::Game,
        player: &<<Self as Evaluator>::Game as Game>::Player,
        evaluation: &Self::Evaluation,
    ) -> Self::Evaluation;

    /// Combines the given evaluations into a single evaluation.
    ///
    /// # Arguments
    ///
    /// * `evaluations` - The evaluations to combine.
    ///
    /// # Returns
    ///
    /// The combined evaluation.
    fn combine_evaluations(
        &self,
        evaluations: impl Iterator<Item = Self::Evaluation>,
    ) -> Self::Evaluation;
}
