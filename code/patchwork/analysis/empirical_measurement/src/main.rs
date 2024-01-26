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
            game.turns
                .iter()
                .enumerate()
                .filter(|(_, turn)| turn.action.is_some())
                .for_each(|(ply, turn)| {
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
                        format!("special_patch_placement({:02})", action.get_quilt_board_index())
                    } else if action.is_patch_placement() {
                        format!(
                            "patch_placement({:02}, {:03})",
                            action.get_patch_id(),
                            action.get_patch_transformation_index()
                        )
                    } else {
                        unreachable!(
                            "[get_game_statistics(action_scores)] Other actions types should not be in the dataset"
                        )
                    };

                    let percentage = ply as f64 / game.turns.len() as f64;

                    let entry = action_scores_map.entry((key, F64Key(percentage))).or_insert((
                        description,
                        percentage,
                        0,
                        0,
                        0,
                    ));

                    entry.2 += actual_score;
                    entry.3 += score;
                    entry.4 += 1; // count
                });
        }

        games += 1;
        if games % 10000 == 0 {
            print!("\r================= Game {} =================", games);
        }
    }

    println!();

    if no_games {
        println!("No games found");
    }

    if gather.action_scores {
        println!("Running post processing for action scores");
        let mut action_scores_writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(output.join("action_scores.csv"))
            .unwrap();

        let mut data_vector = action_scores_map.values().collect::<Vec<_>>();
        data_vector.sort_by_key(|(desc, percentage, _, _, _)| {
            if desc == "walking" {
                (0, desc, F64Key(*percentage))
            } else if desc.starts_with("special_patch_placement") {
                (1, desc, F64Key(*percentage))
            } else if desc.starts_with("patch_placement") {
                (2, desc, F64Key(*percentage))
            } else {
                unreachable!("[get_game_statistics(action_scores)] Other actions types should not be in the dataset")
            }
        });

        for (description, percentage, score, win_loss, count) in data_vector {
            action_scores_writer
                .serialize((description, percentage, score, win_loss, count))
                .unwrap();
        }
    }

    println!("================= FINISHED GATHERING STATISTICS =================");
}

struct F64Key(pub f64);

impl std::cmp::PartialEq for F64Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl std::cmp::Eq for F64Key {}
impl std::cmp::PartialOrd for F64Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for F64Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}
impl std::hash::Hash for F64Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
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
