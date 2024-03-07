use std::sync::Arc;
use std::{rc::Rc, sync::atomic::AtomicBool};

use candle_core::{Device, Tensor};
use patchwork_core::{NaturalActionId, Patchwork, PlayerResult, TreePolicy};
use rand_distr::{Dirichlet, Distribution};

use crate::AlphaZeroEndCondition;
use crate::{
    action::map_games_to_action_tensors,
    game_state::GameState,
    mcts::{AreaAllocator, Node, SearchData, WorkerThread},
    network::{PatchZero, DEFAULT_AMOUNT_FILTERS, DEFAULT_AMOUNT_PATCH_LAYERS, DEFAULT_AMOUNT_RESIDUAL_LAYERS},
    AlphaZeroOptions,
};

pub type DefaultSearchTree<Policy> =
    SearchTree<Policy, DEFAULT_AMOUNT_PATCH_LAYERS, DEFAULT_AMOUNT_RESIDUAL_LAYERS, DEFAULT_AMOUNT_FILTERS>;

/// The search tree for the Monte Carlo Tree Search (MCTS) algorithm of the `AlphaZero` Player.
#[derive(Clone)]
pub struct SearchTree<
    Policy: TreePolicy,
    const AMOUNT_PATCH_LAYERS: usize,
    const AMOUNT_RESIDUAL_LAYERS: usize,
    const AMOUNT_FILTERS: usize,
