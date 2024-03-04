use std::{fs, num::NonZeroUsize, path::PathBuf, rc::Rc};

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{Optimizer, VarBuilder, VarMap};
use patchwork_core::{Logging, Patchwork, PlayerResult, TerminationType, TreePolicy};
use rand::{seq::SliceRandom, thread_rng};
use rand_distr::{Distribution, WeightedIndex};
use regex::Regex;
use tqdm::tqdm;

use crate::{
    action::map_games_to_action_tensors,
    mcts::DefaultSearchTree,
    network::DefaultPatchZero,
    train::{
        optimizer::{AdamW, ParamsAdamW},
        TrainingArgs,
    },
    AlphaZeroEndCondition, AlphaZeroOptions,
};

pub struct Trainer {
    pub device: Device,
    pub args: TrainingArgs,
    pub training_directory: PathBuf,
}

struct History {
    state: Patchwork,
    policy: Tensor,
    outcome: TerminationType,
}

impl Trainer {
    /// Creates a new [`Trainer`].
    ///
    /// # Arguments
    ///
    /// * `training_directory` - The path to the directory to save and load the training data to and from.
    /// * `args` - The arguments to use for training the neural network.
    /// * `device` - The device to use.
    pub fn new<P: AsRef<std::path::Path>>(training_directory: P, args: TrainingArgs, device: Device) -> Self {
        Self {
            training_directory: training_directory.as_ref().to_path_buf(),
            args,
            device,
        }
    }

    pub fn learn<Policy: TreePolicy + Default>(&self) -> PlayerResult<()> {
        let (var_map, starting_index) = self.get_var_map()?;
        let mut optimizer = self.get_optimizer(&var_map, starting_index)?;

        println!("Started training at {starting_index} with {:?}", self.args);

        for iteration in 0..self.args.number_of_training_iterations {
            let mut history = vec![];

            let iterations = (self.args.number_of_self_play_iterations as f64
                / self.args.number_of_parallel_games as f64)
                .ceil() as usize;
            for _ in tqdm(0..iterations).style(tqdm::Style::Block).desc(Some("Self-Play")) {
                loop {
                    if let Ok(partial_history) = self.self_play::<Policy>(&var_map) {
                        history.extend(partial_history);
                        break;
                    }
                    println!("Self-play failed, retrying...");
                }
            }

            for _ in tqdm(0..self.args.number_of_epochs)
                .style(tqdm::Style::Block)
                .desc(Some("Train"))
            {
                self.train(&mut history, &mut optimizer, &var_map)?;
            }

            let index = iteration + starting_index + 1;
            println!(
                "Finished iteration {index}. Saving weights to {:?}",
                self.training_directory
            );

            let network_weights = self.training_directory.join(format!("network_{index:04}.safetensors"));
            let optimizer_weights = self
                .training_directory
                .join(format!("optimizer_{index:04}.safetensors"));

            var_map.save(network_weights)?;
            optimizer.save(optimizer_weights)?;
        }

        Ok(())
    }

