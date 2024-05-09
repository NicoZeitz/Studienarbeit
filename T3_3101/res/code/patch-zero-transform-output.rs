let (mut policies, values) = self.search_data.network
    .forward_t(&games, self.search_data.train)?;
policies = candle_nn::ops::softmax(&policies, 1)?;
policies = (policies * available_actions_tensor)?;
let policies_sum = policies.sum_keepdim(1)?;
policies = policies.broadcast_div(&policies_sum)?;
let policies = policies.to_device(&Device::Cpu)?.to_vec2::<f32>()?;
let values = values.to_device(&Device::Cpu)?.to_vec1::<f32>()?;
