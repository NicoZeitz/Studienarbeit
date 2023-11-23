use patchwork::prelude::*;
use std::env;

pub fn main() {
    let mut args: Vec<String> = env::args().map(|arg| arg.to_lowercase()).collect();

    args.push("npc".to_string());

    if args.contains(&"npc".to_string()) {
        let mut player_1 = MCTSPlayer::new("MCTS Player 1".to_owned(), None);
        // let mut player_1 = RandomPlayer::new("Random Player 1".to_owned(), None);
        let mut player_2 = RandomPlayer::new("Random Player 2".to_owned(), None);
        game_loop(&mut player_1, &mut player_2);
    } else if args.contains(&"test".to_string()) {
        test();
    } else {
        // play command
        let mut player_1 = HumanPlayer::new("Human Player 1".to_owned());
        let mut player_2 = RandomPlayer::new("Random Player 2".to_owned(), None);
        game_loop(&mut player_1, &mut player_2);
    }
}

fn game_loop(player_1: &mut impl Player, player_2: &mut impl Player) {
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

pub fn test() {
    let mut max_player_1_score = i32::MIN;
    let mut max_player_2_score = i32::MIN;
    let mut min_player_1_score = i32::MAX;
    let mut min_player_2_score = i32::MAX;
    let mut player_1_wins = 0;
    let mut player_2_wins = 0;
    let mut draws = 0;

    #[allow(unused_variables)]
    for i in 0..100_000 {
        let mut player_2 = RandomPlayer::new("Random Player 2".to_owned(), None);
        let mut player_1 = RandomPlayer::new("Random Player 1".to_owned(), None);
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
                        player_1_wins += 1;
                    }
                    TerminationType::Player2Won => {
                        player_2_wins += 1;
                    }
                    TerminationType::Draw => {
                        draws += 1;
                    }
                }

                max_player_1_score = max_player_1_score.max(termination.player_1_score);
                max_player_2_score = max_player_2_score.max(termination.player_2_score);
                min_player_1_score = min_player_1_score.min(termination.player_1_score);
                min_player_2_score = min_player_2_score.min(termination.player_2_score);
                break;
            }
        }
        #[cfg(debug_assertions)]
        print!("\rIteration {}", i + 1);
    }

    println!();
    println!("max_player_1_score: {}", max_player_1_score);
    println!("max_player_2_score: {}", max_player_2_score);
    println!("min_player_1_score: {}", min_player_1_score);
    println!("min_player_2_score: {}", min_player_2_score);
    println!("player_1_wins: {}", player_1_wins);
    println!("player_2_wins: {}", player_2_wins);
    println!("draws: {}", draws);
}
