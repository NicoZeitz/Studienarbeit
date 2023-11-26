use game::{Evaluator, Game};
use patchwork_core::{Patchwork, TerminationType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WinLossEvaluator {}

impl WinLossEvaluator {
    /// Creates a new [`WinLossEvaluator`].
    pub fn new() -> Self {
        WinLossEvaluator {}
    }
}

impl Default for WinLossEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for WinLossEvaluator {
    type Game = Patchwork;

    fn evaluate_intermediate_node(&self, game: &Self::Game) -> f64 {
        self.evaluate_terminal_node(&game.random_rollout())
    }

    fn evaluate_terminal_node(&self, game: &Self::Game) -> f64 {
        match game.get_termination_result().termination {
            TerminationType::Player1Won => 1.0,
            TerminationType::Player2Won => -1.0,
            TerminationType::Draw => 0.0,
        }
    }
}
