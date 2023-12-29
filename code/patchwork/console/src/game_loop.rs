use std::sync::atomic::{self, AtomicI32, AtomicU32, AtomicU64, Ordering};

use patchwork::{
    player::{
        AlphaZeroPlayer, GreedyPlayer, HumanPlayer, MCTSPlayer, MinimaxOptions, MinimaxPlayer, PVSOptions, PVSPlayer,
        Player, RandomOptions, RandomPlayer,
    },
    Patchwork, TerminationType,
};
use regex::Regex;

pub struct GameLoop;

impl GameLoop {
    #[allow(clippy::field_reassign_with_default)]
    pub fn get_player(player: &str, player_position: usize) -> Box<dyn Player> {
        match player {
            // HUMAN
            "human" => Box::new(HumanPlayer::new(format!("Human Player {player_position}"))),
            // SIMPLE ENGINES
            "random" => Box::new(RandomPlayer::new(format!("Random Player {player_position}"), None)),
            _ if player.starts_with("random") => {
                let seed = Regex::new(r"random\((?<seed>\d+)\)")
                    .unwrap()
                    .captures(player)
                    .unwrap()
                    .name("seed")
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();
                Box::new(RandomPlayer::new(
                    format!("Random Player {player_position} ({})", seed),
                    Some(RandomOptions::new(seed)),
                ))
            }
            "greedy" => Box::new(GreedyPlayer::new(format!("Greedy Player {player_position}"))),
            // TREE SEARCH ENGINES
            "minimax" => Box::new(MinimaxPlayer::new(
                format!("Minimax Player {player_position}"),
                Default::default(),
            )),
            _ if player.starts_with("minimax") => {
                let regex = Regex::new(r"minimax\((?<depth>\d+),\s*(?<pieces>\d+)\)").unwrap();
                let captures = regex.captures(player).unwrap();
                let depth = captures.name("depth").unwrap().as_str().parse().unwrap();
                let amount_actions_per_piece = captures.name("pieces").unwrap().as_str().parse().unwrap();
                Box::new(MinimaxPlayer::new(
                    format!(
                        "Minimax Player {player_position} ({}, {})",
                        depth, amount_actions_per_piece
                    ),
                    Some(MinimaxOptions::new(depth, amount_actions_per_piece)),
                ))
            }
            "pvs" => Box::new(PVSPlayer::new(format!("PVS Player {player_position}"), None)),
            _ if player.starts_with("pvs") => {
                let regex = Regex::new(r"pvs\((?<time>\d+)\)").unwrap();
                let captures = regex.captures(player).unwrap();
                let time = captures.name("time").unwrap().as_str().parse().unwrap();
                let mut options = PVSOptions::default();
                options.time_limit = std::time::Duration::from_secs(time);

                Box::new(PVSPlayer::new(
                    format!("PVS Player {player_position} ({})", time),
                    Some(options),
                ))
            }
            // MCTS ENGINES
            "mcts" => Box::new(MCTSPlayer::new(
                format!("MCTS Player {player_position}"),
                Default::default(),
            )),
            "alphazero" => Box::new(AlphaZeroPlayer::new(format!("AlphaZero Player {player_position}"))),
            // ERROR
            _ => panic!("Unknown player: {player}"),
        }
    }

    pub fn run(player_1: &str, player_2: &str) {
        let mut player_1 = GameLoop::get_player(player_1, 1);
        let mut player_2 = GameLoop::get_player(player_2, 2);

        println!("Player 1: {}", player_1.name());
        println!("Player 2: {}", player_2.name());

        let mut state = Patchwork::get_initial_state(None);

        let mut i = 1;
        loop {
            println!("─────────────────────────────────────────────────── TURN {} ──────────────────────────────────────────────────", i);
            println!("{}", state);

            #[cfg(debug_assertions)]
            let old_state = state.clone();

            let action = if state.is_player_1() {
                player_1.get_action(&state).unwrap()
            } else {
                player_2.get_action(&state).unwrap()
            };

            #[cfg(debug_assertions)]
            if old_state != state {
                println!("─────────────────────────────────────────────────── ERROR ───────────────────────────────────────────────────");
                println!("Old state:");
                println!("{}", old_state);
                println!("New state:");
                println!("{}", state);
                panic!("State changed!");
            }

            println!(
                "Player '{}' chose action: {}",
                if state.is_player_1() {
                    player_1.name()
                } else {
                    player_2.name()
                },
                action
            );

            let mut next_state = state.clone();
            next_state.do_action(action, false).unwrap();
            state = next_state;

            if state.is_terminated() {
                let termination = state.get_termination_result();

                println!("────────────────────────────────────────────────── RESULT ────────────────────────────────────────────────────");
                println!("{}", state);

                match termination.termination {
                    TerminationType::Player1Won => println!("Player 1 won!"),
                    TerminationType::Player2Won => println!("Player 2 won!"),
                    TerminationType::Draw => println!("Draw!"),
                }

                println!("{}", termination.player_1_score);
                println!("{}", termination.player_2_score);
                break;
            }

            i += 1;
        }
    }

