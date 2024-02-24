use std::rc::Rc;

use candle_core::{Device, Tensor};
use patchwork_core::{ActionId, NaturalActionId, Patchwork, PatchworkError, PlayerResult, TerminationType, TreePolicy};
use rand_distr::{Dirichlet, Distribution};

use crate::{
    action::map_games_to_action_tensors,
    game_state::GameState,
    mcts::{AreaAllocator, NodeId},
    network::PatchZero,
    AlphaZeroOptions,
};

/// The search tree for the Monte Carlo Tree Search (MCTS) algorithm of the AlphaZero Player.
pub struct SearchTree<
    // The tree policy to use for the MCTS algorithm.
    Policy: TreePolicy,
    // The amount of layers to use for encoding patches inside the neural network as own blocks.
    // The minimum that should be used is 3 so that each available patch can be encoded in it's own
    // layer.
    const AMOUNT_PATCH_LAYERS: usize = 3,
    // The amount of residual layers to use in the neural network.
    //
    // # Note
    //
    // In the AlphaZero paper, the amount of residual layers is 40.
    const AMOUNT_RESIDUAL_LAYERS: usize = 10,
    // The amount of filters to use in the neural network.
    //
    // # Note
    //
    // In the AlphaZero paper, the amount of filters is 256.
    const AMOUNT_FILTERS: usize = 64,
> {
    /// Whether the network is in training or evaluation/interference mode
    pub train: bool,
    /// The network to use to evaluate the game states.
    pub network: PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
    /// The options to use for the search tree.
    pub options: Rc<AlphaZeroOptions>,
    /// The dirichlet noise distribution to use for the root node.
    pub dirichlet_noise: Dirichlet<f32>,
    /// The policy to select nodes during the selection phase.
    tree_policy: Policy,
    // The batch states to evaluate in the next network evaluation
    batch_states: Vec<(GameState, NodeId)>,
}

