use std::num::NonZeroUsize;

/// Different end conditions for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MCTSEndCondition {
    /// The number of simulations to run.
    Iterations(usize),
    /// The number of seconds to run simulations for.
    Time(std::time::Duration),
}

/// Different options for the Monte Carlo Tree Search (MCTS) algorithm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MCTSOptions {
    /// Indicates if there should be multiple mcts searches running in parallel.
    /// 1 for no parallelization.
    pub root_parallelization: NonZeroUsize,
    /// Indicates if the simulation phase is to be run in parallel.
    /// 1 for no parallelization.
    pub leaf_parallelization: NonZeroUsize,
    /// The end condition for the MCTS algorithm.
    pub end_condition: MCTSEndCondition,
    /// Indicates if the tree should be reused between turns.
    pub reuse_tree: bool, // TODO: implement
}

impl MCTSOptions {
    /// Creates a new [`MCTSOptions`].
    pub fn new(
        root_parallelization: NonZeroUsize,
        leaf_parallelization: NonZeroUsize,
        end_condition: MCTSEndCondition,
        reuse_tree: bool,
    ) -> Self {
        Self {
            root_parallelization,
            leaf_parallelization,
            end_condition,
            reuse_tree,
        }
    }
}

pub(crate) const NON_ZERO_USIZE_ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
#[allow(dead_code)] // TODO: use this when it is stable
pub(crate) const NON_ZERO_USIZE_FOUR: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(4) };

impl Default for MCTSOptions {
    fn default() -> Self {
        Self {
            root_parallelization: NON_ZERO_USIZE_ONE,
            leaf_parallelization: NON_ZERO_USIZE_ONE,
            end_condition: MCTSEndCondition::Iterations(10_000),
            reuse_tree: false,
        }

        // let root_parallelization = std::thread::available_parallelism()
        //     .map(|n| unsafe { NonZeroUsize::new_unchecked(Into::<usize>::into(n) / 2) })
        //     .unwrap_or(NON_ZERO_USIZE_FOUR);

        // // TODO: use this when it is stable
        // Self::new(
        //     root_parallelization,
        //     NON_ZERO_USIZE_ONE,
        //     MCTSEndCondition::Time(std::time::Duration::from_secs(10)),
        //     true,
        // )
    }
}
