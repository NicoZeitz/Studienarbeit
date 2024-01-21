mod deserialization;

use patchwork_core::TurnType;

use crate::deserialization::GameLoader;

fn get_game_statistics(input: &std::path::PathBuf, output: &std::path::Path, gather: Gather) {
    let mut game_length_writer = if gather.game {
        Some(
            csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(output.join("game_lengths.csv"))
                .unwrap(),
        )
    } else {
        None
    };
    let mut available_actions_writer = if gather.available_actions {
        Some(
            csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(output.join("available_actions.csv"))
                .unwrap(),
        )
    } else {
        None
    };

    println!("Getting game statistics from {:?}", input);
    let mut games = 0;
    for game in GameLoader::new(input, None) {
        // Game length writer
        if gather.game {
            let game_length = game.turns.iter().filter(|turn| turn.action.is_some()).count();
            game_length_writer.as_mut().unwrap().serialize((game_length,)).unwrap();
        }

        // Available actions writer
        // This slows down the process a lot as we need to calculate the available actions for each state
        if gather.available_actions {
            let available_actions = game
                .turns
                .iter()
                .map(|turn| turn.state.clone())
                .filter(|state| !matches!(state.turn_type, TurnType::SpecialPatchPlacement) && !state.is_terminated())
                .map(|state| state.get_valid_actions())
                .filter(|actions| {
                    if actions.is_empty() {
                        return false;
                    }

                    if actions[0].is_special_patch_placement() {
                        return false;
                    }

                    true
                })
                .map(|actions| actions.len());
            for available_action in available_actions {
                available_actions_writer
                    .as_mut()
                    .unwrap()
                    .serialize((available_action,))
                    .unwrap();
            }
        }

        // game.turns
        //     .iter()
        //     .map(|turn| turn.action)
        //     .filter_map(|action| action)
        //     .filter(|action| !action.is_special_patch_placement());

        games += 1;
        if games % 10000 == 0 {
            print!("\r================= Game {} =================", games);
        }
    }
    println!("\r================= FINISHED GATHERING STATISTICS =================");
}

struct Gather {
    game: bool,
    available_actions: bool,
}

fn main() {
    let cmd = clap::Command::new("empirical-measurement")
        .bin_name("empirical-measurement")
        .about("Gets different empirical measurements from a set of recorded games")
        .arg(
            clap::Arg::new("in")
                .short('i')
                .long("input")
                .alias("in")
                .help("The path to the directory where the recorded games are")
                .required(true)
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .arg(
            clap::Arg::new("out")
                .short('o')
                .long("output")
                .alias("out")
                .help("The path to the directory where to store the extracted statistical data")
                .required(true)
                .value_parser(clap::value_parser!(std::path::PathBuf)),
        )
        .arg(
            // List all things to gather e.g. --game --available-actions
            clap::Arg::new("game")
                .long("game")
                .alias("game")
                .default_value("false")
                .value_parser(clap::value_parser!(bool))
                .help("Gathers statistics about the length of games"),
        )
        .arg(
            clap::Arg::new("available-actions")
                .long("available-actions")
                .alias("available-actions")
                .default_value("false")
                .value_parser(clap::value_parser!(bool))
                .help("Gathers statistics about the number of available actions per turn"),
        );

    let matches = cmd.get_matches();
    get_game_statistics(
        matches.get_one::<std::path::PathBuf>("in").unwrap(),
        matches.get_one::<std::path::PathBuf>("out").unwrap(),
        Gather {
            game: *matches.get_one::<bool>("game").unwrap_or(&false),
            available_actions: *matches.get_one::<bool>("available-actions").unwrap_or(&false),
        },
    );
}
