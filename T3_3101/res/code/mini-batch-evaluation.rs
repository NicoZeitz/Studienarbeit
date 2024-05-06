pub fn do_mini_batch_evaluation(&self, force: bool) -> PlayerResult<()> {
  if !force && self.search_data.mini_batch_counter
    .fetch_update(Ordering::Release, Ordering::Acquire, |value| {
    if value >= self.search_data.mini_batch_size { Some(0) } 
    else { None }
  }).is_err() {
    return Ok(());
  }
  // Atomically swap the eval minibatch to a new empty one
  let evaluation_mini_batch = self.search_data.evaluation_mini_batch
    .swap(Arc::new(DashMap::with_capacity( self.search_data.mini_batch_size)));
  
  if evaluation_mini_batch.is_empty() {
    return Ok(());
  }

  let evaluation_mini_batch = evaluation_mini_batch.iter()
    .map(|entry| (*entry.key(), *entry.value()))
    .collect::<HashMap<_, _>>();

  let games = evaluation_mini_batch.iter().map(|((batch_index, node_id), _amount)| { self.search_data.batch[*batch_index].allocator .get_node_read(*node_id).state.clone() }).collect::<Vec<_>>();
  let games = games.iter().collect::<Vec<_>>();

  let (actions_tensor, mut corresponding_action_ids) = map_games_to_action_tensors(&games, &self.search_data.device)?;
  let actions_tensor = actions_tensor.detach();

  let (policies, values) = self.search_data.network
    .forward_t(&games, self.search_data.train)?;

  let mut policies = policies.detach();
  let values = values.detach();
  policies = candle_nn::ops::softmax(&policies, 1)?;
  policies = (policies * actions_tensor)?;
  let policies_sum = policies.sum_keepdim(1)?;
  policies = policies.broadcast_div(&policies_sum)?;
  let policies = policies
    .to_device(&Device::Cpu)?.to_vec2::<f32>()?;
  let values = values
    .to_device(&Device::Cpu)?.to_vec1::<f32>()?;

  for (mini_batch_index, ((batch_index, node_id), amount))
    in evaluation_mini_batch.iter().enumerate() {
    let policy = &policies[mini_batch_index];
    let value = values[mini_batch_index];
    let corresponding_actions = corresponding_action_ids
      .pop_front().unwrap();

    let node = self.search_data.batch[*batch_index]
      .allocator.get_node_write(*node_id);
    let color = if node.state.is_player_1() { 1.0 }
                else { -1.0 };
    drop(node);

    let value = color * value;
    Node::expand(
        *node_id,
        policy,
        &corresponding_actions,
        &self.search_data.batch[*batch_index].allocator,
    )?;
    Node::backpropagate(
        *node_id,
        value,
        &self.search_data.batch[*batch_index].allocator,
        *amount,
    );
  }
  Ok(())
}
