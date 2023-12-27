use patchwork_core::{Evaluator, Patchwork, StableEvaluator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl StableEvaluator for ScoreEvaluator {}
impl Evaluator for ScoreEvaluator {
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> isize {
        self.evaluate_terminal_node(&game.random_rollout())
    }

    fn evaluate_terminal_node(&self, game: &Patchwork) -> isize {
        let player_1_flag = game.get_player_1_flag();
        let player_2_flag = game.get_player_2_flag();

        let player_1_score = game.get_score(player_1_flag);
        let player_2_score = game.get_score(player_2_flag);

        player_1_score - player_2_score
    }
}
