use patchwork_core::ActionId;

use crate::ActionList;

// TODO: write a real sorter

// 1. PV-Action -> MAX-Score
// 2. TT / Hash Moves (if nothing available internal iterative deepening
// 3. Handcrafted Action Ordering
//    * Heavily penalize moving actions (especially starting at later starting indices or with much money)
// Train parameters with texels tuning
// Look into: Killer Heuristic, History Heuristic

// A typical move ordering consists as follows:
// * PV-move of the principal variation from the previous iteration of an iterative deepening framework for the leftmost path, often implicitly done by 2.
// * Hash move from hash tables
// * Winning captures/promotions
// * Equal captures/promotions
// * Killer moves (non capture), often with mate killers first
// * Non-captures sorted by history heuristic and that like
// * Losing captures (* but see below

pub trait ActionSorter {
    /// Scores the given actions. The given actions are scored in place.
    ///
    /// # Arguments
    ///
    /// * `actions` - The actions to score.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `n` is the amount of actions and `𝒪(𝑚)` is the complexity of the `score_action` function
    /// which is usually `𝒪(𝟣)`.
    fn score_actions(&self, actions: &mut ActionList, pv_action: Option<ActionId>, current_ply: usize) {
        for i in 0..actions.len() {
            actions.scores[i] = self.score_action(actions.get_action(i), pv_action, current_ply);
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
