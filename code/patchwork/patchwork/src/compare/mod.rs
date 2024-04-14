use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    panic,
    path::Path,
    sync::atomic::{self, AtomicI32, AtomicU32, AtomicU64, Ordering},
};

use anyhow::Error;
use clap::Parser;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};

use crate::common::{get_logging, get_player, interactive_get_player, PlayerType, CTRL_C_MESSAGE, CTRL_D_MESSAGE};
use patchwork_lib::{
    player::{Logging, Player},
    Patchwork, TerminationType,
};

#[derive(Debug, Parser, Default)]
#[command(no_binary_name(true))]
struct CmdArgs {
    #[arg(long = "player-1", alias = "p1", short = '1')]
    player_1: Option<String>,
    #[arg(long = "player-2", alias = "p2", short = '2')]
    player_2: Option<String>,
    #[arg(long = "logging-1", alias = "l1", default_value = "disabled")]
    logging_player_1: String,
    #[arg(long = "logging-2", alias = "l2", default_value = "disabled")]
    logging_player_2: String,
    #[arg(long = "games", short = 'g')]
    games: Option<usize>,
    #[arg(long = "update", short = 'u', default_value = "100")]
    update: u64,
    #[arg(long = "parallel", short = 'p')]
    parallel: Option<usize>,
}

struct RecordedGame {
    pub player_1_name: String,
    pub player_2_name: String,
    pub result: TerminationType,
}

pub fn handle_compare(rl: &mut Editor<(), FileHistory>, args: Vec<String>) -> anyhow::Result<()> {
    let args = CmdArgs::parse_from(args);

    let player_1_logging = get_logging(args.logging_player_1.as_str())?;
    let player_2_logging = get_logging(args.logging_player_2.as_str())?;

    let player_1 = interactive_get_player(rl, args.player_1, 1, player_1_logging)?;
    let player_2 = interactive_get_player(rl, args.player_2, 2, player_2_logging)?;

    let games = if let Some(games) = args.games {
        games
    } else {
        loop {
            match rl.readline_with_initial("Games: ", ("100", "")) {
                Ok(games) => {
                    if let Ok(games) = games.parse::<usize>() {
                        break games;
                    }
                    println!("Please enter a valid positive number.");
                    std::io::stdout().flush().unwrap();
                }
                Err(ReadlineError::Interrupted) => return Err(Error::msg(CTRL_C_MESSAGE)),
                Err(ReadlineError::Eof) => return Err(Error::msg(CTRL_D_MESSAGE)),
                Err(err) => return Err(Error::from(err)),
            }
        }
    };
    let available_parallelism: usize = std::thread::available_parallelism().map_or(1, |p| p.get() - 1);
    let parallelization = if let Some(parallelization) = args.parallel {
        parallelization
    } else {
        loop {
            match rl.readline_with_initial(
                &format!("Parallelization (max {available_parallelism}): "),
                (available_parallelism.to_string().as_str(), ""),
            ) {
                Ok(parallelization) => {
                    if let Ok(parallelization) = parallelization.parse::<usize>() {
                        break parallelization;
                    }
                    println!("Please enter a valid positive number.");
                    std::io::stdout().flush().unwrap();
                }
                Err(ReadlineError::Interrupted) => return Err(Error::msg(CTRL_C_MESSAGE)),
                Err(ReadlineError::Eof) => return Err(Error::msg(CTRL_D_MESSAGE)),
                Err(err) => return Err(Error::from(err)),
            }
        }
    };

    compare(
        games,
        &player_1,
        &player_2,
        std::time::Duration::from_millis(args.update),
        parallelization,
    )
}