    fn self_play<Policy: TreePolicy + Default>(&self, var_map: &VarMap) -> PlayerResult<Vec<History>> {
        struct PartialHistory {
            state: Patchwork,
            policy: Tensor,
        }

        let var_builder = VarBuilder::from_varmap(var_map, DType::F32, &self.device);
        let alphazero_options = Rc::new(AlphaZeroOptions {
            device: self.device.clone(),
            parallelization: NonZeroUsize::new((AlphaZeroOptions::default().parallelization.get() - 1).max(1)).unwrap(),
            batch_size: NonZeroUsize::new(20 * self.args.number_of_parallel_games).unwrap(),
            end_condition: AlphaZeroEndCondition::Iterations {
                iterations: self.args.number_of_mcts_iterations,
            },
            logging: Logging::Disabled,
        });

        let network = DefaultPatchZero::new(var_builder, self.device.clone())?;
        let mut search_tree = DefaultSearchTree::<Policy>::new(
            false,
            Default::default(),
            network,
            alphazero_options,
            self.args.dirichlet_alpha,
            self.args.dirichlet_epsilon,
        );

        let mut return_history = vec![];
        let mut history = (0..self.args.number_of_parallel_games)
            .map(|_| vec![])
            .collect::<Vec<_>>();
        let mut games = (0..self.args.number_of_parallel_games)
            .map(|_| Patchwork::get_initial_state(None))
            .collect::<Vec<_>>();

        let temperature_tensor = Tensor::new(1.0 / self.args.temperature, &self.device)?;

        while !games.is_empty() {
            let policies = search_tree
                .search(games.iter().collect::<Vec<_>>().as_slice())?
                .detach();

            let (available_actions_tensor, mut corresponding_action_ids) =
                map_games_to_action_tensors(games.iter().collect::<Vec<_>>().as_slice(), &self.device)?;
            let available_actions_tensor = available_actions_tensor.detach();

            let policies = (policies * available_actions_tensor)?;
            let policies_sum = policies.sum_keepdim(1)?;
            let policies = policies.broadcast_div(&policies_sum)?;

            for i in (0..games.len()).rev() {
                let game = &mut games[i];
                let policy = policies.i((i, ..))?;
                let actions = corresponding_action_ids.pop_back().unwrap();

                history[i].push(PartialHistory {
                    state: game.clone(),
                    policy: policy.clone().detach().to_device(&self.device)?,
                });

                let temperature_action_probabilities = if history[i].len() >= self.args.temperature_end {
                    policy
                } else {
                    policy.broadcast_pow(&temperature_tensor)?
                };
                let dist = WeightedIndex::new(temperature_action_probabilities.to_vec1::<f32>()?)?;

                let action = actions[dist.sample(&mut thread_rng())];
                game.do_action(action, false)?;

                if game.is_terminated() {
                    let history = std::mem::take(&mut history[i]);
                    return_history.extend(history.into_iter().map(|partial_history| History {
                        state: partial_history.state,
                        policy: partial_history.policy,
                        outcome: game.get_termination_result().termination,
                    }));

                    games.swap_remove(i);
                }
            }
        }

        Ok(return_history)
    }

    fn train(
        &self,
        training_set: &mut [History],
        optimizer: &mut impl Optimizer,
        var_map: &VarMap,
    ) -> PlayerResult<()> {
        fn get_value_from_outcome(outcome: TerminationType, is_current_player_1: bool, device: &Device) -> Tensor {
            let multiplier: f32 = if is_current_player_1 { 1.0 } else { -1.0 };
            match outcome {
                TerminationType::Player1Won => Tensor::new(multiplier * 1.0, device).unwrap(),
                TerminationType::Player2Won => Tensor::new(multiplier * -1.0, device).unwrap(),
            }
        }

        let var_builder = VarBuilder::from_varmap(var_map, DType::F32, &self.device);
        let network = DefaultPatchZero::new(var_builder, self.device.clone())?;

        training_set.shuffle(&mut thread_rng());

        let mut last_loss = 0.0;
        let mut last_policy_loss = 0.0;
        let mut last_value_loss = 0.0;
        let mut last_regularization_loss = 0.0;

        for batch in tqdm(training_set.chunks(self.args.batch_size))
            .style(tqdm::Style::Block)
            .desc(Some("Batch"))
            .clear(true)
        {
            let (games, policy_targets, value_targets) = batch.iter().fold(
                (vec![], vec![], vec![]),
                |(mut games, mut policy_targets, mut value_targets), history| {
                    let value_target =
                        get_value_from_outcome(history.outcome, history.state.is_player_1(), &self.device);

                    games.push(history.state.clone());
                    policy_targets.push(history.policy.clone());
                    value_targets.push(value_target);

                    (games, policy_targets, value_targets)
                },
            );

            let policy_targets = Tensor::stack(&policy_targets, 0)?;
            let value_targets = Tensor::stack(&value_targets, 0)?;

            let (out_policies, out_values) = network.forward_t(games.iter().collect::<Vec<_>>().as_slice(), true)?;

            let policy_loss = multi_target_cross_entropy_loss(&out_policies, &policy_targets)?;
            let value_loss = candle_nn::loss::mse(&out_values, &value_targets)?;
            let regularization_loss = Tensor::stack(
                var_map
                    .all_vars()
                    .iter()
                    .map(|var| var.as_tensor().sqr()?.sum_all())
                    .collect::<candle_core::Result<Vec<_>>>()?
                    .as_slice(),
                0,
            )?
            .sum_all()?
            .affine(self.args.regularization, 0.0)?;

            last_policy_loss = policy_loss.to_scalar::<f32>()?;
            last_value_loss = value_loss.to_scalar::<f32>()?;
            last_regularization_loss = regularization_loss.to_scalar::<f32>()?;

            let loss = (policy_loss + value_loss + regularization_loss)?;

            last_loss = loss.to_scalar::<f32>()?;

            #[allow(clippy::manual_assert)]
            if !last_loss.is_finite() {
                panic!("Loss is infinite or NaN (try reducing the learning rate or increasing the batch_size)");
            }

            optimizer.backward_step(&loss)?;
        }

        println!(
            "\x1B[1A\rPolicy loss: {last_policy_loss}, Value loss: {last_value_loss}, L² loss: {last_regularization_loss}, Total loss: {last_loss}"
        );

        Ok(())
    }

