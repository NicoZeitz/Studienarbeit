use std::sync::atomic::{self, AtomicI32, AtomicU32, Ordering};

use patchwork::{
    player::{
        AlphaZeroPlayer, GreedyPlayer, HumanPlayer, MCTSPlayer, MinimaxPlayer, NegamaxPlayer,
        Player, RandomPlayer,
    },
    Game, Patchwork, TerminationType,
};

pub struct GameLoop;

impl GameLoop {
    pub fn get_player(player: &str, player_position: usize) -> Box<dyn Player<Game = Patchwork>> {
        match player {
            "human" => Box::new(HumanPlayer::new(format!("Human Player {player_position}"))),
            "random" => Box::new(RandomPlayer::new(
                format!("Random Player {player_position}"),
                None,
            )),
            "greedy" => Box::new(GreedyPlayer::new(format!(
                "Greedy Player {player_position}"
            ))),
            "mcts" => Box::new(MCTSPlayer::new(
                format!("MCTS Player {player_position}"),
                Default::default(),
            )),
            "minimax" => Box::new(MinimaxPlayer::new(format!(
                "Minimax Player {player_position}"
            ))),
            "negamax" => Box::new(NegamaxPlayer::new(format!(
                "Negamax Player {player_position}"
            ))),
            "alphazero" => Box::new(AlphaZeroPlayer::new(format!(
                "AlphaZero Player {player_position}"
            ))),
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
            println!("=================================================== TURN {} ==================================================", i);
            println!("{}", state);

            let action = if state.is_player_1() {
                player_1.get_action(&state)
            } else {
                player_2.get_action(&state)
            };

            println!(
                "Player '{}' chose action: {}",
                if state.is_player_1() {
                    player_1.name()
                } else {
                    player_2.name()
                },
                action
            );

            state = state.get_next_state(&action);

            if state.is_terminated() {
                let termination = state.get_termination_result();

                println!("================================================== RESULT ====================================================");
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

    pub fn compare(iterations: usize, player_1: &str, player_2: &str, update: Option<usize>) {
        let update = update
            .map(|u| std::time::Duration::from_millis(u as u64))
            .unwrap_or(std::time::Duration::from_millis(100));
        let temp_player_1 = GameLoop::get_player(player_1, 1);
        let temp_player_2 = GameLoop::get_player(player_2, 2);
        let player_1_name = temp_player_1.name();
        let player_2_name = temp_player_2.name();

        let amount_parallel = std::thread::available_parallelism().unwrap().into();

        println!(
            "Comparing {} iterations with {} threads: {} vs. {}",
            iterations, amount_parallel, player_1_name, player_2_name
        );

        let max_player_1_score = AtomicI32::new(i32::MIN);
        let max_player_2_score = AtomicI32::new(i32::MIN);
        let min_player_1_score = AtomicI32::new(i32::MAX);
        let min_player_2_score = AtomicI32::new(i32::MAX);
        let sum_player_1_score = AtomicI32::new(0);
        let sum_player_2_score = AtomicI32::new(0);
        let wins_player_1 = AtomicU32::new(0);
        let wins_player_2 = AtomicU32::new(0);
        let draws = AtomicU32::new(0);

        print!("\n\n\n\n\n");

        let iterations_done = AtomicU32::new(0);
        let iterations_per_thread = iterations / amount_parallel;
        std::thread::scope(|s| {
            for i in 0..amount_parallel {
                let is_last = i + 1 == amount_parallel;
                let player_1 = player_1.to_string();
                let player_2 = player_2.to_string();
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
                s.spawn(move || {
                    let mut player_1 = GameLoop::get_player(&player_1, 1);
                    let mut player_2 = GameLoop::get_player(&player_2, 2);
                    let iterations_per_thread = if is_last {
                        iterations - iterations_per_thread * (amount_parallel - 1)
                    } else {
                        iterations_per_thread
                    };

                    for _ in 0..iterations_per_thread {
                        let mut state = Patchwork::get_initial_state(None);
                        loop {
                            let action = if state.is_player_1() {
                                player_1.get_action(&state)
                            } else {
                                player_2.get_action(&state)
                            };
                            state = state.get_next_state(&action);
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

                                max_player_1_score
                                    .fetch_max(termination.player_1_score, Ordering::Relaxed);
                                max_player_2_score
                                    .fetch_max(termination.player_2_score, Ordering::Relaxed);
                                min_player_1_score
                                    .fetch_min(termination.player_1_score, Ordering::Relaxed);
                                min_player_2_score
                                    .fetch_min(termination.player_2_score, Ordering::Relaxed);
                                sum_player_1_score
                                    .fetch_add(termination.player_1_score, Ordering::Relaxed);
                                sum_player_2_score
                                    .fetch_add(termination.player_2_score, Ordering::Relaxed);
                                iterations_done.fetch_add(1, Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                });
            }
            loop {
                let iterations_done = iterations_done.load(Ordering::Relaxed) as usize;
                if iterations_done == iterations {
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
        player_1_name: &str,
        player_2_name: &str,
    ) {
        let avg_player_1_score = avg_player_1_score / iteration as f64;
        let avg_player_2_score = avg_player_2_score / iteration as f64;

        print!("\x1b[5A\r");
        println!("Iteration {: >7} / {}", iteration, iterations);
        println!(
            "Player 1: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}]                       ",
            wins_player_1,
            (wins_player_1 as f64 / iteration as f64 * 100.0),
            avg_player_1_score,
            max_player_1_score,
            min_player_1_score,
        );
        println!(
            "Player 2: {: >7} wins  ({:0>5.2}%) [avg score: {: >6.02}, max score: {: >3}, min score: {: >3}]                       ",
            wins_player_2,
            (wins_player_2 as f64 / iteration as f64 * 100.0),
            avg_player_2_score,
            max_player_2_score,
            min_player_2_score,
        );
        println!(
            "          {: >7} draws ({:0>5.2}%)                       ",
            draws,
            (draws as f64 / iteration as f64 * 100.0)
        );
        let progress_bar_length: i32 = 120;
        let progress_player_1 =
            (wins_player_1 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_player_2 =
            (wins_player_2 as f64 / iteration as f64 * progress_bar_length as f64).round() as usize;
        let progress_draws =
            (progress_bar_length - progress_player_1 as i32 - progress_player_2 as i32).max(0)
                as usize;

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
