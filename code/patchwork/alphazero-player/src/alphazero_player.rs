use std::rc::Rc;

use candle_core::{safetensors, DType, Device};
use candle_nn::VarBuilder;
use patchwork_core::{ActionId, Patchwork, Player, PlayerResult, TreePolicy};
use tree_policy::PUCTPolicy;

use crate::{
    action::map_games_to_action_tensors, mcts::DefaultSearchTree, network::DefaultPatchZero, AlphaZeroOptions,
};

/// A computer player that uses the `AlphaZero` algorithm to choose an action.
pub struct AlphaZeroPlayer<Policy: TreePolicy = PUCTPolicy> {
    /// The name of the player.
    pub name: String,
    /// The options for the AlphaZero algorithm.
    pub options: Rc<AlphaZeroOptions>,
    /// The search tree used to search for the best action.
    search_tree: DefaultSearchTree<Policy>,
}

/// The default network weights for the `AlphaZeroPlayer`.
static NETWORK_WEIGHTS: &[u8; include_bytes!("network/weights/patch_zero.safetensors").len()] =
    include_bytes!("network/weights/patch_zero.safetensors");

impl<Policy: TreePolicy + Default> AlphaZeroPlayer<Policy> {
    /// Creates a new [`AlphaZeroPlayer`] with the given name and options.
    /// If no options are given, the default options are used.
    /// The network weights are loaded from the included weights file.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the player.
    /// * `options` - The options for the `AlphaZero` algorithm.
    ///
    /// # Panics
    ///
    /// Panics if the network weights cannot be loaded or the network cannot be created.
    #[rustfmt::skip]
    #[must_use]
    pub fn new(name: &str, options: Option<AlphaZeroOptions>) -> Self {
        let options = Rc::new(options.unwrap_or_default());

        let weights = safetensors::load_buffer(NETWORK_WEIGHTS, &options.device).expect("[AlphaZeroPlayer::new] Failed to load network weights");
        let vb = VarBuilder::from_tensors(weights, DType::F32, &options.device);
        let network = DefaultPatchZero::new(vb, options.device.clone()).expect("[AlphaZeroPlayer::new] Failed to create network");

        Self {
            name:  Self::format_name(name, options.as_ref(), None),
            search_tree: DefaultSearchTree::new(
                false,
                Policy::default(),
                network,
                Rc::clone(&options),
                1.0, // Dummy value as the network is in evaluation mode anyways
                0.0
            ),
            options,
        }
    }

    fn format_name(name: &str, options: &AlphaZeroOptions, weights_file: Option<&std::path::Path>) -> String {
        format!(
            "{} [{}|B{}|P{}]{}",
            name,
            if options.device.is_cuda() {
                "GPU"
            } else if options.device.is_metal() {
                "METAL"
            } else if cfg!(feature = "mkl") {
                "MKL"
            } else if cfg!(features = "accelerate") {
                "ACCELERATE"
            } else {
                "CPU"
            },
            options.batch_size.get(),
            options.parallelization.get(),
            weights_file.map_or_else(String::new, |weights_file| format!(" ({})", weights_file.display()))
        )
    }
}

impl<Policy: TreePolicy + Default> Default for AlphaZeroPlayer<Policy> {
    fn default() -> Self {
        Self::new("AlphaZero Player", None)
    }
}

impl<Policy: TreePolicy> Player for AlphaZeroPlayer<Policy> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let games = [game];
        let policies = self.search_tree.search(&games)?;

        let (available_actions_tensor, mut corresponding_action_ids) =
            map_games_to_action_tensors(&games, &self.options.device)?;

        let policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum_keepdim(1)?;
        let policies = policies.broadcast_div(&policies_sum)?;
        let policies = policies.squeeze(0)?.to_device(&Device::Cpu)?.to_vec1::<f32>()?;
        let corresponding_action_ids = corresponding_action_ids.pop_front().unwrap();

        let mut best_action_id = ActionId::null();
        let mut best_probability = 0.0;

        // choose the argmax of the visit probabilities
        for (index, policy) in policies.iter().enumerate() {
            if *policy > best_probability {
                best_probability = *policy;
                best_action_id = corresponding_action_ids[index];
            }
        }

        Ok(best_action_id)
    }
}
