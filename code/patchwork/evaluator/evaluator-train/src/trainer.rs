use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use candle_core::{DType, Device, Tensor};
use candle_nn::{Optimizer, VarBuilder, VarMap, SGD};
use evaluator::{NeuralNetworkEvaluator, StaticEvaluator};
use greedy_player::GreedyPlayer;
use patchwork_core::{evaluator_constants, ActionId, Evaluator, Patchwork, PlayerResult, Termination, TerminationType};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand_distr::{Distribution, WeightedIndex};
use regex::Regex;
use tqdm::{refresh, tqdm};

use crate::training_args::TrainingArgs;

pub struct Trainer {
    pub args: TrainingArgs,
    pub training_directory: PathBuf,
    pub last_eval_percentage: f64,
}

pub struct History {
    pub state: Patchwork,
    pub termination: Termination,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RandomizedGreedyPlayer<Eval: Evaluator = StaticEvaluator> {
    pub temperature: f64,
    pub evaluator: Eval,
}

impl<Eval: Evaluator> RandomizedGreedyPlayer<Eval> {
    pub const fn new_with_evaluator(temperature: f64, evaluator: Eval) -> Self {
        Self { temperature, evaluator }
    }

    fn get_action(&self, game: &Patchwork) -> PlayerResult<ActionId> {
        let mut game = game.clone();
        let valid_actions = game.get_valid_actions().into_iter().collect::<Vec<_>>();
        let color = if game.is_player_1() { 1 } else { -1 };

        let action_probabilities = valid_actions
            .iter()
            .map(|action| {
                game.do_action(*action, false)?;
                let evaluation = self.evaluator.evaluate_node(&game);
                game.undo_action(*action, false)?;

                Ok(
                    f64::from(color * evaluation + evaluator_constants::POSITIVE_INFINITY + 1)
                        / f64::from(2 * evaluator_constants::POSITIVE_INFINITY),
                )
            })
            .collect::<PlayerResult<Vec<_>>>()?;

        let sum = action_probabilities.iter().sum::<f64>();
        let action_probabilities = action_probabilities
            .iter()
            .map(|x| x / sum)
            .map(|x| x.powf(1.0 / self.temperature))
            .collect::<Vec<_>>();

        let dist = WeightedIndex::new(action_probabilities)?;
        let action = valid_actions[dist.sample(&mut thread_rng())];

        Ok(action)
    }
}

impl Trainer {
    pub fn new<P: AsRef<Path>>(training_directory: P, args: TrainingArgs) -> Self {
        Self {
            training_directory: training_directory.as_ref().to_path_buf(),
            args,
            last_eval_percentage: f64::NAN,
        }
    }

    pub fn learn(&mut self) -> PlayerResult<()> {
        let mut iteration = 0;
        let mut network_improvements = 0;

        let (var_map, starting_index) = get_var_map(&self.training_directory)?;
        println!("Starting at index {starting_index:?}");

        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &Device::Cpu);
        let network = NeuralNetworkEvaluator::new(var_builder)?;
        self.evaluate_network(network, 5); // evaluate the current network with more accuracy
        drop(var_map);
        let mut history = vec![];

        loop {
            // play games
            println!(
                "[{network_improvements:?}/{iteration:?}]: Self playing {:?} games",
                self.args.number_of_games
            );
            let start_time = std::time::Instant::now();

            // retain only 2x amount self play games
            // history.shuffle(&mut rand::thread_rng());
            // history.truncate(self.args.number_of_games.get() * 2 * 43);
            history.clear();
            history.extend(self.play()?);
            history.shuffle(&mut rand::thread_rng());

            // train network
            println!("[{network_improvements:?}/{iteration:?}]: Training network");
            let (var_map, starting_index, new_network) = self.train(&history)?;

            // test against old network
            println!("[{network_improvements:?}/{iteration:?}]: Evaluating network");

            iteration += 1;

            if self.evaluate_network(new_network.clone(), 1) {
                // save network and use as new best
                let index = starting_index + 1;
                let network_weights = self.training_directory.join(format!("network_{index:04}.safetensors"));

                network_improvements += 1;
                println!(
                    "[{network_improvements:?}/{iteration:?}]: New network won. Saving network to {network_weights:?} (last eval: {:?})",
                    self.last_eval_percentage
                );
                var_map.save(&network_weights)?;
                history.clear();

                // self.compare_against_other(new_network);
            } else {
                // discard network
                println!(
                    "[{network_improvements:?}/{iteration:?}]: New network lost. Discarding network (last eval: {:?})",
                    self.last_eval_percentage
                );
            }

            println!(
                "[{network_improvements:?}/{iteration:?}]: Done in {:?}",
                start_time.elapsed()
            );
        }
    }

    pub fn play(&self) -> PlayerResult<Vec<History>> {
        let (var_map, _) = get_var_map(&self.training_directory)?;
        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &Device::Cpu);
        let network = NeuralNetworkEvaluator::new(var_builder)?;

