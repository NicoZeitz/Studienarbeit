use std::cmp::Reverse;

use patchwork_core::Action;

pub trait ActionSorter {
    /// Sorts the given actions. The given actions are sorted in place.
    ///
    /// # Arguments
    ///
    /// * `actions` - The actions to sort.
    ///
    /// # Complexity
    /// `𝒪(𝑚 · 𝑛 + 𝑛 · log(𝑛))` where `n` is the amount of actions and `𝒪(𝑚)` is the complexity of the `score_action` function.
    fn sort_actions(&self, actions: &mut [Action], pv_action: Option<&Action>) {
        // Sort highest score first
        actions.sort_by_cached_key(|action| Reverse(self.score_action(action, pv_action)));
    }

    /// Scores the given action. The score is used to sort the actions.
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
    fn score_action(&self, action: &Action, pv_action: Option<&Action>) -> isize;
}