    fn get_var_map(&self) -> PlayerResult<(VarMap, usize)> {
        let network_regex = Regex::new(r"network_(?P<epoch>\d{4}).safetensors").unwrap();

        let mut starting_index = 0;
        let mut network_weights = None;

        for file in fs::read_dir(self.training_directory.as_path())? {
            let Ok(dir_entry) = file else {
                continue;
            };

            let Ok(metadata) = dir_entry.metadata() else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            let file_name = dir_entry.file_name();
            let file_name = file_name.to_string_lossy();

            if let Some(captures) = network_regex.captures(&file_name) {
                let epoch = captures.name("epoch").unwrap().as_str().parse::<usize>().unwrap();

                if epoch < starting_index {
                    continue;
                }

                starting_index = epoch;
                network_weights = Some(dir_entry.path());
            }
        }
        let mut var_map = VarMap::new();

        if let Some(network_weights) = network_weights {
            var_map.load(network_weights)?;
        }

        Ok((var_map, starting_index))
    }

    fn get_optimizer(&self, var_map: &VarMap, starting_epoch: usize) -> PlayerResult<AdamW> {
        // force consistent ordering for saving and loading
        let guard = var_map.data().lock().unwrap();
        let mut all_vars = guard.iter().collect::<Vec<_>>();
        all_vars.sort_unstable_by(|entry1, entry2| entry1.0.cmp(entry2.0));
        let all_vars = all_vars.iter().map(|entry| entry.1.clone()).collect::<Vec<_>>();
        drop(guard);

        let mut optimizer = AdamW::new(
            all_vars,
            ParamsAdamW {
                lr: self.args.learning_rate,
                ..Default::default()
            },
        )?;

        if starting_epoch != 0 {
            return Ok(optimizer);
        }

        // TODO: get the optimizer to load from a safetensors file
        if starting_epoch == 0 {
            return Ok(optimizer);
        }

        let file_name = format!("optimizer_{starting_epoch:04}.safetensors");
        let file = self.training_directory.join(file_name);

        optimizer.load(file)?;

        Ok(optimizer)
    }
}

/// Computes the multi-target cross-entropy loss.
///
/// # Arguments
///
/// * `input` - The input tensor of dimensions `N, C` where `N` is the batch size and `C` the number
///            of categories.
/// * `target` - The ground truth labels as a tensor of dimensions `N, C` where `N` is the batch size and `C` the number
///           of categories.
///
/// # Formula
///
/// ```math
/// input = [
///     [1, 2, 3],
///     [2, 4, 8],
/// ]
/// targets = [
///     [0, 1, 0],
///     [0, 0.5, 0.5]
/// ]
/// softmax = [
///     [0.09, 0.24, 0.67],
///     [0.02, 0.18, 0.8],
/// ]
/// log_softmax = [
///     [-2.4, -1.4, -0.4],
///     [-3.9, -1.7, -0.2],
/// ]
/// multiplied = [
///    [0, -1.4, 0],
///    [0, -0.85, -0.1]
/// ]
/// loss = -1 * (0 + -1.4 + 0 + 0 + -0.85 + -0.1) / 2 = 1.175
/// ```
///
/// # Returns
///
/// The resulting tensor is a scalar containing the average value over the batch.
fn multi_target_cross_entropy_loss(input: &Tensor, target: &Tensor) -> candle_core::Result<Tensor> {
    const EPSILON: f64 = f32::MIN_POSITIVE as f64;

    let input = candle_nn::ops::log_softmax(input, 1)?.affine(1.0, EPSILON)?;
    let batch_size = target.dims()[0] as f64;

    let mask = target.ne(0.0)?;
    mask.where_cond(&(input * target)?, &target.zeros_like()?)?
        .sum_all()?
        .affine(-1f64 / batch_size, 0.)
}
