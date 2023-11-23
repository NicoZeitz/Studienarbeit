use crate::{EvaluationNode, Evaluator};
use patchwork_core::TerminationType;

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
    type Game = patchwork_core::Patchwork;

    type Evaluation = f64;

    fn evaluate_intermediate_node<Node: EvaluationNode<Game = Self::Game>>(
        &self,
        node: Node,
    ) -> Self::Evaluation {
        self.evaluate_terminal_node(node.random_rollout())
    }

    fn evaluate_terminal_node<Node: EvaluationNode<Game = Self::Game>>(
        &self,
        node: Node,
    ) -> Self::Evaluation {
        match node.game().get_termination_result().termination {
            TerminationType::Player1Won => 1.0,
            TerminationType::Player2Won => -1.0,
            TerminationType::Draw => 0.0,
        }
    }

    fn interpret_evaluation_for_player(
        &self,
        state: &Self::Game,
        player: &<<Self as Evaluator>::Game as patchwork_core::Game>::Player,
        evaluation: &Self::Evaluation,
    ) -> Self::Evaluation {
        if state.is_flag_player_1(*player) {
            *evaluation
        } else {
            -*evaluation
        }
    }
}
