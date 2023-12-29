use patchwork_core::{ActionId, Evaluator, Patchwork, Player, PlayerResult};

use patchwork_evaluator::StaticEvaluator as MinimaxEvaluator;

use crate::MinimaxOptions;

/// A computer player that uses the Minimax algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MinimaxPlayer {
    /// The name of the player.
    pub name: String,
    /// The depth to search to.
    pub depth: usize,
    /// The amount of actions to consider per piece.
    /// This is used to reduce the branching factor.
    pub amount_actions_per_piece: usize,
    /// The evaluator to evaluate the game state.
    pub evaluator: MinimaxEvaluator, // TODO: use boxed stable evaluator
}

impl MinimaxPlayer {
    /// Creates a new [`MinimaxPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<MinimaxOptions>) -> Self {
        let MinimaxOptions {
            depth,
            amount_actions_per_piece,
        } = options.unwrap_or_default();
        MinimaxPlayer {
            name: name.into(),
            evaluator: Default::default(),
            depth,
            amount_actions_per_piece,
        }
    }
}

impl Default for MinimaxPlayer {
    fn default() -> Self {
        Self::new("Minimax Player".to_string(), Default::default())
    }
}

impl Player for MinimaxPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let valid_actions = game.get_valid_actions();

        if valid_actions.len() == 1 {
            return Ok(valid_actions[0]);
        }

        let maximizing_player = game.is_flag_player_1(game.get_current_player());

        let mut chosen_action = valid_actions[0];
        let mut chosen_evaluation = if maximizing_player { i32::MIN } else { i32::MAX };

        let filter_actions = |game: &Patchwork, valid_actions: &Vec<ActionId>| {
            Self::get_best_actions(game, valid_actions, self.amount_actions_per_piece, &self.evaluator)
        };

        for (next_state, action, _) in filter_actions(game, &valid_actions) {
            let evaluation = Self::minimax(
                &next_state,
                self.depth - 1,
                i32::MIN,
                i32::MAX,
                &self.evaluator,
                &filter_actions,
            );

            // break ties randomly
            if evaluation == chosen_evaluation && rand::random() {
                chosen_action = action;
            } else {
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
            }
        }

        Ok(chosen_action)
    }
}

impl MinimaxPlayer {
    fn minimax<Filter>(
        game: &Patchwork,
        depth: usize,
        alpha: i32,
        beta: i32,
        evaluator: &impl Evaluator,
        filter_actions: &Filter, // TODO: generic filtering
    ) -> i32
    where
        Filter: Fn(&Patchwork, &Vec<ActionId>) -> Vec<(Patchwork, ActionId, i32)>,
    {
        if depth == 0 || game.is_terminated() {
            return evaluator.evaluate_node(game);
        }

        let mut alpha = alpha;
        let mut beta = beta;

        let maximizing_player = game.is_flag_player_1(game.get_current_player());
        let valid_actions = game.get_valid_actions();

        if maximizing_player {
            let mut value = i32::MIN;
            for (next_state, _, _) in filter_actions(game, &valid_actions) {
                let evaluation = Self::minimax(&next_state, depth - 1, alpha, beta, evaluator, filter_actions);
                value = value.max(evaluation);
                if value > beta {
                    break;
                }
                alpha = alpha.max(value);
            }
            value
        } else {
            let mut value = i32::MAX;
            for (next_state, _, _) in filter_actions(game, &valid_actions) {
                let evaluation = Self::minimax(&next_state, depth - 1, alpha, beta, evaluator, filter_actions);
                value = value.min(evaluation);
                if value < alpha {
                    break;
                }
                beta = beta.min(value);
            }
            value
        }
    }

    fn get_best_actions(
        game: &Patchwork,
        valid_actions: &[ActionId],
        amount_actions_per_piece: usize,
        evaluator: &impl Evaluator,
    ) -> Vec<(Patchwork, ActionId, i32)> {
        let place_first_piece_tuple = valid_actions
            .iter()
            .filter(|a| a.is_first_patch_taken() || a.is_special_patch_placement())
            .map(|action| {
                let mut state = game.clone(); // TODO: avoid cloning
                state.do_action(*action, false).unwrap();
                let evaluation = evaluator.evaluate_node(&state);
                (state, *action, evaluation)
            })
            .take(amount_actions_per_piece)
            .collect::<Vec<_>>();

        if place_first_piece_tuple
            .get(0)
            .map(|(_, a, _)| a.is_special_patch_placement())
            .unwrap_or(false)
        {
            let mut place_first_piece_tuple = place_first_piece_tuple;
            place_first_piece_tuple.sort_by(|(_, _, e1), (_, _, e2)| {
                match e2.cmp(e1) {
                    // break ties randomly
                    std::cmp::Ordering::Equal => {
                        if rand::random() {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        }
                    }
                    ordering => ordering,
                }
            });

            // special patch placement move
            return place_first_piece_tuple
                .into_iter()
                .take(amount_actions_per_piece * 3)
                .collect::<Vec<_>>();
        }

        let walking_tuple = valid_actions
            .iter()
            .find(|a| a.is_walking())
            .map(|action| {
                let mut state = game.clone(); // TODO: avoid cloning
                state.do_action(*action, false).unwrap();
                let evaluation = evaluator.evaluate_node(&state);
                (state, *action, evaluation)
            })
            .unwrap();

        let place_second_piece_tuple = valid_actions
            .iter()
            .filter(|a| a.is_second_patch_taken())
            .map(|action| {
                let mut state = game.clone(); // TODO: avoid cloning
                state.do_action(*action, false).unwrap();
                let evaluation = evaluator.evaluate_node(&state);
                (state, *action, evaluation)
            })
            .take(amount_actions_per_piece)
            .collect::<Vec<_>>();
        let place_third_piece_tuple = valid_actions
            .iter()
            .filter(|a| a.is_third_patch_taken())
            .map(|action| {
                let mut state = game.clone(); // TODO: avoid cloning
                state.do_action(*action, false).unwrap();
                let evaluation = evaluator.evaluate_node(&state);
                (state, *action, evaluation)
            })
            .take(amount_actions_per_piece)
            .collect::<Vec<_>>();

        let place_first_piece_len = (amount_actions_per_piece * 3).min(place_first_piece_tuple.len());
        let place_second_piece_len = (amount_actions_per_piece * 3).min(place_second_piece_tuple.len());
        let place_third_piece_len = (amount_actions_per_piece * 3).min(place_third_piece_tuple.len());

        let mut result = Vec::with_capacity(1 + place_first_piece_len + place_second_piece_len + place_third_piece_len);

        result.push(walking_tuple);
        result.extend(place_first_piece_tuple);
        result.extend(place_second_piece_tuple);
        result.extend(place_third_piece_tuple);

        result.sort_by(|(_, _, e1), (_, _, e2)| match e2.cmp(e1) {
            // break ties randomly
            std::cmp::Ordering::Equal => {
                if rand::random() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            }
            ordering => ordering,
        });
        result
    }
}
