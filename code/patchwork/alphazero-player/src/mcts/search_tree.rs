use candle_core::{Device, IndexOp, Tensor};
use patchwork_core::{ActionId, NaturalActionId, Patchwork, PatchworkError, Player, PlayerResult};
use rand_distr::{Dirichlet, Distribution};

use crate::{
    action::convert_action_ids_to_possible_natural_actions, game_state::GameState, AlphaZeroEndCondition,
    AlphaZeroOptions, PatchZero,
};

use super::{area_allocator::AreaAllocator, NodeId};

pub struct SearchTree<
    'a,
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
    pub network: &'a PatchZero<'a, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
    /// The allocator to allocate new nodes.
    pub(crate) allocator: AreaAllocator,
    pub options: AlphaZeroOptions,
}

impl<
        'a,
        const NUMBER_OF_PATCH_LAYERS: usize,
        const NUMBER_OF_RESIDUAL_LAYERS: usize,
        const NUMBER_OF_FILTERS: usize,
    > SearchTree<'a, NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
{
    pub fn new(train: bool) -> Self {
        Self {
            train,
            allocator: AreaAllocator::new(),
            network: todo!(),
            options: todo!(),
        }
    }

    pub fn search(&self, states: &mut [GameState], end_condition: AlphaZeroEndCondition) -> PlayerResult<()> {
        self.create_root_node(states);

        // 1. Expand the root node directly adding dirichlet noise to the policy
        // 2. Run a number of simulations/iterations fitting in time
        // 3. Return the best move

        Ok(())
    }

    pub fn create_root_node(&self, states: &mut [GameState]) -> PlayerResult<()> {
        let games = states.iter().map(|state| state.game).collect::<Vec<_>>();

        let (policies, _values) = self.network.forward_t(&games, self.train)?;
        let policies = candle_nn::ops::softmax(&policies, 1)?.to_device(&Device::Cpu)?;

        let dirichlet_noise =
            Dirichlet::new(&[self.options.dirichlet_alpha; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS])?
                .sample(&mut rand::thread_rng());
        let noise = Tensor::from_vec(dirichlet_noise, (dirichlet_noise.len(),), &Device::Cpu)?.detach()?;

        let policies = (Tensor::new(1.0 - self.options.dirichlet_epsilon, &Device::Cpu)? * policies)?
            .broadcast_add(&(Tensor::new(self.options.dirichlet_epsilon, &Device::Cpu)? * noise)?)?;

        for (index, game_state) in states.iter().enumerate() {
            let policy = policies.i((index, ..))?;
            let valid_actions = game_state.game.get_valid_actions();
            let (valid_actions_tensor, corresponding_actions) =
                convert_action_ids_to_possible_natural_actions(&valid_actions, &Device::Cpu)?;
            let valid_actions_tensor = valid_actions_tensor.detach()?;

            let policy = (policy * valid_actions_tensor)?;
            let policy = (policy / policy.sum(0)?)?;

            let root = self.allocator.new_node(game_state.game.clone(), None, None, None);
            game_state.root = Some(root);
            self.node_expand(root, policy.to_vec1()?, corresponding_actions);
        }

        // let policies =
        //     ((1 - self.args.dirichlet_epsilon) * policies + self.args.dirichlet_epsilon * noise).to_vec2::<f32>()?;

        // policy, _ = self.model(
        //     torch.tensor(self.game.get_encoded_state(states), device=self.model.device)
        // )
        // policy = torch.softmax(policy, axis=1).cpu().numpy()
        // policy = (1 - self.args['dirichlet_epsilon']) * policy + self.args['dirichlet_epsilon'] \
        //     * np.random.dirichlet([self.args['dirichlet_alpha']] * self.game.action_size, size=policy.shape[0])

        // for i, spg in enumerate(spGames):
        //     spg_policy = policy[i]
        //     valid_moves = self.game.get_valid_moves(states[i])
        //     spg_policy *= valid_moves
        //     spg_policy /= np.sum(spg_policy)

        //     spg.root = Node(self.game, self.args, states[i], visit_count=1)
        //     spg.root.expand(spg_policy)

        // let games = states.iter().map(|state| state.game).collect::<Vec<_>>();

        // let (policies, values) = self.network.forward_t(&games, false);

        // .to_vec2::<f32>()?;
        // TODO: add dirichlet noise

        Ok(())
    }
}

/// Implementation of the methods for the Monte Carlo Tree Search (MCTS) algorithm.
impl<
        'a,
        const NUMBER_OF_PATCH_LAYERS: usize,
        const NUMBER_OF_RESIDUAL_LAYERS: usize,
        const NUMBER_OF_FILTERS: usize,
    > SearchTree<'a, NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
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
        policies: Vec<f32>,
        corresponding_actions: Vec<ActionId>,
    ) -> Result<(), PatchworkError> {
        for (probability, action) in policies
            .into_iter()
            .zip(corresponding_actions)
            .filter(|(p, _)| *p > 0.0)
        {
            let mut child_state = self.allocator.get_node(node_id).state.clone();
            child_state.do_action(action, false)?;
            // TODO: take q_value from parent as new fpu parameter
            self.allocator
                .new_node(child_state, Some(node_id), Some(action), Some(probability));
        }

        Ok(())
    }

    fn node_backpropagate(&mut self, mut node_id: NodeId, value: f32) {
        loop {
            let node = self.allocator.get_node_mut(node_id);

            node.neutral_score_sum += value as i64;
            node.visit_count += 1;

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }
}
