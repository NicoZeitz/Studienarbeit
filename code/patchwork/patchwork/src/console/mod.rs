use clap::Parser;
use rustyline::{history::FileHistory, Editor};

use crate::common::{interactive_get_diagnostics, interactive_get_player, PlayerType};
use patchwork_lib::{player::Player, GameOptions, Notation, Patchwork, TerminationType};

#[derive(Debug, Parser, Default)]
#[command(no_binary_name(true))]
struct CmdArgs {
    #[arg(long = "player-1", alias = "p1", short = '1')]
    player_1: Option<String>,
    #[arg(long = "player-2", alias = "p2", short = '2')]
    player_2: Option<String>,
    #[arg(long = "diagnostics-1", alias = "d1")]
    diagnostics_player_1: Option<String>,
    #[arg(long = "diagnostics-2", alias = "d2")]
    diagnostics_player_2: Option<String>,
    #[arg(long = "seed", short = 's')]
    seed: Option<u64>,
}

pub fn handle_console(rl: &mut Editor<(), FileHistory>, args: Vec<String>) -> anyhow::Result<()> {
    let args = CmdArgs::parse_from(args);

    let player_1_diagnostics = interactive_get_diagnostics(rl, 1, args.diagnostics_player_1)?;
    let player_2_diagnostics = interactive_get_diagnostics(rl, 2, args.diagnostics_player_2)?;

    let player_1 = interactive_get_player(rl, args.player_1, 1, player_1_diagnostics)?;
    let player_2 = interactive_get_player(rl, args.player_2, 2, player_2_diagnostics)?;

    handle_console_repl(player_1, player_2, args.seed)
}

fn handle_console_repl(mut player_1: PlayerType, mut player_2: PlayerType, seed: Option<u64>) -> anyhow::Result<()> {
    let mut state = Patchwork::get_initial_state(seed.map(|seed| GameOptions { seed }));

    let mut i = 1;
    loop {
        println!("─────────────────────────────────────────────────── TURN {} ──────────────────────────────────────────────────", i);
        println!("{}", state);

        #[cfg(debug_assertions)]
        let old_state = state.clone();

        let start_time = std::time::Instant::now();
        let action = if state.is_player_1() {
            player_1.get_action(&state)?
        } else {
            player_2.get_action(&state)?
        };
        let end_time = std::time::Instant::now();

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
            "Player '{}' chose action: {} ({}) after {:?}",
            if state.is_player_1() {
                player_1.name()
            } else {
                player_2.name()
            },
            action,
            action.save_to_notation().unwrap_or("######".to_string()),
            end_time - start_time
        );

        let mut next_state = state.clone();
        next_state.do_action(action, false)?;
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

    Ok(())
}
