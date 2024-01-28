use std::{
    io::Write,
    sync::atomic::{self, AtomicI32, AtomicU32, AtomicU64, Ordering},
};

use crate::{exit::handle_exit, player as player_mod};
use patchwork_lib::{player::Player, ActionId, Patchwork, TerminationType};
use rustyline::{history::FileHistory, Editor};

// TODO: allow args via command line
pub fn handle_compare(rl: &mut Editor<(), FileHistory>) {
    let player_1 = loop {
        match rl.readline("Player 1: ") {
            Ok(player) => {
                if get_player_from_str(&player.to_ascii_lowercase(), 1).is_some() {
                    break player;
                } else {
                    println!("Could not find player {}. Available players: ", player);
                    for player in player_mod::get_available_players() {
                        println!("  {}", player);
                    }
                    println!("  extern <path-to-application>");
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };
    let player_2 = loop {
        match rl.readline("Player 2: ") {
            Ok(player) => {
                if get_player_from_str(&player.to_ascii_lowercase(), 1).is_some() {
                    break player;
                } else {
                    println!("Could not find player {}. Available players: ", player);
                    for player in player_mod::get_available_players() {
                        println!("  {}", player);
                    }
                    println!("  extern <path-to-application>");
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };
    let iterations = loop {
        match rl.readline_with_initial("Iterations: ", ("100", "")) {
            Ok(iterations) => {
                if let Ok(iterations) = iterations.parse::<usize>() {
                    break iterations;
                } else {
                    println!("Please enter a valid positive number.");
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };
    let update = loop {
        match rl.readline_with_initial("Update (in ms): ", ("100", "")) {
            Ok(update) => {
                if let Ok(update) = update.parse::<usize>() {
                    break Some(update);
                } else {
                    println!("Please enter a valid positive number.");
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };
    let available_parallelism: usize = std::thread::available_parallelism().map_or(1, |p| p.into() - 1);
    let parallelization = loop {
        match rl.readline_with_initial("Parallelization: ", (format!("{}", available_parallelism).as_str(), "")) {
            Ok(parallelization) => {
                if let Ok(parallelization) = parallelization.parse::<usize>() {
                    if let Ok(max_threads) = std::thread::available_parallelism() {
                        if parallelization > max_threads.get() {
                            println!(
                                "Please enter a valid positive number between 1 and {}.",
                                available_parallelism
                            );
                            std::io::stdout().flush().unwrap();
                            continue;
                        }
                    }

                    break Some(parallelization);
                }

                println!(
                    "Please enter a valid positive number between 1 and {}",
                    std::thread::available_parallelism().map_or(1, |p| p.into())
                );
                std::io::stdout().flush().unwrap();
            }
            Err(_) => handle_exit(),
        }
    };

    compare(iterations, &player_1, &player_2, update, parallelization);
}

enum PlayerType {
    BuildIn(Box<dyn Player>),
    #[allow(dead_code)]
    Upi(String), // TODO: implement extern UPI
}

impl PlayerType {
    pub fn name(&self) -> &str {
        match self {
            PlayerType::BuildIn(player) => player.name(),
            PlayerType::Upi(_) => unimplemented!("[PlayerType::name] UPI is not yet implemented."),
        }
    }

    pub fn get_action(&mut self, state: &Patchwork) -> anyhow::Result<ActionId> {
        match self {
            PlayerType::BuildIn(player) => player.get_action(state),
            PlayerType::Upi(_) => unimplemented!("[PlayerType::get_action] UPI is not yet implemented."),
        }
    }
}

fn get_player_from_str(name: &str, player_position: usize) -> Option<PlayerType> {
    if name.starts_with("extern") {
        unimplemented!("[get_player_from_str] Extern upi players are not yet implemented.");
    }

    player_mod::get_player(name, player_position).map(PlayerType::BuildIn)
}

fn compare(iterations: usize, player_1: &str, player_2: &str, update: Option<usize>, parallelization: Option<usize>) {
    let update = update
        .map(|u| std::time::Duration::from_millis(u as u64))
        .unwrap_or(std::time::Duration::from_millis(100));
    let parallelization = parallelization
        .or(std::thread::available_parallelism().map(|p| p.into()).ok())
        .unwrap_or(1);

    let temp_player_1 = get_player_from_str(player_1, 1).unwrap();
    let temp_player_2 = get_player_from_str(player_2, 2).unwrap();
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
                let mut player_1 = get_player_from_str(&player_1, 1).unwrap();
                let mut player_2 = get_player_from_str(&player_2, 2).unwrap();

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
                                u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos()).unwrap();

                            sum_time_player_1.fetch_add(end, Ordering::Relaxed);
                            n_time_player_1.fetch_add(1, Ordering::Relaxed);
                            action
                        } else {
                            let start_time = std::time::Instant::now();
                            let action = player_2.get_action(&state).unwrap();
                            let end =
                                u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos()).unwrap();
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
            print_progress(
                iterations_done,
                iterations,
                wins_player_1.load(Ordering::Relaxed) as usize,
                wins_player_2.load(Ordering::Relaxed) as usize,
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

    print_progress(
        iterations_done.load(Ordering::Relaxed) as usize,
        iterations,
        wins_player_1.load(Ordering::Relaxed) as usize,
        wins_player_2.load(Ordering::Relaxed) as usize,
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

    print!("\x1b[4A\r");
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
    let progress_bar_length = 100;

    if iteration == 0 {
        println!(
            "{} {} {}  ",
            player_1_name,
            "█".repeat(progress_bar_length),
            player_2_name,
        );
    } else {
        let progress_player_1 = (wins_player_1 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_player_2 = (progress_bar_length as i32 - progress_player_1 as i32).max(0) as usize;
        println!(
            "{} \x1b[0;32m{}\x1b[0;31m{}\x1b[0m {}  ",
            player_1_name,
            "█".repeat(progress_player_1),
            "█".repeat(progress_player_2),
            player_2_name
        );
    }
    std::io::stdout().flush().unwrap();
}
