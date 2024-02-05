use std::{
    num::NonZeroUsize,
    sync::{atomic::AtomicBool, Arc},
};

use patchwork_core::Logging;

use crate::mcts_player::{NON_ZERO_USIZE_FOUR, NON_ZERO_USIZE_ONE};

/// Different end conditions for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Clone, Debug)]
pub enum MCTSEndCondition {
    /// The number of simulations to run.
    Iterations(usize),
    /// The time to run simulations for.
    Time(std::time::Duration),
    /// Run until the flag is set.
    Flag(Arc<AtomicBool>),
}

/// Different options for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Debug)]
pub struct MCTSOptions {
    /// Indicates if there should be multiple mcts searches running in parallel.
    /// 1 for no parallelization.
    pub root_parallelization: NonZeroUsize,
    /// Indicates if the simulation phase is to be run in parallel.
    /// 1 for no parallelization.
    pub leaf_parallelization: NonZeroUsize,
    /// Indicates if the tree should be reused between turns.
    pub reuse_tree: bool,
    /// The end condition for the MCTS algorithm.
    pub end_condition: MCTSEndCondition,
    /// Logging configuration on what to collect during the search.
    pub logging: Logging,
}

impl MCTSOptions {
    /// Creates a new [`MCTSOptions`].
    pub fn new(
        root_parallelization: NonZeroUsize,
        leaf_parallelization: NonZeroUsize,
        end_condition: MCTSEndCondition,
        reuse_tree: bool,
        logging: Logging,
    ) -> Self {
        Self {
            root_parallelization,
            leaf_parallelization,
            end_condition,
            reuse_tree,
            logging,
        }
    }
}

impl Default for MCTSOptions {
    fn default() -> Self {
        let root_parallelization = std::thread::available_parallelism()
            .map(|n| unsafe { NonZeroUsize::new_unchecked(n.get() / 2) })
            .unwrap_or(NON_ZERO_USIZE_FOUR);

        Self {
            root_parallelization,
            leaf_parallelization: NON_ZERO_USIZE_ONE,
            end_condition: MCTSEndCondition::Time(std::time::Duration::from_secs(10)),
            reuse_tree: true,
            logging: Default::default(),
        }
    }
}
