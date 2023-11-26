use game::{Evaluator, Game, Player};
use patchwork_core::Patchwork;

use patchwork_evaluator::StaticEvaluator as GreedyEvaluator;

/// A player that selects the action with the highest score.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GreedyPlayer {
    /// The name of the player.
    pub name: String,
    /// The evaluator to evaluate the game state.
    pub evaluator: GreedyEvaluator,
}

impl GreedyPlayer {
    /// Creates a new [`GreedyPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        GreedyPlayer {
            name: name.into(),
            evaluator: Default::default(),
        }
    }
}

impl Default for GreedyPlayer {
    fn default() -> Self {
        Self::new("Greedy Player".to_string())
    }
}

impl Player for GreedyPlayer {
    type Game = Patchwork;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Self::Game) -> <Self::Game as Game>::Action {
        let valid_actions = game.get_valid_actions();

        if valid_actions.len() == 1 {
            return valid_actions[0].clone();
        }

        let maximizing_player = game.is_maximizing_player(&game.get_current_player());

        let mut chosen_action = &valid_actions[0];
        let mut chosen_evaluation = if maximizing_player {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };

        for action in valid_actions.iter() {
            let next_state = game.get_next_state(action);

            let evaluation = self.evaluator.evaluate_node(&next_state);

            #[allow(clippy::collapsible_else_if)]
            if maximizing_player {
                if evaluation > chosen_evaluation {
                    chosen_action = action;
                    chosen_evaluation = evaluation;
                }
            } else {
                if evaluation < chosen_evaluation {
                    chosen_action = action;
                    chosen_evaluation = evaluation;
                }
            }
            // break ties randomly
            if evaluation == chosen_evaluation && rand::random() {
                chosen_action = action;
            }
        }

        chosen_action.clone()
    }
}
