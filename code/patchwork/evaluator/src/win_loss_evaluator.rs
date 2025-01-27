use patchwork_core::{Evaluator, Patchwork, TerminationType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WinLossEvaluator {}

impl WinLossEvaluator {
    /// Creates a new [`WinLossEvaluator`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for WinLossEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for WinLossEvaluator {
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32 {
        self.evaluate_terminal_node(&game.random_rollout())
    }

    fn evaluate_terminal_node(&self, game: &Patchwork) -> i32 {
        match game.get_termination_result().termination {
            TerminationType::Player1Won => 1,
            TerminationType::Player2Won => -1,
        }
    }
}
