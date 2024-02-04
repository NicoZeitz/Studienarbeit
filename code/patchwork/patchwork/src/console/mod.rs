use std::io::Write;

use rustyline::{history::FileHistory, Editor};

use crate::exit::handle_exit;
use patchwork_lib::{Patchwork, TerminationType};

use crate::player as player_mod;

pub fn handle_console(rl: &mut Editor<(), FileHistory>) {
    let mut player_1 = loop {
        match rl.readline_with_initial("Player 1: ", ("Human", "")) {
            Ok(player) => {
                if let Some(player) = player_mod::get_player(&player.to_ascii_lowercase(), 1, Default::default()) {
                    break player;
                } else {
                    println!("Could not find player {}. Available players: ", player);
                    for player in player_mod::get_available_players() {
                        println!("  {}", player);
                    }
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };
    let mut player_2 = loop {
        match rl.readline("Player 2: ") {
            Ok(player) => {
                if let Some(player) = player_mod::get_player(&player.to_ascii_lowercase(), 2, Default::default()) {
                    break player;
                } else {
                    println!("Could not find player {}. Available players: ", player);
                    for player in player_mod::get_available_players() {
                        println!("  {}", player);
                    }
                    std::io::stdout().flush().unwrap();
                }
            }
            Err(_) => handle_exit(),
        }
    };

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
                TerminationType::Player1Won => println!("Player 1 ({}) won!", player_1.name()),
                TerminationType::Player2Won => println!("Player 2 ({}) won!", player_2.name()),
            }

            println!("{}", termination.player_1_score);
            println!("{}", termination.player_2_score);
            break;
        }

        i += 1;
    }
}
