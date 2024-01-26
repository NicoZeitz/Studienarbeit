use evaluator::StaticEvaluator as GreedyEvaluator;
use patchwork_core::{ActionId, Evaluator, Patchwork, Player, PlayerResult};

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
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let mut game = game.clone();
        let valid_actions = game.get_valid_actions().into_iter().collect::<Vec<_>>();

        if valid_actions.len() == 1 {
            return Ok(valid_actions[0]);
        }

        let maximizing_player = game.is_player_1();

        let mut chosen_action = valid_actions[0];
        let mut chosen_evaluation = if maximizing_player { i32::MIN } else { i32::MAX };

        for action in valid_actions.iter() {
            game.do_action(*action, false)?;
            let evaluation = self.evaluator.evaluate_node(&game);
            game.undo_action(*action, false)?;

            #[allow(clippy::collapsible_else_if)]
            if maximizing_player {
                if evaluation > chosen_evaluation {
                    chosen_action = *action;
                    chosen_evaluation = evaluation;
                }
            } else {
                if evaluation < chosen_evaluation {
                    chosen_action = *action;
                    chosen_evaluation = evaluation;
                }
            }
            // break ties randomly
            if evaluation == chosen_evaluation && rand::random() {
                chosen_action = *action;
            }
        }

        Ok(chosen_action)
    }
}
