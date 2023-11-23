use crate::{EvaluationNode, Evaluator};

pub struct ScoreEvaluator {}

impl ScoreEvaluator {
    /// Creates a new [`ScoreEvaluator`].
    pub fn new() -> Self {
        ScoreEvaluator {}
    }
}

impl Default for ScoreEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for ScoreEvaluator {
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
        let game = node.game();
        let player_1_flag = game.get_player_1_flag();
        let player_2_flag = game.get_player_2_flag();

        let player_1_score = game.get_score(player_1_flag);
        let player_2_score = game.get_score(player_2_flag);

        (player_1_score - player_2_score).into()
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

    fn combine_evaluations(
        &self,
        evaluations: impl Iterator<Item = Self::Evaluation>,
    ) -> Self::Evaluation {
        let evaluations = evaluations.collect::<Vec<_>>();
        evaluations.iter().sum::<Self::Evaluation>() / evaluations.len() as f64
    }
}
