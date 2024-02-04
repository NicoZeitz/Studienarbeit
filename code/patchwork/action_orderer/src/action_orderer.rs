use patchwork_core::ActionId;

use crate::ActionList;

/// The base trait to order Actions.
///
/// # FEATURE: Other Optimal ActionSorter Implementations
/// - Handcrafted Action Ordering in Tables
/// - Move Ordering via Machine Learning (something like [Neural MoveMap Heuristic](https://www.chessprogramming.org/Neural_MoveMap_Heuristic))
pub trait ActionOrderer {
    /// Scores the given actions. The given actions are scored in place.
    ///
    /// # Arguments
    ///
    /// * `actions` - The actions to score.
    /// * `pv_action` - The principal variation action.
    /// * `current_ply` - The current ply.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `n` is the amount of actions and `𝒪(𝑚)` is the complexity of the `score_action` function
    /// which is usually `𝒪(𝟣)`.
    fn score_actions(&self, actions: &mut ActionList, pv_action: Option<ActionId>, current_ply: usize) {
        for i in 0..actions.len() {
            actions.scores[i] = self.score_action(actions.get_action(i), pv_action, current_ply);
        }

        #[cfg(debug_assertions)]
        if let Some(pv_action) = pv_action {
            let (highest_score_index, highest_score) =
                actions
                    .scores
                    .iter()
                    .enumerate()
                    .fold((0, f64::NEG_INFINITY), |accumulator, (index, score)| {
                        if *score > accumulator.1 {
                            (index, *score)
                        } else {
                            accumulator
                        }
                    });

            let highest_action = actions.get_action(highest_score_index);
            let Some(pv_action_index) = actions.actions.iter().position(|action| *action == pv_action) else {
                // PV-Action wrong as it is not in the list of actions.
                return;
            };

            let pv_action_score = actions.scores[pv_action_index];

            if highest_action != pv_action {
                println!("PV-Node action is not sorted first!");
                println!("PV-Action: {:?} with score {}", pv_action, pv_action_score);
                println!("BEST_ACTION: {:?} with score {}", highest_action, highest_score);

                debug_assert_eq!(highest_action, pv_action, "Highest Action != PV-Action");
            }
        }
    }

    /// Scores the given action. The score is used to order the actions.
    ///
    /// # Laws
    ///
    /// Instances of `ActionSorter` must obey the following laws:
    /// * `∀action: score_action(action, pv_action) ≤ score_action(pv_action, pv_action)`
    /// * `score_action(action, pv_action) = score_action(pv_action, pv_action) ⇒ action = pv_action`
    ///
    /// # Arguments
    ///
    /// * `action` - The action to score.
    /// * `pv_action` - The principal variation action.
    /// * `current_ply` - The current ply.
    ///
    /// # Returns
    ///
    /// The score of the given action.
    ///
    /// # Complexity
    ///
    /// Should be implemented in `𝒪(𝟣)`.
    fn score_action(&self, action: ActionId, pv_action: Option<ActionId>, current_ply: usize) -> f64;

    /// Picks the best action from the given actions. The given actions are ordered in place.
    ///
    /// # Arguments
    ///
    /// * `actions` - The actions to pick from.
    /// * `start_index` - The index to start picking from.
    ///
    /// # Returns
    ///
    /// The best action from the given actions.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of actions.
    fn pick_action(&self, actions: &mut ActionList, start_index: usize) -> ActionId {
        for i in (start_index + 1)..actions.len() {
            if actions.get_score(i) > actions.get_score(start_index) {
                actions.swap(start_index, i);
            }
        }
        actions.get_action(start_index)
    }
}
