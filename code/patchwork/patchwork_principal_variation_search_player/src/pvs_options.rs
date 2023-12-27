use patchwork_action_sorter::{ActionSorter, NoopActionSorter};
use patchwork_core::StableEvaluator;
use patchwork_evaluator::StaticEvaluator;

use crate::transposition_table::Size;

/// Different options for the Principal Variation Search (PVS) algorithm.
pub struct PVSOptions {
    /// The time limit for the search.
    pub time_limit: std::time::Duration,
    /// The evaluator to evaluate the game state.
    pub evaluator: Box<dyn StableEvaluator>,
    /// The action sorter to sort the actions.
    pub action_sorter: Box<dyn ActionSorter>,
    /// The size of the transposition table.
    pub transposition_table_size: Size,
    /// If diagnostics should be printed.
    pub diagnostics: Option<Box<dyn std::io::Write>>,
}

impl PVSOptions {
    /// Creates a new [`PVSOptions`].
    pub fn new(
        time_limit: std::time::Duration,
        evaluator: Box<dyn StableEvaluator>,
        action_sorter: Box<dyn ActionSorter>,
        transposition_table_size: Size,
        diagnostics: Option<Box<dyn std::io::Write>>,
    ) -> Self {
        Self {
            time_limit,
            evaluator,
            action_sorter,
            transposition_table_size,
            diagnostics,
        }
    }
}

impl Default for PVSOptions {
    fn default() -> Self {
        Self {
            time_limit: std::time::Duration::from_secs(10), // TODO: real time limit
            evaluator: Box::<StaticEvaluator>::default(),
            action_sorter: Box::<NoopActionSorter>::default(),
            transposition_table_size: Size::MiB(10), // TODO: bigger value
            // diagnostics: if cfg!(debug_assertions) {
            //     Some(Box::new(std::io::stdout()))
            // } else {
            //     None
            // },
            diagnostics: Some(Box::new(std::io::stdout())),
        }
    }
}
