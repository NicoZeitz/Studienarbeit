mod deserialization;

use patchwork_core::{PatchManager, TerminationType, TurnType};

use crate::deserialization::GameLoader;

fn get_game_statistics(input: &std::path::PathBuf, output: &std::path::Path, gather: Gather) {
    if !gather.has_something() {
        println!("Nothing to gather");
        return;
    }

    // Create output directory if it does not exist
    if !output.exists() {
        std::fs::create_dir_all(output).unwrap();
    }

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

    let mut available_special_actions_writer = if gather.available_special_actions {
        Some(
            csv::WriterBuilder::new()
                .has_headers(false)
                .from_path(output.join("available_special_actions.csv"))
                .unwrap(),
        )
    } else {
        None
    };
    let mut action_scores_map = std::collections::HashMap::new();

    println!("Getting game statistics from {:?}", input);
    let mut games = 0;
    let mut no_games = true;
    for game in GameLoader::new(input, None) {
        no_games = false;

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
                .filter(|turn| {
                    matches!(turn.state.turn_type, TurnType::Normal | TurnType::NormalPhantom)
                        && !turn.state.is_terminated()
                })
                .map(|turn| turn.state.get_valid_actions())
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

        // Available special actions writer
        // This slows down the process a lot (but way less than available actions) as we need to calculate the available actions for each state
        if gather.available_special_actions {
            let available_special_actions = game
                .turns
                .iter()
                .filter(|turn| {
                    matches!(
                        turn.state.turn_type,
                        TurnType::SpecialPatchPlacement | TurnType::SpecialPhantom
                    ) && !turn.state.is_terminated()
                })
                .map(|turn| turn.state.get_valid_actions())
                .filter(|actions| {
                    if actions.is_empty() {
                        return false;
                    }

                    if !actions[0].is_special_patch_placement() {
                        return false;
                    }

                    true
                })
                .map(|actions| actions.len());
            for available_special_action in available_special_actions {
                available_special_actions_writer
                    .as_mut()
                    .unwrap()
                    .serialize((available_special_action,))
                    .unwrap();
            }
        }

        // Action scores writer
        if gather.action_scores {
            let end_state = &game.turns.last().unwrap().state;
            assert!(
                end_state.is_terminated(),
                "[get_game_statistics(action_scores)] Game is not terminated"
            );

            let result = end_state.get_termination_result();

            game.turns.iter().filter(|turn| turn.action.is_some()).for_each(|turn| {
                let action = turn.action.unwrap();
                let is_player_1 = turn.state.is_player_1();

                let score = match result.termination {
                    TerminationType::Player1Won => {
                        if is_player_1 {
                            1
                        } else {
                            -1
                        }
                    }
                    TerminationType::Player2Won => {
                        if is_player_1 {
                            -1
                        } else {
                            1
                        }
                    }
                };

                // TODO: look at with ply offset
                // TODO: look at actual_score vs only score

                let actual_score = score * ((result.player_1_score - result.player_2_score).abs() + 1);
                let key = if action.is_walking() {
                    0
                } else if action.is_special_patch_placement() {
                    action.get_quilt_board_index() as u32 + 1
                } else if action.is_patch_placement() {
                    action.get_patch_id() as u32 * PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS
                        + action.get_patch_transformation_index() as u32
                        + 82
                } else {
                    unreachable!(
                        "[get_game_statistics(action_scores)] Other actions types should not be in the dataset"
                    )
                };
                let description = if action.is_walking() {
                    "walking".to_string()
                } else if action.is_special_patch_placement() {
                    format!("special_patch_placement({})", action.get_quilt_board_index())
                } else if action.is_patch_placement() {
                    format!(
                        "patch_placement({}, {})",
                        action.get_patch_id(),
                        action.get_patch_transformation_index()
                    )
                } else {
                    unreachable!(
                        "[get_game_statistics(action_scores)] Other actions types should not be in the dataset"
                    )
                };

                let entry = action_scores_map.entry(key).or_insert((description, 0, 0));
                entry.1 += actual_score;
                entry.2 += score;
            });
        }

        games += 1;
        if games % 10000 == 0 {
            print!("\r================= Game {} =================", games);
        }
    }

    if no_games {
        println!("No games found");
    }

    if gather.action_scores {
        println!("Running post processing for action scores");
        let mut action_scores_writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(output.join("action_scores.csv"))
            .unwrap();

        let mut action_scores_win_loss_writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(output.join("action_scores_win_loss.csv"))
            .unwrap();

        let mut data_vector = action_scores_map.values().collect::<Vec<_>>();
        data_vector.sort_by_key(|(desc, _, _)| desc);

        for (description, score, win_loss) in data_vector {
            action_scores_writer.serialize((description, score)).unwrap();
            action_scores_win_loss_writer
                .serialize((description, win_loss))
                .unwrap();
        }
    }

    println!("\r================= FINISHED GATHERING STATISTICS =================");
}

struct Gather {
    game: bool,
    available_actions: bool,
    available_special_actions: bool,
    action_scores: bool,
}

impl Gather {
    pub fn has_something(&self) -> bool {
        self.game || self.available_actions || self.available_special_actions || self.action_scores
    }
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
                .required(false)
                .num_args(0)
                .help("Gathers statistics about the length of games"),
        )
        .arg(
            clap::Arg::new("available-actions")
                .long("available-actions")
                .required(false)
                .num_args(0)
                .help("Gathers statistics about the number of available actions per turn"),
        )
        .arg(
            clap::Arg::new("available-special-actions")
                .long("available-special-actions")
                .required(false)
                .num_args(0)
                .help("Gathers statistics about the number of available special actions per turn"),
        )
        .arg(
            clap::Arg::new("action-scores")
                .long("action-scores")
                .required(false)
                .num_args(0)
                .help("Gathers statistics about the scores of actions"),
        );

    let matches = cmd.get_matches();
    get_game_statistics(
        matches.get_one::<std::path::PathBuf>("in").unwrap(),
        matches.get_one::<std::path::PathBuf>("out").unwrap(),
        Gather {
            game: matches.get_flag("game"),
            available_actions: matches.get_flag("available-actions"),
            available_special_actions: matches.get_flag("available-special-actions"),
            action_scores: matches.get_flag("action-scores"),
        },
    );
}
