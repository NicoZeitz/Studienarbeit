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
    device: Device,
    args: TrainingArgs,
    training_directory: PathBuf,
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
    /// * `device` - The device to use for the trainer.
    /// * `args` - The arguments to use for training the neural network.
    /// * `training_directory` - The path to the directory to save and load the training data to and from.
    pub fn new<P: AsRef<std::path::Path>>(training_directory: P, args: TrainingArgs, device: Device) -> Self {
        Self {
            training_directory: training_directory.as_ref().to_path_buf(),
            args,
            device,
        }
    }

    pub fn learn<Policy: TreePolicy + Default>(&self) -> PlayerResult<()> {
        let (var_map, starting_index) = self.get_var_map()?;

        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &self.device);

        let alphazero_options = Rc::new(AlphaZeroOptions {
            device: self.device.clone(),
            batch_size: NonZeroUsize::new(20 * self.args.batch_size).unwrap(),
            end_condition: AlphaZeroEndCondition::Iterations {
                iterations: self.args.number_of_mcts_iterations,
            },
            logging: Logging::Disabled,
            ..Default::default()
        });

        let network = DefaultPatchZero::new(var_builder, self.device.clone())?;
        let mut search_tree = DefaultSearchTree::<Policy>::new(
            false, // Will be set later anyways
            Default::default(),
            network,
            alphazero_options,
            self.args.dirichlet_alpha,
            self.args.dirichlet_epsilon,
        );
        let mut optimizer = self.get_optimizer(&var_map, starting_index)?;

        println!("Started training at {starting_index} with {:?}", self.args);

        for _ in 0..self.args.number_of_training_iterations {
            let mut history = vec![];

            search_tree.set_train(false);
            let iterations = (self.args.number_of_self_play_iterations as f64
                / self.args.number_of_parallel_games as f64)
                .ceil() as usize;
            for _ in tqdm(0..iterations) {
                history.extend(self.self_play(&mut search_tree)?);
            }

            search_tree.set_train(true);
            for _ in tqdm(0..self.args.number_of_epochs) {
                self.train(&mut history, search_tree.network.as_ref().unwrap(), &mut optimizer)?;
            }

            let index = iterations + starting_index;
            println!("Finished iteration {index}");

            let network_weights = self
                .training_directory
                .with_file_name(format!("network_{index:04}.safetensors"));
            let optimizer_weights = self
                .training_directory
                .with_file_name(format!("optimizer_{index:04}.safetensors"));

            var_map.save(network_weights)?;
            optimizer.save(optimizer_weights)?;
        }

        Ok(())
    }

    fn self_play(&self, search_tree: &mut DefaultSearchTree<impl TreePolicy>) -> PlayerResult<Vec<History>> {
        struct PartialHistory {
            state: Patchwork,
            policy: Tensor,
        }

        let mut return_history = vec![];
        let mut history = (0..self.args.number_of_parallel_games)
            .map(|_| vec![])
            .collect::<Vec<_>>();
        let mut games = (0..self.args.number_of_parallel_games)
            .map(|_| Patchwork::get_initial_state(None))
            .collect::<Vec<_>>();

        let temperature_tensor = Tensor::new(1.0 / self.args.temperature, &self.device)?;

        while !games.is_empty() {
            let policies = search_tree.search(games.iter().collect::<Vec<_>>().as_slice())?;

            let (available_actions_tensor, mut corresponding_action_ids) =
                map_games_to_action_tensors(games.iter().collect::<Vec<_>>().as_slice(), &self.device)?;

            let policies = (policies * available_actions_tensor)?;
            let policies_sum = policies.sum_keepdim(1)?;
            let policies = policies.broadcast_div(&policies_sum)?;

            for i in (0..games.len()).rev() {
                let game = &mut games[i];
                let policy = policies.i((i, ..))?;
                let actions = corresponding_action_ids.pop_back().unwrap();

                history[i].push(PartialHistory {
                    state: game.clone(),
                    policy: policy.clone().detach()?,
                });

                let temperature_action_probabilities = policy.pow(&temperature_tensor)?;
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
        network: &DefaultPatchZero,
        optimizer: &mut impl Optimizer,
    ) -> PlayerResult<()> {
        fn get_value_from_outcome(outcome: TerminationType, is_current_player_1: bool, device: &Device) -> Tensor {
            let multiplier = if is_current_player_1 { 1.0 } else { -1.0 };
            // TODO: label smoothing
            match outcome {
                TerminationType::Player1Won => Tensor::new(multiplier * 1.0, device).unwrap(),
                TerminationType::Player2Won => Tensor::new(multiplier * -1.0, device).unwrap(),
            }
        }

        training_set.shuffle(&mut thread_rng());

        for batch in training_set.chunks(self.args.batch_size) {
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

            let policy_loss = candle_nn::loss::cross_entropy(&out_policies, &policy_targets)?;
            let value_loss = candle_nn::loss::mse(&out_values, &value_targets)?;
            let _regularization_loss = Tensor::new(0.01, &self.device); // TODO: L2 regularization
            let loss = (policy_loss + value_loss)?;

            optimizer.backward_step(&loss)?;
        }

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
        let mut optimizer = AdamW::new(
            var_map.all_vars(),
            ParamsAdamW {
                lr: self.args.learning_rate,
                ..Default::default()
            },
        )?;

        if starting_epoch == 0 {
            return Ok(optimizer);
        }

        let file_name = format!("optimizer_{starting_epoch:04}.safetensors");
        let file = self.training_directory.with_file_name(file_name);

        optimizer.load(file)?;

        Ok(optimizer)
    }
}
