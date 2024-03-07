use std::{
    fs::{self, OpenOptions},
    io::Write,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        mpsc::{Sender, TryRecvError},
        Arc, Mutex,
    },
};

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
    current_var_map: Arc<Mutex<VarMap>>,
    starting_index: usize,
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
    pub fn new<P: AsRef<Path>>(training_directory: P, args: TrainingArgs, device: Device) -> PlayerResult<Self> {
        let (var_map, starting_index) = get_var_map(training_directory.as_ref())?;

        Ok(Self {
            training_directory: training_directory.as_ref().to_path_buf(),
            args,
            device,
            current_var_map: Arc::new(Mutex::new(var_map)),
            starting_index,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn learn<Policy: TreePolicy + Default + Clone>(&self) -> PlayerResult<()> {
        let starting_index = self.starting_index;

        let log_path = self.training_directory.join(format!("log_{starting_index:04?}.txt"));
        let log_path = log_path.as_path();

        let mut log_file = OpenOptions::new().append(true).create(true).open(log_path)?;

        writeln!(log_file, "Started training at {starting_index} with {:?}", self.args)?;

        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::scope(|s| {
            let mut threads: Vec<std::thread::ScopedJoinHandle<'_, PlayerResult<()>>> =
                Vec::with_capacity(self.args.number_of_parallel_games);
            for _ in 0..self.args.number_of_parallel_games {
                threads.push(s.spawn(|| loop {
                    let var_map = self.current_var_map.lock().unwrap().clone();
                    let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &self.device);
                    let alphazero_options = Rc::new(AlphaZeroOptions {
                        device: self.device.clone(),
                        parallelization: NonZeroUsize::new(1).unwrap(),
                        end_condition: AlphaZeroEndCondition::Iterations {
                            iterations: self.args.number_of_mcts_iterations,
                        },
                        logging: Logging::Disabled,
                        ..AlphaZeroOptions::default()
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
                    search_tree.set_dirichlet_noise(true);

                    let sender = sender.clone();

                    if let Err(error) = self.self_play(search_tree, sender) {
                        let mut log_file = OpenOptions::new().append(true).create(true).open(log_path)?;
                        writeln!(
                            log_file,
                            "Self-play at Thread {:?} failed, retrying...",
                            std::thread::current().id()
                        )?;
                        writeln!(log_file, "{error:?}")?;
                        continue;
                    };
                }));
            }

            let mut history = Vec::new();
            let mut index = starting_index + 1;

            // Block at first
            while history.len() < self.args.batch_size {
                history.push(receiver.recv()?);
            }

            'outer: loop {
                loop {
                    match receiver.try_recv() {
                        Ok(finished_game) => {
                            history.push(finished_game);
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break 'outer,
                    }
                }
                while history.len() > self.args.training_set_size {
                    history.remove(0);
                }

                writeln!(log_file, "Training iteration {index}")?;

                let var_map = VarMap::new();
                let mut new_data = var_map.data().lock().unwrap();

                #[allow(clippy::significant_drop_in_scrutinee)]
                for (name, var) in self.current_var_map.lock().unwrap().data().lock().unwrap().iter() {
                    new_data.insert(
                        name.clone(),
                        candle_core::Var::from_tensor(var.as_detached_tensor().as_ref())?,
                    );
                }
                drop(new_data);

                let mut optimizer = get_optimizer(&var_map, self.args.learning_rate)?;
                let mut train_sample = history
                    .choose_multiple(&mut thread_rng(), self.args.training_sample_size)
                    .collect::<Vec<_>>();

                for _ in tqdm(0..self.args.number_of_epochs) {
                    self.train(&mut train_sample, &mut optimizer, &var_map, &mut log_file)?;
                }
               
                writeln!(
                    log_file,
                    "Finished iteration {index}. Saving weights to {:?}",
                    self.training_directory
                )?;
                let network_weights = self.training_directory.join(format!("network_{index:04}.safetensors"));
                var_map.save(network_weights)?;
                index += 1;
                let mut mutex = self.current_var_map.lock().unwrap();
                *mutex = var_map;
            }

            for thread in threads {
                thread.join().unwrap()?;
            }

            Ok(())
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn self_play<Policy: TreePolicy>(
        &self,
        mut search_tree: DefaultSearchTree<Policy>,
        channel: Sender<History>,
    ) -> PlayerResult<()> {
        struct PartialHistory {
            state: Patchwork,
            policy: Tensor,
        }

        let temperature_tensor = Tensor::new(1.0 / self.args.temperature, &self.device)?;
        let mut iteration = 0;

        loop {
            iteration += 1;
            let mut history = vec![];
            let mut game = Patchwork::get_initial_state(None);

            for _ in tqdm(0..64)
                .style(tqdm::Style::Block)
                .desc(Some(format!(
                    "Self-Play {:?} G {:05?}",
                    std::thread::current().id(),
                    iteration
                )))
                .clear(true)
            {
                if game.is_terminated() {
                    continue;
                }

                let policies = search_tree.search(&[&game])?.detach();

                let (available_actions_tensor, mut corresponding_action_ids) =
                    map_games_to_action_tensors(&[&game], &self.device)?;
                let available_actions_tensor = available_actions_tensor.detach();

                let policies = (policies * available_actions_tensor)?;
                let policies_sum = policies.sum_keepdim(1)?;
                let policies = policies.broadcast_div(&policies_sum)?;

                let policy = policies.i((0, ..))?;
                let actions = corresponding_action_ids.pop_back().unwrap();

                history.push(PartialHistory {
                    state: game.clone(),
                    policy: policy.clone().detach().to_device(&self.device)?,
                });

                let temperature_action_probabilities = if history.len() >= self.args.temperature_end {
                    policy
                } else {
                    policy.broadcast_pow(&temperature_tensor)?
                };
                let dist = WeightedIndex::new(temperature_action_probabilities.to_vec1::<f32>()?)?;

                let action = actions[dist.sample(&mut thread_rng())];
                game.do_action(action, false)?;

                if game.is_terminated() {
                    let history = std::mem::take(&mut history);
                    for partial_history in history {
                        channel.send(History {
                            state: partial_history.state,
                            policy: partial_history.policy,
                            outcome: game.get_termination_result().termination,
                        })?;
                    }
                }
            }
        }
    }

    fn train(
        &self,
        training_set: &mut [&History],
        optimizer: &mut impl Optimizer,
        var_map: &VarMap,
        log_file: &mut fs::File,
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

            let start_time = std::time::Instant::now();
            let (out_policies, out_values) = network.forward_t(games.iter().collect::<Vec<_>>().as_slice(), true)?;
            writeln!(log_file, "Forward pass took {:?}", start_time.elapsed())?;

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

        writeln!(log_file,
            "Policy loss: {last_policy_loss}, Value loss: {last_value_loss}, L² loss: {last_regularization_loss}, Total loss: {last_loss}"
        )?;

        Ok(())
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

fn get_var_map<P: AsRef<Path>>(training_directory: P) -> PlayerResult<(VarMap, usize)> {
    let network_regex = Regex::new(r"network_(?P<epoch>\d{4}).safetensors").unwrap();

    let mut starting_index = 0;
    let mut network_weights = None;

    for file in fs::read_dir(training_directory)? {
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

fn get_optimizer(var_map: &VarMap, learning_rate: f64) -> PlayerResult<AdamW> {
    Ok(AdamW::new(
        var_map.all_vars(),
        ParamsAdamW {
            lr: learning_rate,
            ..Default::default()
        },
    )?)

    // // force consistent ordering for saving and loading
    // let guard = var_map.data().lock().unwrap();
    // let mut all_vars = guard.iter().collect::<Vec<_>>();
    // all_vars.sort_unstable_by(|entry1, entry2| entry1.0.cmp(entry2.0));
    // let all_vars = all_vars.iter().map(|entry| entry.1.clone()).collect::<Vec<_>>();
    // drop(guard);

    // let mut optimizer = AdamW::new(
    //     all_vars,
    //     ParamsAdamW {
    //         lr: self.args.learning_rate,
    //         ..Default::default()
    //     },
    // )?;

    // if starting_epoch != 0 {
    //     return Ok(optimizer);
    // }

    // let file_name = format!("optimizer_{starting_epoch:04}.safetensors");
    // let file = self.training_directory.join(file_name);

    // optimizer.load(file)?;

    // Ok(optimizer)
}
