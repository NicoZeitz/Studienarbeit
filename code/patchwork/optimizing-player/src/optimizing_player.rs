use std::collections::HashMap;

use itertools::Itertools;
use patchwork_core::{ActionId, Evaluator, Patchwork, Player, PlayerResult};

use crate::lpsolve::{Problem, SolveStatus};



/// A computer player that searches for the best option to take.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptimizingPlayer<Eval: Evaluator> {
    /// The name of the player.
    name: String,
    /// The evaluator used to score game states.
    evaluator: Eval,
    options: OptimizingOptions
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptimizingOptions {
    pub timeout: std::time::Duration,
}

impl Default for OptimizingOptions {
    fn default() -> Self {
        Self { timeout: std::time::Duration::from_secs(10) }
    }
}

impl<Eval: Evaluator + Default> OptimizingPlayer<Eval> {
    /// Creates a new [`OptimizingPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<OptimizingOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            name: name.into(),
            evaluator: Eval::default(),
            options
        }
    }
}

impl<Eval: Evaluator + Default> Default for OptimizingPlayer<Eval> {
    fn default() -> Self {
        Self::new("Optimizing Player", None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ActionType {
    Walking = 0,
    FirstPatchTaken = 1,
    SecondPatchTaken = 2,
    ThirdPatchTaken = 3,
    SpecialPatchPlacement = 4,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn group_actions(action: &ActionId) -> ActionType {
    if action.is_walking() {
        ActionType::Walking
    } else if action.is_first_patch_taken() {
        ActionType::FirstPatchTaken
    } else if action.is_second_patch_taken() {
        ActionType::SecondPatchTaken
    } else if action.is_third_patch_taken() {
        ActionType::ThirdPatchTaken
    } else if action.is_special_patch_placement() {
        ActionType::SpecialPatchPlacement
    } else {
        unreachable!("[optimizing_player::group_actions] Unknown action: {:?}", action)
    }
}

impl<Eval: Evaluator> Player for OptimizingPlayer<Eval> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let mut action_map = HashMap::new();

        for (key, group) in &game
            .get_valid_actions()
            .into_iter()
            .group_by(group_actions) {
            action_map.insert(key, group.collect::<Vec<_>>());
        }

        let amount_walking_actions = action_map.get(&ActionType::Walking).map_or(0, Vec::len);
        let amount_first_patch_taken_actions = action_map.get(&ActionType::FirstPatchTaken).map_or(0, Vec::len);
        let amount_second_patch_taken_actions = action_map.get(&ActionType::SecondPatchTaken).map_or(0, Vec::len);
        let amount_third_patch_taken_actions = action_map.get(&ActionType::ThirdPatchTaken).map_or(0, Vec::len);
        let amount_special_patch_placement_actions = action_map.get(&ActionType::SpecialPatchPlacement).map_or(0, Vec::len);

        if amount_special_patch_placement_actions > 0 {
            // there can only be special patch placements
            return self.choose_from(game, &action_map[&ActionType::SpecialPatchPlacement]);
        }

        if amount_first_patch_taken_actions == 0 && amount_second_patch_taken_actions == 0 && amount_third_patch_taken_actions == 0 {
            // there can only be the walking action
            return Ok(action_map[&ActionType::Walking][0]);
        }

        debug_assert!(amount_walking_actions == 1, "[OptimizingPlayer::get_action] Wrong amount of walking actions: {amount_walking_actions}");

        let actions = std::thread::scope(|s| {
            let mut handles = vec![];
            let mut scores = vec![];

            if amount_first_patch_taken_actions > 0 {
                let actions = action_map.remove(&ActionType::FirstPatchTaken).unwrap();
                let game = game.clone();
                let timeout = self.options.timeout;

                handles.push(s.spawn(move || {
                    let score = get_optimal_score_for(&game, actions[0], timeout);
                    (score, actions)
                }));
            }

            if amount_second_patch_taken_actions > 0 {
                let actions = action_map.remove(&ActionType::SecondPatchTaken).unwrap();
                let game = game.clone();
                let timeout = self.options.timeout;

                handles.push(s.spawn(move || {
                    let score = get_optimal_score_for(&game, actions[0], timeout);
                    (score, actions)
                }));
            }

            if amount_third_patch_taken_actions > 0 {
                let actions = action_map.remove(&ActionType::ThirdPatchTaken).unwrap();
                let game = game.clone();
                let timeout = self.options.timeout;

                handles.push(s.spawn(move || {
                    let score = get_optimal_score_for(&game, actions[0], timeout);
                    (score, actions)
                }));
            }

            // walking action
            let actions = action_map.remove(&ActionType::SecondPatchTaken).unwrap();
            let game = game.clone();
            let timeout = self.options.timeout;


            let score = get_optimal_score_for(&game, actions[0], timeout);
            scores.push((score, actions));

            for handle in handles {
                scores.push(handle.join().unwrap());
            }

            let (_score, actions) = scores
                .iter()
                .max_by(|(score1, _), (score2, _)| score1.total_cmp(score2))
                .unwrap();

            actions.clone()
        });

        self.choose_from(game, &actions)
    }
}

impl<Eval: Evaluator> OptimizingPlayer<Eval> {
    fn choose_from(&self, game: &Patchwork, actions: &[ActionId]) -> PlayerResult<ActionId> {
        let mut best_score = i32::MIN;
        let mut best_index = 0;

        let color = if game.is_player_1() { 1 } else { -1 };
        let mut game = game.clone();

        for (index, action) in actions.iter().copied().enumerate() {
            game.do_action(action, false)?;

            let evaluation = color * self.evaluator.evaluate_node(&game);
            if evaluation > best_score {
                best_score = evaluation;
                best_index = index;
            } else if evaluation == best_score && rand::random::<bool>() {
                // break ties randomly
                best_index = index;
            }

            game.undo_action(action, false)?;
        }

        Ok(actions[best_index])
    }
}

fn get_optimal_score_for(game: &Patchwork, action: ActionId, timeout: std::time::Duration) -> f64 {
    let mut problem = generate_problem_for(game, action, timeout);
    let _solve_result = problem.solve(); // ignore errors
    problem.get_objective()
}

fn generate_problem_for(game: &Patchwork, action: ActionId, timeout: std::time::Duration) -> Problem {
    debug_assert!(action.is_walking() || action.is_patch_placement(), "[optimizing_player::generate_problem_for] Invalid action: {action:?}");

    // TODO:
    let mut problem = Problem::new(0, 0).unwrap();
    problem.set_timeout(timeout);

    problem
}