#[allow(clippy::too_many_lines)]
fn compare(
    iterations: usize,
    player_1: &PlayerType,
    player_2: &PlayerType,
    update: std::time::Duration,
    parallelization: usize,
) -> anyhow::Result<()> {
    println!(
        "Comparing {} iterations with {} threads: {} vs. {}",
        iterations,
        parallelization,
        player_1.name(),
        player_2.name()
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

    let mut recorded_games = vec![];
    let iterations_done = AtomicU32::new(0);
    std::thread::scope(|s| {
        let mut handles = vec![];

        for _ in 0..parallelization {
            let iterations = iterations as u32;
            let iterations_done = &iterations_done;
            let max_player_1_score = &max_player_1_score;
            let max_player_2_score = &max_player_2_score;
            let min_player_1_score = &min_player_1_score;
            let min_player_2_score = &min_player_2_score;
            let sum_player_1_score = &sum_player_1_score;
            let sum_player_2_score = &sum_player_2_score;
            let sum_time_player_1 = &sum_time_player_1;
            let sum_time_player_2 = &sum_time_player_2;
            let turns_player_1 = &n_time_player_1;
            let turns_player_2 = &n_time_player_2;
            let wins_player_1 = &wins_player_1;
            let wins_player_2 = &wins_player_2;
            let player_1_str = player_1.get_construct_name();
            let player_2_str = player_2.get_construct_name();
            handles.push(s.spawn(move || {
                let panic_result = panic::catch_unwind(move || {
                    let mut recorded_games = vec![];
                    let mut player_1 = get_player(player_1_str, Logging::Disabled).unwrap();
                    let mut player_2 = get_player(player_2_str, Logging::Disabled).unwrap();

                    'outer: while iterations_done.load(Ordering::Acquire) < iterations {
                        let mut state = Patchwork::get_initial_state(None);
                        loop {
                            if iterations_done.load(Ordering::Acquire) >= iterations {
                                break 'outer;
                            }

                            let start_time = std::time::Instant::now();
                            let action = if state.is_player_1() {
                                let action = player_1.get_action(&state).unwrap();
                                let end =
                                    u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos())
                                        .unwrap();

                                sum_time_player_1.fetch_add(end, Ordering::Relaxed);
                                turns_player_1.fetch_add(1, Ordering::Relaxed);
                                action
                            } else {
                                let action = player_2.get_action(&state).unwrap();
                                let end =
                                    u64::try_from(std::time::Instant::now().duration_since(start_time).as_nanos())
                                        .unwrap();
                                sum_time_player_2.fetch_add(end, Ordering::Relaxed);
                                turns_player_2.fetch_add(1, Ordering::Relaxed);
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

                                recorded_games.push(RecordedGame {
                                    player_1_name: player_1.name().to_string(),
                                    player_2_name: player_2.name().to_string(),
                                    result: termination.termination,
                                });

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

                    recorded_games
                });

                match panic_result {
                    Ok(recorded_games) => recorded_games,
                    Err(cause) => {
                        iterations_done.store(u32::MAX, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_secs(1)); // Wait progress printing

                        println!("\n\n\n\n\n");

                        println!("Panic in thread: {:?}", std::thread::current().id());
                        cause.downcast_ref::<String>().map_or_else(
                            || {
                                println!("Cause: {cause:?}");
                            },
                            |cause| {
                                println!("Cause: {cause}");
                            },
                        );

                        println!("\n\n\n\n\n");
                        vec![]
                    }
                }
            }));
        }
        loop {
            let iterations_done = iterations_done.load(Ordering::Relaxed) as usize;
            if iterations_done >= iterations {
                break;
            }
            print_progress(
                &mut std::io::stdout(),
                iterations_done,
                iterations,
                wins_player_1.load(Ordering::Relaxed) as usize,
                wins_player_2.load(Ordering::Relaxed) as usize,
                max_player_1_score.load(Ordering::Relaxed),
                max_player_2_score.load(Ordering::Relaxed),
                min_player_1_score.load(Ordering::Relaxed),
                min_player_2_score.load(Ordering::Relaxed),
                f64::from(sum_player_1_score.load(Ordering::Relaxed)),
                f64::from(sum_player_2_score.load(Ordering::Relaxed)),
                sum_time_player_1.load(Ordering::Relaxed) as f64,
                sum_time_player_2.load(Ordering::Relaxed) as f64,
                n_time_player_1.load(Ordering::Relaxed) as f64,
                n_time_player_2.load(Ordering::Relaxed) as f64,
                player_1.name(),
                player_2.name(),
            )?;
            std::thread::sleep(update);
        }

        for handle in handles {
            match handle.join() {
                Ok(games) => recorded_games.extend(games),
                Err(_) => {}
            }
        }

        anyhow::Result::<()>::Ok(())
    })?;

    atomic::fence(Ordering::SeqCst);

    print_progress(
        &mut std::io::stdout(),
        iterations_done.load(Ordering::Relaxed) as usize,
        iterations,
        wins_player_1.load(Ordering::Relaxed) as usize,
        wins_player_2.load(Ordering::Relaxed) as usize,
        max_player_1_score.load(Ordering::Relaxed),
        max_player_2_score.load(Ordering::Relaxed),
        min_player_1_score.load(Ordering::Relaxed),
        min_player_2_score.load(Ordering::Relaxed),
        f64::from(sum_player_1_score.load(Ordering::Relaxed)),
        f64::from(sum_player_2_score.load(Ordering::Relaxed)),
        sum_time_player_1.load(Ordering::Relaxed) as f64,
        sum_time_player_2.load(Ordering::Relaxed) as f64,
        n_time_player_1.load(Ordering::Relaxed) as f64,
        n_time_player_2.load(Ordering::Relaxed) as f64,
        player_1.name(),
        player_2.name(),
    )?;

    let rating_folder = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("analysis").join("player-rating");
    let display_output = rating_folder.join("output.txt");
    let games_output = rating_folder.join("games.txt");

    let output = OpenOptions::new().append(true).create(true).open(display_output)?;
    let mut writer = BufWriter::new(output);
    print_progress(
        &mut writer,
        iterations_done.load(Ordering::Relaxed) as usize,
        iterations,
        wins_player_1.load(Ordering::Relaxed) as usize,
        wins_player_2.load(Ordering::Relaxed) as usize,
        max_player_1_score.load(Ordering::Relaxed),
        max_player_2_score.load(Ordering::Relaxed),
        min_player_1_score.load(Ordering::Relaxed),
        min_player_2_score.load(Ordering::Relaxed),
        f64::from(sum_player_1_score.load(Ordering::Relaxed)),
        f64::from(sum_player_2_score.load(Ordering::Relaxed)),
        sum_time_player_1.load(Ordering::Relaxed) as f64,
        sum_time_player_2.load(Ordering::Relaxed) as f64,
        n_time_player_1.load(Ordering::Relaxed) as f64,
        n_time_player_2.load(Ordering::Relaxed) as f64,
        player_1.name(),
        player_2.name(),
    )?;

    let output = OpenOptions::new().append(true).create(true).open(games_output)?;
    let mut writer = BufWriter::new(output);
    for game in recorded_games {
        // Write Portable Game Notation (PGN)
        writeln!(
            writer,
            "Game:\n[White \"{}\"]\n[Black \"{}\"]\n[Result \"{}\"]\n\n",
            game.player_1_name,
            game.player_2_name,
            match game.result {
                TerminationType::Player1Won => "1-0",
                TerminationType::Player2Won => "0-1",
            }
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn print_progress(
    output: &mut impl Write,
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
    sum_time_player_2: f64,
    turns_player_1: f64,
    turns_player_2: f64,
    player_1_name: &str,
    player_2_name: &str,
) -> anyhow::Result<()> {
    let avg_player_1_score = avg_player_1_score / iteration as f64;
    let avg_player_2_score = avg_player_2_score / iteration as f64;

    let avg_player_1_time = sum_time_player_1 / turns_player_1;
    let avg_player_2_time = sum_time_player_2 / turns_player_2;

    write!(output, "\x1b[4A\r")?;
    writeln!(output, "Iteration {iteration: >7} / {iterations}")?;
    writeln!(output,
        "Player 1: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}, avg time: {: >9.3?}, turns: {}]                       ",
        wins_player_1,
        (wins_player_1 as f64 / iteration as f64 * 100.0),
        avg_player_1_score,
        if max_player_1_score == i32::MIN { 0 } else { max_player_1_score },
        if min_player_1_score == i32::MAX { 0 } else { min_player_1_score },
        std::time::Duration::from_nanos(avg_player_1_time.round() as u64),
        turns_player_1
    )?;
    writeln!(output,
        "Player 2: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}, avg time: {: >9.3?}, turns: {}]                       ",
        wins_player_2,
        (wins_player_2 as f64 / iteration as f64 * 100.0),
        avg_player_2_score,
        if max_player_2_score == i32::MIN { 0 } else { max_player_2_score },
        if min_player_2_score == i32::MAX { 0 } else { min_player_2_score },
        std::time::Duration::from_nanos(avg_player_2_time.round() as u64),
        turns_player_2
    )?;
    let progress_bar_length = 100;

    if iteration == 0 {
        writeln!(
            output,
            "{} {} {}  ",
            &player_1_name.chars().take(30).collect::<String>(),
            "█".repeat(progress_bar_length),
            &player_2_name.chars().take(30).collect::<String>(),
        )?;
    } else {
        let progress_player_1 = (wins_player_1 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_player_2 = (progress_bar_length as i32 - progress_player_1 as i32).max(0) as usize;
        writeln!(
            output,
            "{} \x1b[0;32m{}\x1b[0;31m{}\x1b[0m {}  ",
            player_1_name,
            "█".repeat(progress_player_1),
            "█".repeat(progress_player_2),
            player_2_name
        )?;
    }
    output.flush()?;
    Ok(())
}
