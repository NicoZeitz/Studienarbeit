use patchwork_core::ActionId;

use crate::ActionSorter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NoopActionSorter;

impl ActionSorter for NoopActionSorter {
    fn score_action(&self, action: ActionId, pv_action: Option<ActionId>) -> isize {
        if pv_action.is_some() && action == pv_action.unwrap() {
            return 10000;
        }

        0
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
