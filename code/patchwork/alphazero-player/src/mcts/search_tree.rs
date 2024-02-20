use std::rc::Rc;

use candle_core::{Device, Tensor};
use patchwork_core::{ActionId, NaturalActionId, PatchworkError, PlayerResult, TerminationType, TreePolicy};
use rand_distr::{Dirichlet, Distribution};

use crate::{
    action::map_games_to_action_tensors,
    game_state::GameState,
    mcts::{AreaAllocator, NodeId},
    AlphaZeroOptions, PatchZero,
};

/// The search tree for the Monte Carlo Tree Search (MCTS) algorithm of the AlphaZero Player.
pub struct SearchTree<
    // The tree policy to use for the MCTS algorithm.
    Policy: TreePolicy,
    // The 3 patches that can be taken are encoded into separate layers.
    const AMOUNT_PATCH_LAYERS: usize = 3,
    // 40 in paper
    const AMOUNT_RESIDUAL_LAYERS: usize = 20,
    // 256 in paper
    const AMOUNT_FILTERS: usize = 128,
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
        let dirichlet_noise =
            Dirichlet::new(&[options.dirichlet_alpha; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS])
                .expect("Failed to create dirichlet noise distribution");

        Self {
            train,
            tree_policy,
            network,
            options,
            dirichlet_noise,
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
    pub fn search(&mut self, states: &mut [GameState]) -> PlayerResult<Tensor> {
        // 1. Expand the root node directly adding dirichlet noise to the policy
        self.create_root_node(states)?;

        // 2. Run a number of simulations/iterations fitting into end condition
        self.options.end_condition.clone().run_till_end(
            #[inline(always)]
            || self.iteration(states),
        );

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

        let policies = (Tensor::new(1.0 - self.options.dirichlet_epsilon, &self.options.device)? * policies)?
            .broadcast_add(&(Tensor::new(self.options.dirichlet_epsilon, &self.options.device)? * noise)?)?
            .detach()?;

        let (available_actions_tensor, mut corresponding_action_ids) = map_games_to_action_tensors(
            &states.iter().map(|s| &s.game).collect::<Vec<_>>(),
            &self.options.device,
        )?;
        let policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum(1)?;
        let policies = (policies / policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;

        for (index, game_state) in states.iter_mut().enumerate() {
            let root_node_id = game_state.allocator.new_node(game_state.game.clone(), None, None, None);
            let policy = &policies[index];
            let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

            game_state.root = Some(root_node_id);
            self.node_expand(root_node_id, &policy, &corresponding_actions, &mut game_state.allocator)?;
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
    pub fn iteration(&mut self, states: &mut [GameState]) -> PlayerResult<()> {
        let mut states_to_evaluate = vec![];

        for game_state in states.iter_mut() {
            let mut node_id = game_state.root.unwrap();
            while game_state.allocator.get_node(node_id).is_fully_expanded() {
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

        let games = states_to_evaluate
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
        let policies_sum = policies.sum(1)?;
        policies = (policies / policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;
        let values = values.to_device(&Device::Cpu)?.to_vec1::<f32>()?;

        for (index, (game_state, node_id)) in states_to_evaluate.iter_mut().enumerate() {
            let node = game_state.allocator.get_node_mut(*node_id);
            let policy = &policies[index];
            let value = values[index];
            let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

            self.node_expand(*node_id, &policy, &corresponding_actions, &mut game_state.allocator)?;
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
            // TODO: take q_value from parent as new fpu parameter
            allocator.new_node(child_state, Some(node_id), Some(*action), Some(*probability));
        }

        Ok(())
    }

    /// Selects the best child node of the given parent node using the tree policy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the parent node to select the best child node from.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    fn node_select(&self, node_id: NodeId, allocator: &mut AreaAllocator) -> NodeId {
        let node = allocator.get_node(node_id);
        let children = node.children.iter().map(|node_id| allocator.get_node(*node_id));

        let selected_child = self.tree_policy.select_node(node, children);
        selected_child.id

        //     best_child = None
        //     best_ucb = -np.inf

        //     for child in self.children:
        //         ucb = self.get_ucb(child)
        //         if ucb > best_ucb:
        //             best_child = child
        //             best_ucb = ucb

        //     return best_child

        //     if child.visit_count == 0:
        //     q_value = 0
        // else:
        //     q_value = 1 - ((child.value_sum / child.visit_count) + 1) / 2
        // return q_value + self.args['C'] * (math.sqrt(self.visit_count) / (child.visit_count + 1)) * child.prior
    }

    fn node_backpropagate(&mut self, mut node_id: NodeId, value: f32, allocator: &mut AreaAllocator) {
        loop {
            let node = allocator.get_node_mut(node_id);

            node.neutral_max_score = node.neutral_max_score.max(value);
            node.neutral_min_score = node.neutral_min_score.min(value);
            node.neutral_score_sum += value as f64;
            node.neutral_wins += if value > 0.0 { 1 } else { -1 };
            node.visit_count += 1;

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }
}
