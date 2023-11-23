use std::num::NonZeroUsize;

/// Different end conditions for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MCTSEndCondition {
    /// The number of simulations to run.
    Iterations(u32),
    /// The number of seconds to run simulations for.
    Time(std::time::Duration),
}

/// Different options for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct MCTSOptions {
    /// Indicates if there should be multiple mcts searches running in parallel.
    /// 0 for using all available cores.
    /// 1 for no parallelization.
    pub root_parallelization: usize,
    /// Indicates if the simulation phase is to be run in parallel.
    /// 0 for using all available cores.
    /// 1 for no parallelization.
    pub leaf_parallelization: usize,
    /// The end condition for the MCTS algorithm.
    pub end_condition: MCTSEndCondition,
}

impl MCTSOptions {
    pub fn new() -> Self {
        Self {
            root_parallelization: std::thread::available_parallelism()
                .map(|n| <NonZeroUsize as Into<usize>>::into(n) / 2 * 2) // TODO: 1 -> 2
                .unwrap_or(4),
            leaf_parallelization: 1,
            // end_condition: MCTSEndCondition::Iterations(100_000),
            end_condition: MCTSEndCondition::Time(std::time::Duration::from_secs(60)),
        }
    }
}

impl Default for MCTSOptions {
    fn default() -> Self {
        Self::new()
    }
}
