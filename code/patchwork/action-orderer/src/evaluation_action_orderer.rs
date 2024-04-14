use patchwork_core::{ActionId, Evaluator, Patchwork};

use crate::ActionOrderer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvaluationActionOrderer<Eval: Evaluator> {
    evaluator: Eval,
}

impl<Eval: Evaluator> EvaluationActionOrderer<Eval> {
    #[must_use]
    pub const fn new(evaluator: Eval) -> Self {
        Self { evaluator }
    }
}

impl<Eval: Evaluator> ActionOrderer for EvaluationActionOrderer<Eval> {
    fn score_action(
        &self,
        game: &Patchwork,
        action: ActionId,
        pv_action: Option<ActionId>,
        _current_ply: usize,
    ) -> f64 {
        if pv_action.is_some() && action == pv_action.unwrap() {
            return 100_000.0;
        }

        let mut next_state = game.clone();

        match next_state.do_action(action, false) {
            Ok(()) => f64::from(self.evaluator.evaluate_node(&next_state) * 100),
            Err(_) => -100_000.0,
        }
    }
}

impl<Eval: Evaluator + Default> Default for EvaluationActionOrderer<Eval> {
    fn default() -> Self {
        Self {
            evaluator: Eval::default(),
        }
    }
}