    pub fn compare(
        iterations: usize,
        player_1: &str,
        player_2: &str,
        update: Option<usize>,
        parallelization: Option<usize>,
    ) {
        let update = update
            .map(|u| std::time::Duration::from_millis(u as u64))
            .unwrap_or(std::time::Duration::from_millis(100));
        let parallelization = parallelization
            .or(std::thread::available_parallelism().map(|p| p.into()).ok())
            .unwrap_or(1);

        let temp_player_1 = GameLoop::get_player(player_1, 1);
        let temp_player_2 = GameLoop::get_player(player_2, 2);
        let player_1_name = temp_player_1.name();
        let player_2_name = temp_player_2.name();

        println!(
            "Comparing {} iterations with {} threads: {} vs. {}",
            iterations, parallelization, player_1_name, player_2_name
        );

        let max_player_1_score = AtomicI32::new(i32::MIN);
        let max_player_2_score = AtomicI32::new(i32::MIN);
        let min_player_1_score = AtomicI32::new(i32::MAX);
        let min_player_2_score = AtomicI32::new(i32::MAX);
        let sum_player_1_score = AtomicI32::new(0);
        let sum_player_2_score = AtomicI32::new(0);
        let sum_time_player_1 = AtomicU64::new(0);
        let sum_time_player_2 = AtomicU64::new(0);
        let n_time_player_1 = AtomicU64::new(0);
        let n_time_player_2 = AtomicU64::new(0);
        let wins_player_1 = AtomicU32::new(0);
        let wins_player_2 = AtomicU32::new(0);
        let draws = AtomicU32::new(0);

        print!("\n\n\n\n\n");

        let iterations_done = AtomicU32::new(0);
        std::thread::scope(|s| {
            for _ in 0..parallelization {
                let player_1 = player_1.to_string();
                let player_2 = player_2.to_string();
                let iterations = iterations as u32;
                let iterations_done = &iterations_done;
                let wins_player_1 = &wins_player_1;
                let wins_player_2 = &wins_player_2;
                let draws = &draws;
                let max_player_1_score = &max_player_1_score;
                let max_player_2_score = &max_player_2_score;
                let min_player_1_score = &min_player_1_score;
                let min_player_2_score = &min_player_2_score;
                let sum_player_1_score = &sum_player_1_score;
                let sum_player_2_score = &sum_player_2_score;
                let sum_time_player_1 = &sum_time_player_1;
                let sum_time_player_2 = &sum_time_player_2;
                let n_time_player_1 = &n_time_player_1;
                let n_time_player_2 = &n_time_player_2;
                s.spawn(move || {
                    let mut player_1 = GameLoop::get_player(&player_1, 1);
                    let mut player_2 = GameLoop::get_player(&player_2, 2);

                    'outer: while iterations_done.load(Ordering::Acquire) < iterations {
                        let mut state = Patchwork::get_initial_state(None);
                        loop {
                            if iterations_done.load(Ordering::Acquire) >= iterations {
                                break 'outer;
                            }

                            let action = if state.is_player_1() {
                                let start_time = std::time::Instant::now();
                                let action = player_1.get_action(&state).unwrap();
                                let end =
                                    u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos())
                                        .unwrap();

                                sum_time_player_1.fetch_add(end, Ordering::Relaxed);
                                n_time_player_1.fetch_add(1, Ordering::Relaxed);
                                action
                            } else {
                                let start_time = std::time::Instant::now();
                                let action = player_2.get_action(&state).unwrap();
                                let end =
                                    u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos())
                                        .unwrap();
                                sum_time_player_2.fetch_add(end, Ordering::Relaxed);
                                n_time_player_2.fetch_add(1, Ordering::Relaxed);
                                action
                            };

                            let mut next_state = state.clone();
                            next_state.do_action(action, false).unwrap();
                            state = next_state;

                            if state.is_terminated() {
                                let termination = state.get_termination_result();

                                match termination.termination {
                                    TerminationType::Player1Won => {
                                        wins_player_1.fetch_add(1, Ordering::Relaxed);
                                    }
                                    TerminationType::Player2Won => {
                                        wins_player_2.fetch_add(1, Ordering::Relaxed);
                                    }
                                    TerminationType::Draw => {
                                        draws.fetch_add(1, Ordering::Relaxed);
                                    }
                                }

                                max_player_1_score.fetch_max(termination.player_1_score, Ordering::Relaxed);
                                max_player_2_score.fetch_max(termination.player_2_score, Ordering::Relaxed);
                                min_player_1_score.fetch_min(termination.player_1_score, Ordering::Relaxed);
                                min_player_2_score.fetch_min(termination.player_2_score, Ordering::Relaxed);
                                sum_player_1_score.fetch_add(termination.player_1_score, Ordering::Relaxed);
                                sum_player_2_score.fetch_add(termination.player_2_score, Ordering::Relaxed);
                                iterations_done.fetch_add(1, Ordering::Release);
                                break;
                            }
                        }
                    }
                });
            }
            loop {
                let iterations_done = iterations_done.load(Ordering::Relaxed) as usize;
                if iterations_done >= iterations {
                    break;
                }
                GameLoop::print_progress(
                    iterations_done,
                    iterations,
                    wins_player_1.load(Ordering::Relaxed) as usize,
                    wins_player_2.load(Ordering::Relaxed) as usize,
                    draws.load(Ordering::Relaxed) as usize,
                    max_player_1_score.load(Ordering::Relaxed),
                    max_player_2_score.load(Ordering::Relaxed),
                    min_player_1_score.load(Ordering::Relaxed),
                    min_player_2_score.load(Ordering::Relaxed),
                    sum_player_1_score.load(Ordering::Relaxed) as f64,
                    sum_player_2_score.load(Ordering::Relaxed) as f64,
                    sum_time_player_1.load(Ordering::Relaxed) as f64,
                    n_time_player_1.load(Ordering::Relaxed) as f64,
                    sum_time_player_2.load(Ordering::Relaxed) as f64,
                    n_time_player_2.load(Ordering::Relaxed) as f64,
                    player_1_name,
                    player_2_name,
                );
                std::thread::sleep(update);
            }
        });

        atomic::fence(Ordering::SeqCst);

        GameLoop::print_progress(
            iterations_done.load(Ordering::Relaxed) as usize,
            iterations,
            wins_player_1.load(Ordering::Relaxed) as usize,
            wins_player_2.load(Ordering::Relaxed) as usize,
            draws.load(Ordering::Relaxed) as usize,
            max_player_1_score.load(Ordering::Relaxed),
            max_player_2_score.load(Ordering::Relaxed),
            min_player_1_score.load(Ordering::Relaxed),
            min_player_2_score.load(Ordering::Relaxed),
            sum_player_1_score.load(Ordering::Relaxed) as f64,
            sum_player_2_score.load(Ordering::Relaxed) as f64,
            sum_time_player_1.load(Ordering::Relaxed) as f64,
            n_time_player_1.load(Ordering::Relaxed) as f64,
            sum_time_player_2.load(Ordering::Relaxed) as f64,
            n_time_player_2.load(Ordering::Relaxed) as f64,
            player_1_name,
            player_2_name,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn print_progress(
        iteration: usize,
        iterations: usize,
        wins_player_1: usize,
        wins_player_2: usize,
        draws: usize,
        max_player_1_score: i32,
        max_player_2_score: i32,
        min_player_1_score: i32,
        min_player_2_score: i32,
        avg_player_1_score: f64,
        avg_player_2_score: f64,
        sum_time_player_1: f64,
        n_time_player_1: f64,
        sum_time_player_2: f64,
        n_time_player_2: f64,
        player_1_name: &str,
        player_2_name: &str,
    ) {
        let avg_player_1_score = avg_player_1_score / iteration as f64;
        let avg_player_2_score = avg_player_2_score / iteration as f64;

        let avg_player_1_time = sum_time_player_1 / n_time_player_1;
        let avg_player_2_time = sum_time_player_2 / n_time_player_2;

        print!("\x1b[5A\r");
        println!("Iteration {: >7} / {}", iteration, iterations);
        println!(
            "Player 1: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}, avg time: {:?}]                       ",
            wins_player_1,
            (wins_player_1 as f64 / iteration as f64 * 100.0),
            avg_player_1_score,
            max_player_1_score,
            min_player_1_score,
            std::time::Duration::from_nanos(avg_player_1_time.round() as u64)
        );
        println!(
            "Player 2: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}, avg time: {:?}]                       ",
            wins_player_2,
            (wins_player_2 as f64 / iteration as f64 * 100.0),
            avg_player_2_score,
            max_player_2_score,
            min_player_2_score,
            std::time::Duration::from_nanos(avg_player_2_time.round() as u64)
        );
        println!(
            "          {: >7} draws ({:0>5.2}%)                       ",
            draws,
            (draws as f64 / iteration as f64 * 100.0)
        );
        let progress_bar_length: i32 = 120;
        let progress_player_1 = (wins_player_1 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_player_2 = (wins_player_2 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_draws =
            (progress_bar_length - progress_player_1 as i32 - progress_player_2 as i32).max(0) as usize;

        println!(
            "{} \x1b[0;32m{}\x1b[0m{}\x1b[0;31m{}\x1b[0m {}  ",
            player_1_name,
            "█".repeat(progress_player_1),
            "█".repeat(progress_draws),
            "█".repeat(progress_player_2),
            player_2_name,
        );
    }
}

#[cfg(test)]
mod tests {
    use patchwork::GameOptions;

    use super::*;

    #[test]
    fn random_player() {
        let player = Box::<RandomPlayer>::default();
        test_player(player);
    }

    #[test]
    fn greedy_player() {
        let player = Box::<GreedyPlayer>::default();
        test_player(player);
    }

    // TODO: uncomment when pvs is fixed
    // #[test]
    // fn pvs_player() {
    //     let player = Box::new(PVSPlayer::new(
    //         "PVS Player",
    //         Some(PVSOptions::new(
    //             std::time::Duration::from_secs(2),
    //             Box::<StaticEvaluator>::default(),
    //             Box::<NoopActionSorter>::default(),
    //             PVSFeatures {
    //                 aspiration_window: true,
    //                 transposition_table: TranspositionTableFeature::Enabled { size: Size::MiB(10) },
    //                 late_move_reductions: true,
    //                 search_extensions: true,
    //                 diagnostics: DiagnosticsFeature::Enabled {
    //                     writer: Box::new(std::io::stdout()),
    //                 },
    //             },
    //         )),
    //     ));
    //     test_player(player);
    // }

    // TODO: test other players

    fn test_player(mut player: Box<dyn Player>) {
        let mut state = Patchwork::get_initial_state(Some(GameOptions { seed: 42 }));
        loop {
            let action_result = player.get_action(&state);

            let action = match action_result {
                Ok(action) => action,
                Err(error) => {
                    println!("Player '{}' get_action failed with: {}", player.name(), error);
                    println!("State: {}", state);
                    panic!("{}", error);
                }
            };

            let valid_actions = state.get_valid_actions();
            if !valid_actions.contains(&action) {
                println!("Player '{}' chose invalid action: {}", player.name(), action);
                println!("State: {}", state);
                panic!("Invalid action!");
            }

            match state.do_action(action, false) {
                Ok(_) => {}
                Err(error) => {
                    println!("Player '{}' do_action failed with: {}", player.name(), error);
                    println!("State:");
                    println!("{}", state);
                    panic!("{}", error);
                }
            }

            if state.is_terminated() {
                break;
            }
        }
    }
}