        let history = boxcar::vec![];
        let game_counter = AtomicUsize::new(0);
        let number_of_games = self.args.number_of_games.get();
        let neural_net_player = RandomizedGreedyPlayer::new_with_evaluator(self.args.temperature, network);
        let greedy_player = GreedyPlayer::new_with_evaluator("Greedy", StaticEvaluator);

        std::thread::scope(|s| {
            let mut threads = Vec::with_capacity(self.args.parallelization.get() - 1);
            for _ in 0..(self.args.parallelization.get() - 1) {
                threads.push(s.spawn(|| {
                    'outer: while game_counter.load(Ordering::Relaxed) < number_of_games {
                        let mut states = vec![];

                        let mut state = Patchwork::get_initial_state(None);
                        loop {
                            states.push(state.clone());

                            let action = if state.is_player_1() {
                                neural_net_player.get_action(&state).expect("Failed to get action")
                            } else {
                                greedy_player.get_action(&state).expect("Failed to get action")
                            };

                            state.do_action(action, false).expect("Failed to do action");

                            if state.is_terminated() {
                                if game_counter.load(Ordering::Relaxed) >= number_of_games {
                                    break 'outer;
                                }

                                let termination = state.get_termination_result();

                                for state in states {
                                    history.push(History { state, termination });
                                }
                                game_counter.fetch_add(1, Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                }));
            }

            'outer: while game_counter.load(Ordering::Relaxed) < number_of_games {
                print!("\r{:?}/{:?}", game_counter.load(Ordering::Relaxed), number_of_games);
                std::io::stdout().flush().unwrap();

                let mut states = vec![];

                let mut state = Patchwork::get_initial_state(None);
                loop {
                    states.push(state.clone());

                    let action = if state.is_player_1() {
                        neural_net_player.get_action(&state).expect("Failed to get action")
                    } else {
                        greedy_player.get_action(&state).expect("Failed to get action")
                    };

                    state.do_action(action, false).expect("Failed to do action");

                    if state.is_terminated() {
                        if game_counter.load(Ordering::Relaxed) >= number_of_games {
                            break 'outer;
                        }

                        let termination = state.get_termination_result();

                        for state in states {
                            history.push(History { state, termination });
                        }
                        game_counter.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });

        println!("\r{:?}/{:?}", game_counter.load(Ordering::Relaxed), number_of_games);

        Ok(history.into_iter().collect())
    }

    pub fn train(&self, history: &[History]) -> PlayerResult<(VarMap, usize, NeuralNetworkEvaluator)> {
        let (var_map, starting_index) = get_var_map(&self.training_directory)?;
        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &Device::Cpu);
        let mut optimizer = get_optimizer(&var_map, self.args.learning_rate)?;

        let network = NeuralNetworkEvaluator::new(var_builder)?;
        let mut loss_sum = 0.0;
        let mut iterations = 0;

        let history = history
            .iter()
            .filter(|state| state.state.is_player_1())
            .collect::<Vec<_>>();

        for _ in tqdm(0..self.args.epochs)
            .style(tqdm::Style::Block)
            .desc(Some("Epoch"))
            .clear(true)
        {
            for batch in tqdm(history.chunks(self.args.batch_size))
                .style(tqdm::Style::Block)
                .desc(Some("Batch"))
                .clear(true)
            {
                let mut values = vec![];
                let mut targets = vec![];

                for game in batch {
                    let network_eval = network.forward(&game.state)?;
                    values.push(network_eval.clone());
                    values.push(network_eval.clone());
                    // values.push(network_eval);

                    // Move towards expected result
                    let mut probability: f32 = 0.0;
                    for _ in 0..1000 {
                        match game.state.random_rollout().get_termination_result().termination {
                            TerminationType::Player1Won => probability += 1.0,
                            TerminationType::Player2Won => {}
                        }
                    }
                    probability /= 1000.0;
                    targets.push(Tensor::new(probability, &Device::Cpu)?);

                    // // Move to static eval
                    // let target = (StaticEvaluator.evaluate_node(&game.state) as f32
                    //     / evaluator_constants::POSITIVE_INFINITY as f32)
                    //     .clamp(-1.0, 1.0);
                    // let target = Tensor::new(target, &Device::Cpu)?;
                    // targets.push(target);

                    // Move to win loss
                    let multiplier = if game.state.is_player_1() { 1.0 } else { -1.0 };

                    let target_win: f32 = match game.termination.termination {
                        TerminationType::Player1Won => 0.9,
                        TerminationType::Player2Won => -0.9,
                    };
                    let target_score = 0.1 * (game.termination.score() as f32 / 75.0);
                    let target = (target_win + target_score).clamp(-1.0, 1.0);

                    let target = Tensor::new(multiplier * target, &Device::Cpu)?;

                    targets.push(target);
                }

                let values = Tensor::stack(&values, 0)?;
                let targets = Tensor::stack(&targets, 0)?;

                let loss = candle_nn::loss::mse(&values, &targets)?;
                optimizer.backward_step(&loss)?;

                loss_sum += f64::from(loss.to_scalar::<f32>()?);
                iterations += 1;
            }
        }

        refresh()?;
        println!("\nAverage loss: {:?}", loss_sum / f64::from(iterations));

        Ok((var_map, starting_index, network))
    }

    pub fn evaluate_network(&mut self, new_network: NeuralNetworkEvaluator, multiplier: usize) -> bool {
        // load current best network
        // let (var_map, _) = get_var_map(&self.training_directory)?;
        // let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &Device::Cpu);
        // let old_network = NeuralNetworkEvaluator::new(var_builder)?;

        let player_1 = GreedyPlayer::new_with_evaluator("1", new_network);
        // let player_2 = GreedyPlayer::new_with_evaluator("2", old_network);
        let player_2 = GreedyPlayer::new_with_evaluator("2", StaticEvaluator);

        let percentage = self.compare_players(&player_1, &player_2, self.args.evaluation_games * multiplier);

        if self.last_eval_percentage.is_nan() {
            self.last_eval_percentage = percentage;
            return false;
        }

        if percentage >= self.last_eval_percentage {
            self.last_eval_percentage = percentage;
            true
        } else {
            false
        }
    }

    // pub fn compare_against_other(&self, network: NeuralNetworkEvaluator) {
    //     let player_1 = GreedyPlayer::new_with_evaluator("1", network);
    //     let player_2 = GreedyPlayer::<StaticEvaluator>::new("2");

    //     self.compare_players(&player_1, &player_2, self.args.comparison_games);
    // }

    fn compare_players<Eval1: Evaluator, Eval2: Evaluator>(
        &self,
        player_1: &GreedyPlayer<Eval1>,
        player_2: &GreedyPlayer<Eval2>,
        amount_of_games: usize,
    ) -> f64 {
        fn print_comparison(games_played: usize, amount_of_games: usize, wins_player_1: usize, newline: bool) {
            const PROGRESS_BAR_LENGTH: usize = 100;

            let progress_player_1 =
                (wins_player_1 as f64 / games_played as f64 * PROGRESS_BAR_LENGTH as f64).round() as usize;
            let progress_player_2 = (PROGRESS_BAR_LENGTH as i32 - progress_player_1 as i32).max(0) as usize;

            print!(
                "\r{: >3?} \x1b[0;32m{}\x1b[0;31m{}\x1b[0m {: >3?} ({} / {}) ({:.5?}%)",
                wins_player_1,
                "█".repeat(progress_player_1),
                "█".repeat(progress_player_2),
                games_played - wins_player_1,
                games_played,
                amount_of_games,
                (wins_player_1 as f64 / games_played as f64 * 100.0)
            );
            if newline {
                println!();
            }
            std::io::stdout().flush().unwrap();
        }

        let wins = AtomicUsize::new(0);
        let games_played = AtomicUsize::new(0);

        std::thread::scope(|s| {
            let mut threads = Vec::with_capacity(self.args.parallelization.get() - 1);
            for _ in 0..(self.args.parallelization.get() - 1) {
                threads.push(s.spawn(|| {
                    'outer: while games_played.load(Ordering::Relaxed) < amount_of_games {
                        let mut state = Patchwork::get_initial_state(None);

                        loop {
                            let action = if state.is_player_1() {
                                player_1.get_action(&state).expect("Failed to get action")
                            } else {
                                player_2.get_action(&state).expect("Failed to get action")
                            };

                            state.do_action(action, false).expect("Failed to do action");

                            if state.is_terminated() {
                                let termination = state.get_termination_result();

                                if matches!(termination.termination, TerminationType::Player1Won) {
                                    wins.fetch_add(1, Ordering::Relaxed);
                                }

                                if games_played.fetch_add(1, Ordering::Relaxed) >= amount_of_games {
                                    break 'outer;
                                }
                                break;
                            }
                        }
                    }
                }));
            }

            'outer: while games_played.load(Ordering::Relaxed) < amount_of_games {
                print_comparison(
                    games_played.load(Ordering::Relaxed),
                    amount_of_games,
                    wins.load(Ordering::Relaxed),
                    false,
                );

                let mut state = Patchwork::get_initial_state(None);

                loop {
                    let action = if state.is_player_1() {
                        player_1.get_action(&state).expect("Failed to get action")
                    } else {
                        player_2.get_action(&state).expect("Failed to get action")
                    };

                    state.do_action(action, false).expect("Failed to do action");

                    if state.is_terminated() {
                        let termination = state.get_termination_result();

                        if matches!(termination.termination, TerminationType::Player1Won) {
                            wins.fetch_add(1, Ordering::Relaxed);
                        }

                        if games_played.fetch_add(1, Ordering::Relaxed) >= amount_of_games {
                            break 'outer;
                        }
                        break;
                    }
                }
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
        let percentage = wins.load(Ordering::SeqCst) as f64 / games_played.load(Ordering::SeqCst) as f64;

        print_comparison(
            games_played.load(Ordering::Relaxed),
            amount_of_games,
            wins.load(Ordering::Relaxed),
            true,
        );

        percentage
    }
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

fn get_optimizer(var_map: &VarMap, learning_rate: f64) -> PlayerResult<SGD> {
    Ok(SGD::new(var_map.all_vars(), learning_rate)?)
}
