use std::rc::Rc;

use candle_core::{safetensors, DType, Device};
use candle_nn::VarBuilder;
use patchwork_core::{ActionId, Patchwork, Player, PlayerResult, TreePolicy};
use tree_policy::UCTPolicy; // TODO: replace

use crate::{
    action::map_games_to_action_tensors, game_state::GameState, mcts::SearchTree, AlphaZeroOptions, PatchZero,
};

/// A computer player that uses the AlphaZero algorithm to choose an action.
pub struct AlphaZeroPlayer<Policy: TreePolicy = UCTPolicy> {
    /// The name of the player.
    pub name: String,
    /// The options for the AlphaZero algorithm.
    pub options: Rc<AlphaZeroOptions>,
    /// The search tree used to search for the best action.
    search_tree: SearchTree<Policy>,
}

/// The default network weights for the AlphaZeroPlayer.
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
    /// * `options` - The options for the AlphaZero algorithm.
    ///
    /// # Panics
    ///
    /// Panics if the network weights cannot be loaded or the network cannot be created.
    #[rustfmt::skip]
    pub fn new(name: impl Into<String>, options: Option<AlphaZeroOptions>) -> Self {
        let options = Rc::new(options.unwrap_or_default());

        let weights = safetensors::load_buffer(NETWORK_WEIGHTS, &options.device).expect("[AlphaZeroPlayer::new] Failed to load network weights");
        let vb = VarBuilder::from_tensors(weights, DType::F32, &options.device);
        let network: PatchZero = PatchZero::new(vb, options.device.clone()).expect("[AlphaZeroPlayer::new] Failed to create network");

        AlphaZeroPlayer {
            name: name.into(),
            search_tree: SearchTree::new(false, Policy::default(), network, Rc::clone(&options)),
            options: Rc::clone(&options),
        }
    }
}

impl<Policy: TreePolicy + Default> Default for AlphaZeroPlayer<Policy> {
    fn default() -> Self {
        Self::new("AlphaZero Player".to_string(), None)
    }
}

impl<Policy: TreePolicy> Player for AlphaZeroPlayer<Policy> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let mut game_states = [GameState::new(game.clone())];
        let policies = self.search_tree.search(&mut game_states)?;

        let (available_actions_tensor, mut corresponding_action_ids) = map_games_to_action_tensors(
            &game_states.iter().map(|s| &s.game).collect::<Vec<_>>(),
            &self.options.device,
        )?;

        let policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum(1)?;
        let policies = (policies / policies_sum)?;
        let policies = policies.squeeze(0)?.to_device(&Device::Cpu)?.to_vec1::<f32>()?;
        let corresponding_action_ids = corresponding_action_ids.pop_front().unwrap();

        let mut best_action_id = ActionId::null();
        let mut best_probability = 0.0;

        // choose the argmax of the policies
        for (index, policy) in policies.iter().enumerate() {
            if *policy > best_probability {
                best_probability = *policy;
                best_action_id = corresponding_action_ids[index];
            }
        }

        Ok(best_action_id)
    }
}
