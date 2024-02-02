use patchwork_lib::{
    evaluator::StaticEvaluator,
    player::{
        AlphaZeroPlayer, Diagnostics, GreedyPlayer, HumanPlayer, MCTSOptions, MCTSPlayer, MinimaxOptions,
        MinimaxPlayer, PVSOptions, PVSPlayer, Player, RandomOptions, RandomPlayer,
    },
    tree_policy::UCTPolicy,
};
use regex::Regex;

#[allow(clippy::field_reassign_with_default)]
pub fn get_player(name: &str, player_position: usize, diagnostics: Diagnostics) -> Option<Box<dyn Player>> {
    match name.to_ascii_lowercase().as_str() {
        // HUMAN
        "human" => Some(Box::new(HumanPlayer::new(format!("Human Player {player_position}")))),
        _ if name.starts_with("human") => {
            let name = Regex::new(r"human\((?<name>.+)\)")
                .unwrap()
                .captures(name)
                .unwrap()
                .name("name")
                .unwrap()
                .as_str();
            Some(Box::new(HumanPlayer::new(name)))
        }
        // SIMPLE ENGINES
        "random" => Some(Box::new(RandomPlayer::new(
            format!("Random Player {player_position}"),
            None,
        ))),
        _ if name.starts_with("random") => {
            let seed = Regex::new(r"random\((?<seed>\d+)\)")
                .unwrap()
                .captures(name)
                .unwrap()
                .name("seed")
                .unwrap()
                .as_str()
                .parse()
                .unwrap();
            Some(Box::new(RandomPlayer::new(
                format!("Random Player {player_position} ({})", seed),
                Some(RandomOptions::new(seed)),
            )))
        }
        "greedy" => Some(Box::new(GreedyPlayer::new(format!("Greedy Player {player_position}")))),
        // TREE SEARCH ENGINES
        "minimax" => Some(Box::new(MinimaxPlayer::new(
            format!("Minimax Player {player_position}"),
            Default::default(),
        ))),
        _ if name.starts_with("minimax") => {
            let regex = Regex::new(r"minimax\((?<depth>\d+),\s*(?<patches>\d+)\)").unwrap();
            let captures = regex.captures(name).unwrap();
            let depth = captures.name("depth").unwrap().as_str().parse().unwrap();
            let amount_actions_per_piece = captures.name("patches").unwrap().as_str().parse().unwrap();
            Some(Box::new(MinimaxPlayer::new(
                format!(
                    "Minimax Player {player_position} ({}, {})",
                    depth, amount_actions_per_piece
                ),
                Some(MinimaxOptions::new(depth, amount_actions_per_piece)),
            )))
        }
        "pvs" => Some(Box::new(PVSPlayer::new(format!("PVS Player {player_position}"), None))),
        _ if name.starts_with("pvs") => {
            let regex = Regex::new(r"pvs\((?<time>\d+(?:\.\d+)?)\)").unwrap();
            let captures = regex.captures(name).unwrap();
            let time = captures.name("time").unwrap().as_str().parse().unwrap();
            let mut options = PVSOptions::default();
            options.time_limit = std::time::Duration::from_secs_f64(time);

            Some(Box::new(PVSPlayer::new(
                format!("PVS Player {player_position} ({})", time),
                Some(options),
            )))
        }
        // MCTS ENGINES
        "mcts" => Some(Box::new(MCTSPlayer::<UCTPolicy, StaticEvaluator>::new(
            format!("MCTS Player {player_position}"),
            Some(MCTSOptions {
                diagnostics,
                ..Default::default()
            }),
        ))),
        "alphazero" => Some(Box::new(AlphaZeroPlayer::new(format!(
            "AlphaZero Player {player_position}"
        )))),
        // Not found
        _ => None,
    }
}

pub fn get_available_players() -> Vec<String> {
    [
        "human",
        "human(name: string)",
        "random",
        "random(seed: int)",
        "greedy",
        "minimax",
        "minimax(depth: int, patches: int)",
        "pvs",
        "pvs(time: float)",
        "mcts",
        "alphazero",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
