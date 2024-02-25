use std::sync::{atomic::AtomicUsize, Arc};

use arc_swap::ArcSwap;
use candle_core::Device;
use dashmap::DashMap;
use patchwork_core::TreePolicy;

use crate::{game_state::GameState, mcts::NodeId, network::PatchZero, AlphaZeroOptions};

/// The search data that is shared between the search threads
pub struct SearchData<
    Policy: TreePolicy,
    const AMOUNT_PATCH_LAYERS: usize,
    const AMOUNT_RESIDUAL_LAYERS: usize,
    const AMOUNT_FILTERS: usize,
> {
    /// The amount of iterations that have been done.
    pub iterations: AtomicUsize,
    /// The device to use for the neural network.
    pub device: Device,
    /// If the search should be done in a training mode.
    pub train: bool,
    /// The batch size to use for running mcts simulations before doing a network evaluation.
    pub mini_batch_size: usize,
    /// The network to use to evaluate the game states.
    pub network: PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
    /// The policy to select nodes during the selection phase.
    pub tree_policy: Policy,
    /// The batch of game states to evaluate in one search call. The length is the batch size and fixed.
    pub batch: Vec<GameState>, // TODO: multi threaded game state
    /// The states to evaluate in the next mini-batch evaluation. The map contains the index into the correct batch of
    /// the batch vector and the correct node id to evaluate. The value is the amount of times the node needs to
    /// be evaluated. The amount will be 1 most of the time but it can happen that a node is selected multiple times
    /// before a batch evaluation is done even if the virtual loss makes this less likely
    pub evaluation_mini_batch: ArcSwap<DashMap<(usize, NodeId), i32>>,
    /// The counter for the mini-batch evaluation. This is used to determine when to do a mini-batch evaluation.
    /// The counter is incremented every time a node is selected for evaluation.
    pub mini_batch_counter: AtomicUsize,
}

impl<
        Policy: TreePolicy,
        const AMOUNT_PATCH_LAYERS: usize,
        const AMOUNT_RESIDUAL_LAYERS: usize,
        const AMOUNT_FILTERS: usize,
    > SearchData<Policy, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>
{
    /// Creates a new search data.
    ///
    /// # Arguments
    ///
    /// * `options` - The options to use for the search.
    /// * `train` - If the search should be done in a training mode.
    /// * `network` - The network to use to evaluate the game states.
    /// * `states` - The game states to search for the best action.
    ///
    /// # Returns
    ///
    /// The new search data.
    pub fn new(
        options: &AlphaZeroOptions,
        train: bool,
        network: PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
        tree_policy: Policy,
        batch: Vec<GameState>,
    ) -> Self {
        Self {
            iterations: AtomicUsize::new(0),
            device: options.device.clone(),
            mini_batch_size: options.batch_size.get(),
            train,
            network,
            tree_policy,
            batch,
            evaluation_mini_batch: ArcSwap::new(Arc::new(DashMap::with_capacity(options.batch_size.get()))),
            mini_batch_counter: AtomicUsize::new(0),
        }
    }
}
