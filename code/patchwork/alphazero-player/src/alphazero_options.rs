use std::sync::{atomic::AtomicBool, Arc};

use patchwork_core::Logging;

/// Different end conditions for the AlphaZero algorithm.
#[derive(Clone, Debug)]
pub enum AlphaZeroEndCondition {
    /// The number of simulations to run.
    Iterations(usize),
    /// The time to run simulations for.
    Time(std::time::Duration),
    /// Run until the flag is set.
    Flag(Arc<AtomicBool>),
    // TODO: extract end condition for all players
    // add something like till end for other players (e.g. greedy random)
}

pub struct AlphaZeroOptions {
    pub dirichlet_epsilon: f32,
    pub dirichlet_alpha: f32,
    /// The end condition for the AlphaZero algorithm.
    pub end_condition: AlphaZeroEndCondition,
    /// Logging configuration on what to collect during the search.
    pub logging: Logging,
}

impl AlphaZeroOptions {
    /// Creates a new [`AlphaZeroOptions`].
    pub fn new(end_condition: AlphaZeroEndCondition, logging: Logging) -> Self {
        Self { end_condition, logging }
    }
}

impl Default for AlphaZeroOptions {
    fn default() -> Self {
        Self {
            end_condition: MCTSEndCondition::Time(std::time::Duration::from_secs(10)),
            logging: Logging::default(),
        }
    }
}
