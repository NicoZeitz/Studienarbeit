use std::sync::{atomic::AtomicBool, Arc};

use candle_core::Device;
use patchwork_core::{Logging, PlayerResult};

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

impl AlphaZeroEndCondition {
    #[inline(always)]
    pub fn run_till_end<T>(&self, mut closure: T) -> PlayerResult<()>
    where
        T: FnMut() -> PlayerResult<()>,
    {
        match self {
            AlphaZeroEndCondition::Iterations(iterations) => {
                for _ in 0..*iterations {
                    closure()?;
                }
            }
            AlphaZeroEndCondition::Time(duration) => {
                let start = std::time::Instant::now();
                while start.elapsed() < *duration {
                    closure()?;
                }
            }
            AlphaZeroEndCondition::Flag(flag) => {
                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    closure()?;
                }
            }
        }
        Ok(())
    }
}

pub struct AlphaZeroOptions {
    /// The epsilon value for the Dirichlet noise. This is the fraction of the noise to add to the policy.
    pub dirichlet_epsilon: f32,
    /// The alpha value for the Dirichlet noise.
    pub dirichlet_alpha: f32,
    /// The device to use for the neural network.
    pub device: Device,
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
        logging: Logging,
    ) -> Self {
        Self {
            end_condition,
            dirichlet_epsilon,
            dirichlet_alpha,
            device,
            logging,
        }
    }
}

impl Default for AlphaZeroOptions {
    fn default() -> Self {
        let device = if candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).ok()
        } else if candle_core::utils::metal_is_available() {
            Device::new_metal(0).ok()
        } else {
            Some(Device::Cpu)
        }
        .unwrap_or(Device::Cpu);

        Self {
            end_condition: AlphaZeroEndCondition::Time(std::time::Duration::from_secs(10)),
            dirichlet_epsilon: 0.25,
            dirichlet_alpha: 0.3,
            device,
            logging: Logging::default(),
        }
    }
}
