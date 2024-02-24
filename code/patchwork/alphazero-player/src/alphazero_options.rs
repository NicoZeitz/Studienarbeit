use std::{
    num::NonZeroUsize,
    sync::{atomic::AtomicBool, Arc},
};

use candle_core::Device;
use patchwork_core::{Logging, PlayerResult};

/// Different end conditions for the AlphaZero algorithm.
#[derive(Clone, Debug)]
pub enum AlphaZeroEndCondition {
    /// The number of simulations to run.
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
}

impl AlphaZeroEndCondition {
    #[inline(always)]
    pub fn run_till_end<Data, Closure>(&self, mut data: Data, mut closure: Closure) -> PlayerResult<Data>
    where
        Closure: FnMut(Data) -> PlayerResult<Data>,
    {
        match self {
            AlphaZeroEndCondition::Iterations { iterations } => {
                for _ in 0..*iterations {
                    data = closure(data)?;
                }
            }
            AlphaZeroEndCondition::Time {
                duration,
                safety_margin,
            } => {
                let duration = *duration - *safety_margin;

                let start = std::time::Instant::now();
                while start.elapsed() < duration {
                    data = closure(data)?;
                }
            }
            AlphaZeroEndCondition::Flag { flag } => {
                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    data = closure(data)?;
                }
            }
        }
        Ok(data)
    }
}

pub struct AlphaZeroOptions {
    /// The epsilon value for the Dirichlet noise. This is the fraction of the noise to add to the policy.
    pub dirichlet_epsilon: f32,
    /// The alpha value for the Dirichlet noise.
    pub dirichlet_alpha: f32,
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
    pub fn new(
        end_condition: AlphaZeroEndCondition,
        dirichlet_epsilon: f32,
        dirichlet_alpha: f32,
        device: Device,
        batch_size: NonZeroUsize,
        parallelization: NonZeroUsize,
        logging: Logging,
    ) -> Self {
        Self {
            end_condition,
            dirichlet_epsilon,
            dirichlet_alpha,
            device,
            batch_size,
            parallelization,
            logging,
        }
    }

    /// Returns the default device to use for the AlphaZero algorithm.
    ///
    /// # Returns
    ///
    /// The default device to use for the AlphaZero algorithm.
    pub fn default_device() -> Device {
        if candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).ok()
        } else if candle_core::utils::metal_is_available() {
            Device::new_metal(0).ok()
        } else {
            Some(Device::Cpu)
        }
        .unwrap_or(Device::Cpu)
    }

    /// Returns the default parallelization to use for the AlphaZero algorithm.
    ///
    /// # Returns
    ///
    /// The default parallelization to use for the AlphaZero algorithm.
    pub fn default_parallelization() -> NonZeroUsize {
        std::thread::available_parallelism()
            .map(|n| unsafe { NonZeroUsize::new_unchecked(n.get() / 2) })
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
            dirichlet_epsilon: 0.25,
            dirichlet_alpha: 0.3,
            device: Self::default_device(),
            batch_size: NonZeroUsize::new(20).unwrap(),
            parallelization: Self::default_parallelization(),
            logging: Logging::default(),
        }
    }
}
