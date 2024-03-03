use std::sync::{atomic::Ordering, Arc};

use candle_core::Device;
use dashmap::DashMap;
use patchwork_core::{PlayerResult, TreePolicy};

use crate::{
    action::map_games_to_action_tensors,
    mcts::{Node, SearchData},
};

pub struct WorkerThread<
    'worker,
    Policy: TreePolicy,
    const AMOUNT_PATCH_LAYERS: usize,
    const AMOUNT_RESIDUAL_LAYERS: usize,
    const AMOUNT_FILTERS: usize,
> {
    pub search_data: &'worker SearchData<Policy, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
}

impl<
        'worker,
        Policy: TreePolicy,
        const AMOUNT_PATCH_LAYERS: usize,
        const AMOUNT_RESIDUAL_LAYERS: usize,
        const AMOUNT_FILTERS: usize,
    > WorkerThread<'worker, Policy, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>
{
    /// Runs a single iteration of the MCTS algorithm. Will run until a node is encountered that is not expanded. Nodes
    /// with terminal states are evaluated directly and backpropagated and other nodes are inserted into the mini-batch
    /// evaluation list. May do a mini-batch evaluation if enough states are available.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the iteration was run successfully, `Err(PatchworkError)` otherwise.
    pub fn iteration(&self) -> PlayerResult<()> {
        self.search_data.iterations.fetch_add(1, Ordering::Relaxed);

        for (batch_index, game_state) in self.search_data.batch.iter().enumerate() {
            let mut node_id = game_state.root;
            loop {
                let node = game_state.allocator.get_node_read(node_id);
                node.increment_virtual_loss();

                if !node.is_fully_expanded() {
                    break;
                }

                drop(node);
                node_id = Node::select(node_id, &game_state.allocator, &self.search_data.tree_policy);
            }

            let node = game_state.allocator.get_node_read(node_id);
            if node.state.is_terminated() {
                #[allow(clippy::significant_drop_in_scrutinee)]
                let value = match node.state.get_termination_result().termination {
                    patchwork_core::TerminationType::Player1Won => 1.0,
                    patchwork_core::TerminationType::Player2Won => -1.0,
                };
                drop(node);
                Node::backpropagate(node_id, value, &game_state.allocator, 1);
            } else {
                // add current node to allow for batch evaluation with the network
                self.search_data
                    .evaluation_mini_batch
                    .load()
                    .entry((batch_index, node_id))
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                self.search_data.mini_batch_counter.fetch_add(1, Ordering::Relaxed);
            }
        }

        self.do_mini_batch_evaluation(false)?;

        Ok(())
    }

    /// Evaluates the batch states with the network and backpropagates the results.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to force the evaluation of the batch states even if there are less batch states than the
    ///   batch size.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the batch evaluation was run successfully, `Err(PatchworkError)` otherwise.
    pub fn do_mini_batch_evaluation(&self, force: bool) -> PlayerResult<()> {
        if !force
            && self
                .search_data
                .mini_batch_counter
                .fetch_update(Ordering::Release, Ordering::Acquire, |value| {
                    if value >= self.search_data.mini_batch_size {
                        Some(0)
                    } else {
                        None
                    }
                })
                .is_err()
        {
            return Ok(());
        }

        // Atomically swap the evaluation mini batch to a new empty one
        let evaluation_mini_batch = self
            .search_data
            .evaluation_mini_batch
            .swap(Arc::new(DashMap::with_capacity(self.search_data.mini_batch_size)));

        // println!(
        //     "Mini batch Eval ({:?} entries, {:?} values) {:?}",
        //     evaluation_mini_batch.len(),
        //     evaluation_mini_batch.iter().map(|entry| *entry.value()).sum::<i32>(),
        //     evaluation_mini_batch
        //         .iter()
        //         .map(|entry| *entry.value())
        //         .collect::<Vec<_>>()
        // );

        if evaluation_mini_batch.is_empty() {
            return Ok(());
        }

        let nodes = evaluation_mini_batch
            .iter()
            .map(|entry| {
                let (batch_index, node_id) = *entry.key();
                self.search_data.batch[batch_index].allocator.get_node_read(node_id)
            })
            .collect::<Vec<_>>();
        let games = nodes.iter().map(|node| &node.state).collect::<Vec<_>>();

        let (available_actions_tensor, mut corresponding_action_ids) =
            map_games_to_action_tensors(&games, &self.search_data.device)?;
        let available_actions_tensor = available_actions_tensor.detach();

        // TODO: diagnostics
        // let start = std::time::Instant::now();
        let (policies, values) = self.search_data.network.forward_t(&games, self.search_data.train)?;
        drop(games);
        drop(nodes);

        let mut policies = policies.detach();
        let values = values.detach();

        policies = candle_nn::ops::softmax(&policies, 1)?;
        policies = (policies * available_actions_tensor)?;
        let policies_sum = policies.sum_keepdim(1)?;
        policies = policies.broadcast_div(&policies_sum)?;
        let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;
        let values = values.to_device(&Device::Cpu)?.to_vec1::<f32>()?;

        debug_assert_eq!(
            policies.len(),
            values.len(),
            "policies.len() {:?} != values.len() {:?}",
            policies.len(),
            values.len()
        );

        // Dump all policies along with their corresponding action to file
        // let mut file = std::fs::OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .append(false)
        //     .open(format!(
        //         "values_{:?}.txt",
        //         std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        //     ))
        //     .unwrap();
        // writeln!(file, "[").unwrap(); // TODO: remove
        // for action_index in 0..NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS {
        //     for mini_batch_index in 0..policies.len() {
        //         // ActionId(88839) 15
        //         let policy = policies[mini_batch_index][action_index];
        //         let corresponding_action_id = corresponding_action_ids[mini_batch_index][action_index];
        //         write!(file, "      {: <15?} {: <13?}", corresponding_action_id, policy).unwrap();
        //     }
        //     writeln!(file).unwrap();
        // }
        // writeln!(file, "]").unwrap();

        for (mini_batch_index, entry) in evaluation_mini_batch.iter().enumerate() {
            let (batch_index, node_id) = *entry.key();
            let amount = *entry.value();

            let policy = &policies[mini_batch_index];
            let value = values[mini_batch_index];
            let corresponding_actions = corresponding_action_ids.pop_front().unwrap();

            let node = self.search_data.batch[batch_index].allocator.get_node_write(node_id);
            let color = if node.state.is_player_1() { 1.0 } else { -1.0 };
            drop(node);

            let value = color * value;

            Node::expand(
                node_id,
                policy,
                &corresponding_actions,
                &self.search_data.batch[batch_index].allocator,
            )?;
            Node::backpropagate(node_id, value, &self.search_data.batch[batch_index].allocator, amount);
        }

        Ok(())
    }
}