impl<
        Policy: TreePolicy,
        const AMOUNT_PATCH_LAYERS: usize,
        const AMOUNT_RESIDUAL_LAYERS: usize,
        const AMOUNT_FILTERS: usize,
    > SearchTree<Policy, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>
{
    /// Creates a new search tree with the given network and options.
    ///
    /// # Arguments
    ///
    /// * `train` - Whether the network should be in training mode or not.
    /// * `tree_policy` - The tree policy to use for the search tree.
    /// * `network` - The network to use for the search tree.
    /// * `options` - The options to use for the search tree.
    ///
    /// # Returns
    ///
    /// The created search tree.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn new(
        train: bool,
        tree_policy: Policy,
        network: PatchZero<{ AMOUNT_PATCH_LAYERS }, { AMOUNT_RESIDUAL_LAYERS }, { AMOUNT_FILTERS }>,
        options: Rc<AlphaZeroOptions>,
    ) -> Self {
        let dirichlet_noise = Dirichlet::new_with_size(
            options.dirichlet_alpha,
            NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,
        )
        .expect("Failed to create dirichlet noise distribution");

        Self {
            train,
            tree_policy,
            network,
            options,
            dirichlet_noise,
            batch_states: vec![],
        }
    }

    /// Sets the network to evaluation or training mode
    ///
    /// # Arguments
    ///
    /// * `train` - Whether the network should be in training mode or not
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn set_train(&mut self, train: bool) {
        self.train = train;
    }

    /// Searches for the best action to take in the given game states.
    ///
    /// # Arguments
    ///
    /// * `states` - The game states to search for the best action.
    ///
    /// # Returns
    ///
    /// The action probabilities for each game state.
    pub fn search(&mut self, states: &[&Patchwork]) -> PlayerResult<Tensor> {
        let mut states = states
            .iter()
            .map(|game| GameState::new((*game).clone()))
            .collect::<Vec<_>>();

        // 1. Expand the root node directly adding dirichlet noise to the policy
        self.create_root_node(&mut states)?;

        // 2. Run a number of simulations/iterations fitting into end condition
        let states = self.options.end_condition.clone().run_till_end(
            states,
            #[inline(always)]
            |states| self.iteration(states),
        )?;
        self.do_batch_evaluation(true);

        // 3. Return the action probabilities for each game state
        Ok(Tensor::stack(
            states
                .iter()
                .map(|state| {
                    let mut action_probabilities = [0.0; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];

                    let root_node_id = state.root.unwrap();
                    let root_node = state.allocator.get_node(root_node_id);
                    for node_id in root_node.children.iter() {
                        let node = state.allocator.get_node(*node_id);

                        let Some(action_taken) = node.action_taken else {
                            continue;
                        };
                        let natural_action_id = action_taken.to_natural_action_id().as_bits() as usize;
                        action_probabilities[natural_action_id] = node.visit_count as f32;
                    }

                    let sum = action_probabilities.iter().sum::<f32>();
                    for probability in action_probabilities.iter_mut() {
                        *probability /= sum;
                    }

                    Tensor::from_slice(
                        &action_probabilities,
                        (NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,),
                        &Device::Cpu,
                    )
                })
                .collect::<candle_core::Result<Vec<_>>>()?
                .as_slice(),
            0,
        )?)
    }

    /// Creates the root node for the given game states. Also expands the root node by adding all possible child nodes.
    ///
    /// # Arguments
    ///
    /// * `states` - The game states to create the root node for.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the root node was created successfully, `Err(PatchworkError)` otherwise.
    pub fn create_root_node(&mut self, states: &mut [GameState]) -> PlayerResult<()> {
        let games = states.iter().map(|state| &state.game).collect::<Vec<_>>();

        let (policies, _values) = self.network.forward_t(&games, self.train)?;
        let policies = candle_nn::ops::softmax(&policies, 1)?.detach()?;

        let noise = Tensor::from_vec(
            self.dirichlet_noise.sample(&mut rand::thread_rng()),
            (NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,),
            &self.options.device,
        )?
        .detach()?;

        let policies = Tensor::broadcast_add(
            &Tensor::broadcast_mul(
                &Tensor::new(1.0 - self.options.dirichlet_epsilon, &self.options.device)?,
                &policies,
            )?,
            &Tensor::broadcast_mul(
                &Tensor::new(self.options.dirichlet_epsilon, &self.options.device)?,
                &noise,
            )?,
        )?
        .detach()?;

        let (available_actions_tensor, mut corresponding_action_ids) = map_games_to_action_tensors(
            &states.iter().map(|s| &s.game).collect::<Vec<_>>(),
            &self.options.device,
        )?;
        let policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum_keepdim(1)?;
        let policies = policies.broadcast_div(&policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;

        for (index, game_state) in states.iter_mut().enumerate() {
            let root_node_id = game_state.allocator.new_node(game_state.game.clone(), None, None, None);
            let policy = &policies[index];
            let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

            game_state.root = Some(root_node_id);
            self.node_expand(root_node_id, policy, &corresponding_actions, &mut game_state.allocator)?;
        }

        Ok(())
    }

    /// Runs a single iteration of the MCTS algorithm for the given game states.
    ///
    /// # Arguments
    ///
    /// * `states` - The game states to run the iteration for.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the iteration was run successfully, `Err(PatchworkError)` otherwise.
    pub fn iteration(&mut self, states: Vec<GameState>) -> PlayerResult<Vec<GameState>> {
        let mut states_to_evaluate: Vec<(GameState, NodeId)> = vec![];

        for mut game_state in states {
            let mut node_id = game_state.root.unwrap();
            loop {
                let mut node = game_state.allocator.get_node_mut(node_id);
                if !node.is_fully_expanded() {
                    break;
                }

                node.increment_virtual_loss();
                node_id = self.node_select(node_id, &mut game_state.allocator);
            }

            let node = game_state.allocator.get_node(node_id);
            if node.state.is_terminated() {
                // directly evaluate the node if it is a terminal node
                let value = match node.state.get_termination_result().termination {
                    TerminationType::Player1Won => 1.0,
                    TerminationType::Player2Won => -1.0,
                };
                self.node_backpropagate(node_id, value, &mut game_state.allocator);
            } else {
                // add current node to allow for batch evaluation with the network
                states_to_evaluate.push((game_state, node_id));
            }
        }

        if states_to_evaluate.is_empty() {
            return Ok(());
        }

        self.batch_states.extend(states_to_evaluate);
        self.do_batch_evaluation(false)
    }

    /// Evaluates the batch states with the network and backpropagates the results.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to force the evaluation of the batch states even if there are less batch states than the batch size.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the batch evaluation was run successfully, `Err(PatchworkError)` otherwise.
    fn do_batch_evaluation(&mut self, force: bool) -> PlayerResult<()> {
        if self.batch_states.len() < self.options.batch_size.get() && !force {
            return Ok(());
        }

        let mut batch_states = self.batch_states;
        self.batch_states = vec![];

        let games = batch_states
            .iter()
            .map(|(game_state, node_id)| {
                let node = game_state.allocator.get_node(*node_id);
                &node.state
            })
            .collect::<Vec<_>>();
        let (available_actions_tensor, mut corresponding_action_ids) =
            map_games_to_action_tensors(&games, &self.options.device)?;

        let (mut policies, values) = self.network.forward_t(&games, self.train)?;
        policies = candle_nn::ops::softmax(&policies, 1)?;
        policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum_keepdim(1)?;
        policies = policies.broadcast_div(&policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;
        let values = values.to_device(&Device::Cpu)?.to_vec1::<f32>()?;

        for (index, (game_state, node_id)) in batch_states.iter_mut().enumerate() {
            let policy = &policies[index];
            let value = values[index];
            let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

            self.node_expand(*node_id, policy, &corresponding_actions, &mut game_state.allocator)?;
            self.node_backpropagate(*node_id, value, &mut game_state.allocator);
        }

        Ok(())
    }
}

/// Implementation of the methods for the Monte Carlo Tree Search (MCTS) algorithm.
impl<
        Policy: TreePolicy,
        const NUMBER_OF_PATCH_LAYERS: usize,
        const NUMBER_OF_RESIDUAL_LAYERS: usize,
        const NUMBER_OF_FILTERS: usize,
    > SearchTree<Policy, NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
{
    /// Expands the given node by adding all possible child nodes. For each of the given actions a new child node is
    /// created with the given probability.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to expand.
    /// * `policies` - The policy for each action.
    /// * `corresponding_actions` - The actions that correspond to the policies.
    /// * `allocator` - The allocator to use.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the node was expanded successfully, `Err(PatchworkError)` otherwise.
    fn node_expand(
        &mut self,
        node_id: NodeId,
        policies: &[f32],
        corresponding_actions: &[ActionId],
        allocator: &mut AreaAllocator,
    ) -> Result<(), PatchworkError> {
        for (probability, action) in policies.iter().zip(corresponding_actions).filter(|(p, _)| **p > 0.0) {
            let mut child_state = allocator.get_node(node_id).state.clone();
            child_state.do_action(*action, false)?;
            allocator.new_node(child_state, Some(node_id), Some(*action), Some(*probability));
        }

        Ok(())
    }

    /// Selects the best child node of the given parent node using the tree policy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the parent node to select the best child node from.
    /// * `allocator` - The allocator to use.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    fn node_select(&self, node_id: NodeId, allocator: &mut AreaAllocator) -> NodeId {
        let node = allocator.get_node(node_id);
        let children = node.children.iter().map(|node_id| allocator.get_node(*node_id));

        let selected_child = self.tree_policy.select_node(node, children);
        selected_child.id
    }

    /// Backpropagates the score of the game up from the given node until the root node is reached.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to backpropagate from.
    /// * `value` - The value to backpropagate.
    /// * `allocator` - The allocator to use.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the depth of the current node as the chain until the root needs to be traversed
    fn node_backpropagate(&mut self, mut node_id: NodeId, value: f32, allocator: &mut AreaAllocator) {
        loop {
            let node = allocator.get_node_mut(node_id);

            node.neutral_max_score = node.neutral_max_score.max(value);
            node.neutral_min_score = node.neutral_min_score.min(value);
            node.neutral_score_sum += value as f64;
            node.neutral_wins += if value > 0.0 { 1 } else { -1 };
            node.visit_count += 1;
            node.decrement_virtual_loss();

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }
}
