use patchwork_core::ActionId;

use crate::ActionSorter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NoopActionSorter;

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

impl ActionSorter for NoopActionSorter {
    fn score_action(&self, action: ActionId, pv_action: Option<ActionId>) -> isize {
        if pv_action.is_some() && action == pv_action.unwrap() {
            return 10000;
        }

        // return random number between 0 and 1000 TODO: real impl
        ((action.as_bits() as u64 * action.as_bits() as u64) % 1000) as isize
    }
}

impl Default for NoopActionSorter {
    fn default() -> Self {
        Self
    }
}

// TODO: write something like this for the real sorter
#[cfg(feature = "performance_tests")]
mod performance_tests {
    use super::*;
    use patchwork_core::Patchwork;
    use std::time::Instant;

    #[test]
    fn sort_actions() {
        let action_sorter = NoopActionSorter;
        let game = Patchwork::get_initial_state(None);
        let mut actions = game.get_valid_actions();

        let start = Instant::now();
        action_sorter.sort_actions(&mut actions, None);
        let end = Instant::now();
        println!("Sorting took: {:?}", end - start);
    }
}