> {
    /// The network to use to evaluate the game states. Moved to Search Data during the search.
    pub(crate) network: Option<PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>>,
    /// The options to use for the search tree.
    options: Rc<AlphaZeroOptions>,
    /// The dirichlet noise distribution to use for the root node.
    dirichlet_noise: Dirichlet<f32>,
    /// The epsilon value for the Dirichlet noise. This is the fraction of the noise to add to the policy.
    dirichlet_epsilon: f32,
    /// Whether the network is in training or evaluation/interference mode
    train: bool,
    /// The policy to select nodes during the selection phase. Moved to Search Data during the search.
    tree_policy: Option<Policy>,
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
        dirichlet_alpha: f32,
        dirichlet_epsilon: f32,
    ) -> Self {
        let dirichlet_noise =
            Dirichlet::new_with_size(dirichlet_alpha, NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS)
                .expect("[SearchData::new] Failed to create dirichlet noise distribution");

        Self {
            train,
            dirichlet_epsilon,
            dirichlet_noise,
            tree_policy: Some(tree_policy),
            network: Some(network),
            options,
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
    pub fn search(&mut self, games: &[&Patchwork]) -> PlayerResult<Tensor> {
        let start_time = std::time::Instant::now();

        // 1. Create the root nodes
        let batch = self.create_root_nodes(games)?;

        // 2. Create the search data
        let search_data = SearchData::new(
            self.options.as_ref(),
            self.train,
            self.network.take().unwrap(),
            self.tree_policy.take().unwrap(),
            batch,
        );

        // 3. Start searching with threads
        std::thread::scope(|s| {
            let flag = Arc::new(AtomicBool::new(false));

            let handles = (0..self.options.parallelization.get() - 1)
                .map(|_| {
                    let search_data = &search_data;
                    let worker_flag = Arc::clone(&flag);
                    s.spawn(move || {
                        let worker = WorkerThread { search_data };
                        while !worker_flag.load(std::sync::atomic::Ordering::Acquire) {
                            worker.iteration()?;
                        }
                        PlayerResult::Ok(())
                    })
                })
                .collect::<Vec<_>>();

            let worker = WorkerThread {
                search_data: &search_data,
            };

            self.run_until_end(start_time, &worker)?;

            flag.store(true, std::sync::atomic::Ordering::Release);

            for handle in handles {
                handle.join().unwrap()?;
            }

            // 4. Force last mini-batch evaluation
            worker.do_mini_batch_evaluation(true)?;

            PlayerResult::Ok(())
        })?;

        // TODO:
        // println!(
        //     "Iterations: {}",
        //     search_data.iterations.load(std::sync::atomic::Ordering::SeqCst)
        // );

        // 5. Reset search data
        let SearchData {
            tree_policy,
            network,
            batch,
            ..
        } = search_data;
        self.network = Some(network);
        self.tree_policy = Some(tree_policy);

        // 6. Return the action probabilities for each game state using the visit counts
        self.get_action_probabilities(&batch)
    }

    /// Runs the search until the end condition is met.
    ///
    /// # Arguments
    ///
    /// * `start_time` - The time when the search started.
    /// * `worker` - The worker to use for the search.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the search was run successfully, `Err(PatchworkError)` otherwise.
    fn run_until_end(
        &self,
        start_time: std::time::Instant,
        worker: &WorkerThread<'_, Policy, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
    ) -> PlayerResult<()> {
        match &self.options.end_condition {
            AlphaZeroEndCondition::Iterations { iterations } => {
                while worker.search_data.iterations.load(std::sync::atomic::Ordering::Relaxed) < *iterations {
                    worker.iteration()?;
                }
            }
            AlphaZeroEndCondition::Time {
                duration,
                safety_margin,
            } => {
                let duration = *duration - *safety_margin;

                while start_time.elapsed() < duration {
                    worker.iteration()?;
                }
            }
            AlphaZeroEndCondition::Flag { flag } => {
                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    worker.iteration()?;
                }
            }
        }
        Ok(())
    }

    /// Creates the root node for the game states. Also expands the root node by adding all possible child nodes.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the root node was created successfully, `Err(PatchworkError)` otherwise.
    fn create_root_nodes(&self, games: &[&Patchwork]) -> PlayerResult<Vec<GameState>> {
        let (policies, _values) = self.network.as_ref().unwrap().forward_t(games, self.train)?;
        let mut policies = candle_nn::ops::softmax(&policies, 1)?.detach();

        if self.train {
            let noise = Tensor::from_vec(
                self.dirichlet_noise.sample(&mut rand::thread_rng()),
                (NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,),
                &self.options.device,
            )?
            .detach();

            policies = Tensor::broadcast_add(
                &Tensor::broadcast_mul(
                    &Tensor::new(1.0 - self.dirichlet_epsilon, &self.options.device)?,
                    &policies,
                )?,
                &Tensor::broadcast_mul(&Tensor::new(self.dirichlet_epsilon, &self.options.device)?, &noise)?,
            )?
            .detach();
        }

        let (available_actions_tensor, mut corresponding_action_ids) =
            map_games_to_action_tensors(games, &self.options.device)?;
        let policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum_keepdim(1)?;
        let policies = policies.broadcast_div(&policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;

        games
            .iter()
            .enumerate()
            .map(|(index, state)| {
                let allocator = AreaAllocator::new();
                let root = allocator.new_node((*(*state)).clone(), None, None, None);
                let policy = &policies[index];
                let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

                Node::expand(root, policy, &corresponding_actions, &allocator)?;

                Ok(GameState { allocator, root })
            })
            .collect::<PlayerResult<Vec<_>>>()
    }

    /// Returns the action probabilities for each game state using the visit counts.
    ///
    /// # Arguments
    ///
    /// * `batch` - The batch of game states to get the action probabilities for.
    ///
    /// # Returns
    ///
    /// The action probabilities for each game state.

    fn get_action_probabilities(&self, batch: &[GameState]) -> PlayerResult<Tensor> {
        Ok(Tensor::stack(
            batch
                .iter()
                .map(|state| {
                    let mut action_probabilities = [0.0; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];

                    let root_node_id = state.root;
                    let root_node = state.allocator.get_node_read(root_node_id);
                    for node_id in &root_node.children {
                        let node = state.allocator.get_node_read(*node_id);

                        if let Some(action_taken) = node.action_taken {
                            let natural_action_id = action_taken.to_natural_action_id().as_bits() as usize;
                            action_probabilities[natural_action_id] = node.visit_count as f32;
                        }
                    }
                    drop(root_node);

                    let sum = action_probabilities.iter().sum::<f32>();
                    for probability in &mut action_probabilities {
                        *probability /= sum;
                    }

                    Tensor::from_slice(
                        &action_probabilities,
                        (NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,),
                        &self.options.device,
                    )
                })
                .collect::<candle_core::Result<Vec<_>>>()?
                .as_slice(),
            0,
        )?)
    }
}
