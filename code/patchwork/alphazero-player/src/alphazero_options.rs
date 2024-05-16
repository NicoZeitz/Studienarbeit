use std::{
    fmt::{Display, Formatter},
    num::NonZeroUsize,
    sync::{atomic::AtomicBool, Arc},
};

use candle_core::Device;
use patchwork_core::Logging;

/// Different end conditions for the `AlphaZero` algorithm.
#[derive(Clone, Debug)]
pub enum AlphaZeroEndCondition {
    /// The number of simulations to run. This is the minimum number of simulations to run. It can happen that more simulations are run in
    /// multi-threaded environments.
    Iterations { iterations: usize },
    /// The time to run simulations for.
    Time {
        duration: std::time::Duration,
        safety_margin: std::time::Duration,
    },
    /// Run until the flag is set.
    Flag { flag: Arc<AtomicBool> },
    // TODO: extract end condition for all players
    // add something like till end for other players (e.g. greedy random)
    // UntilEnd,
}

impl Display for AlphaZeroEndCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iterations { iterations } => {
                write!(f, "Iterations({iterations})")
            }
            Self::Time { duration, .. } => {
                write!(f, "Time({duration:?})")
            }
            Self::Flag { .. } => {
                write!(f, "Flag")
            }
        }
    }
}

pub struct AlphaZeroOptions {
    /// The device to use for the neural network.
    pub device: Device,
    /// The batch size to use for running mcts simulations before doing a network evaluation.
    pub batch_size: NonZeroUsize,
    /// The number of mcts simulations to run in parallel.
    pub parallelization: NonZeroUsize,
    /// The end condition for the AlphaZero algorithm.
    pub end_condition: AlphaZeroEndCondition,
    /// Logging configuration on what to collect during the search.
    pub logging: Logging,
}

impl AlphaZeroOptions {
    /// Creates a new [`AlphaZeroOptions`].
    #[must_use]
    pub const fn new(
        end_condition: AlphaZeroEndCondition,
        device: Device,
        batch_size: NonZeroUsize,
        parallelization: NonZeroUsize,
        logging: Logging,
    ) -> Self {
        Self {
            device,
            batch_size,
            parallelization,
            end_condition,
            logging,
        }
    }

    /// Returns the default device to use for the `AlphaZero` algorithm.
    ///
    /// # Returns
    ///
    /// The default device to use for the `AlphaZero` algorithm.
    #[must_use]
    pub const fn default_device() -> Device {
        // if candle_core::utils::cuda_is_available() {
        //     Device::new_cuda(0).ok()
        // } else if candle_core::utils::metal_is_available() {
        //     Device::new_metal(0).ok()
        // } else {
        //     Some(Device::Cpu)
        // }
        // .unwrap_or(Device::Cpu)

        // CPU is always faster than GPU. Probably because of the overhead of copying the data to the GPU.
        Device::Cpu
    }

    /// Returns the default parallelization to use for the `AlphaZero` algorithm.
    ///
    /// # Returns
    ///
    /// The default parallelization to use for the `AlphaZero` algorithm.
    #[must_use]
    pub fn default_parallelization() -> NonZeroUsize {
        std::thread::available_parallelism()
            .ok()
            // .and_then(|n| NonZeroUsize::new(n.get() - 1))
            .unwrap_or(NonZeroUsize::new(4).unwrap())
    }
}

impl Default for AlphaZeroOptions {
    fn default() -> Self {
        Self {
            end_condition: AlphaZeroEndCondition::Time {
                duration: std::time::Duration::from_secs(10),
                safety_margin: std::time::Duration::from_millis(100),
            },
            device: Self::default_device(),
            batch_size: NonZeroUsize::new(20).unwrap(),
            parallelization: Self::default_parallelization(),
            logging: Logging::default(),
        }
    }
}
