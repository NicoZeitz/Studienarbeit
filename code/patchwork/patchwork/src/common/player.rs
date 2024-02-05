use std::io::Write;

use anyhow::Error;
use patchwork_lib::{
    evaluator::{Evaluator, NeuralNetworkEvaluator, ScoreEvaluator, StaticEvaluator, WinLossEvaluator},
    player::{
        AlphaZeroPlayer, FailingStrategy, GreedyPlayer, HumanPlayer, LazySMPFeature, Logging, MCTSEndCondition,
        MCTSOptions, MCTSPlayer, MinimaxOptions, MinimaxPlayer, PVSOptions, PVSPlayer, Player, RandomOptions,
        RandomPlayer, Size, TranspositionTableFeature,
    },
    tree_policy::{PartiallyScoredUCTPolicy, ScoredUCTPolicy, TreePolicy, UCTPolicy},
    ActionId, ActionOrderer, Patchwork, TableActionOrderer,
};
use regex::Regex;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};

use super::{CTRL_C_MESSAGE, CTRL_D_MESSAGE};

pub enum PlayerType {
    BuildIn(Box<dyn Player>, String),
    #[allow(dead_code)]
    Upi(String), // TODO: implement extern UPI
}

impl PlayerType {
    pub fn get_construct_name(&self) -> &str {
        match self {
            PlayerType::BuildIn(_, name) => name,
            PlayerType::Upi(name) => name,
        }
    }
}

impl Player for PlayerType {
    fn name(&self) -> &str {
        match self {
            PlayerType::BuildIn(player, _) => player.name(),
            PlayerType::Upi(_) => unimplemented!("[PlayerType::name] UPI is not yet implemented."),
        }
    }

    fn get_action(&mut self, game: &Patchwork) -> anyhow::Result<ActionId> {
        match self {
            PlayerType::BuildIn(player, _) => player.get_action(game),
            PlayerType::Upi(_) => unimplemented!("[PlayerType::get_action] UPI is not yet implemented."),
        }
    }
}

pub fn interactive_get_player(
    rl: &mut Editor<(), FileHistory>,
    player_name: Option<String>,
    player_position: usize,
    logging: Logging,
) -> anyhow::Result<PlayerType> {
    if let Some(player_name) = player_name {
        let Ok(player) = get_player(player_name.as_str(), 1, logging) else {
            println!("Could not find player {}. Available players: ", player_name);
            for p in get_available_players() {
                println!("  {}", p);
            }
            std::io::stdout().flush()?;
            return Err(Error::msg(format!("Could not find player {}", player_position)));
        };
        Ok(player)
    } else {
        ask_for_player(rl, player_position, logging)
    }
}

fn ask_for_player(
    rl: &mut Editor<(), FileHistory>,
    player_position: usize,
    mut logging: Logging,
) -> anyhow::Result<PlayerType> {
    loop {
        // match rl.readline_with_initial("Player 1: ", ("Human", "")) {
        match rl.readline(format!("Player {}: ", player_position).as_str()) {
            Ok(player) => match get_player(&player, 1, logging) {
                Ok(player) => return Ok(player),
                Err(d) => {
                    logging = d;
                    println!("Could not find player {}. Available players: ", player);
                    for player in get_available_players() {
                        println!("  {}", player);
                    }
                    std::io::stdout().flush()?;
                }
            },
            Err(ReadlineError::Interrupted) => return Err(Error::msg(CTRL_C_MESSAGE)),
            Err(ReadlineError::Eof) => return Err(Error::msg(CTRL_D_MESSAGE)),
            Err(err) => return Err(Error::from(err)),
        }
    }
}

pub fn get_player(name: &str, player_position: usize, logging: Logging) -> Result<PlayerType, Logging> {
    let name = name.to_ascii_lowercase();
    let name = name.as_str();

    if name.starts_with("extern") {
        unimplemented!("[get_player_from_str] Extern upi players are not yet implemented.");
    }

    if let Some(player) = parse_human_player(name, player_position) {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    if let Some(player) = parse_random_player(name, player_position) {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    if let Some(player) = parse_greedy_player(name, player_position) {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    if let Some(player) = parse_minimax_player(name, player_position) {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    let (player_option, logging) = parse_pvs_player(name, player_position, logging);
    if let Some(player) = player_option {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    let (player_option, logging) = parse_mcts_player(name, player_position, logging.unwrap());
    if let Some(player) = player_option {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    if let Some(player) = parse_alphazero_player(name, player_position) {
        return Ok(PlayerType::BuildIn(player, name.to_string()));
    }

    Err(logging.unwrap())
}

pub fn get_available_players() -> Vec<String> {
    [
        "human",
        "human(name: string)",
        "random",
        "random(seed: uint)",
        "greedy",
        "minimax",
        "minimax(depth: uint, patches: uint)",
        "pvs",
        "pvs(time: float, ord: table, eval: static|win|score|nn, fail: hard|soft, asp: yes|no, lmr: yes|no, lmp: yes|no, ext: yes|no, tt: enabled|disabled, smp: yes|no)",
        "mcts",
        "mcts(time: float, iter: uint, tree: reuse|new, root: uint, leaf: uint, policy: uct|partial-score|score, eval: static|win|score|nn)",
        "alphazero",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn parse_human_player(name: &str, player_position: usize) -> Option<Box<dyn Player>> {
    if name == "human" {
        return Some(Box::new(HumanPlayer::new(format!("Human Player {player_position}"))));
    }

    if !name.starts_with("human") {
        return None;
    }

    let Some(passed_options) = Regex::new(r"human\((?<options>.*)\)")
        .unwrap()
        .captures(name)
        .and_then(|o| o.name("options"))
        .map(|o| o.as_str())
    else {
        return None;
    };

    let default_name = format!("Human Player {player_position}");
    let name = Regex::new(r"name:\s*(?<name>\w+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("name"))
        .map(|o| o.as_str())
        .unwrap_or(default_name.as_str());

    Some(Box::new(HumanPlayer::new(name)))
}

fn parse_random_player(name: &str, player_position: usize) -> Option<Box<dyn Player>> {
    if name == "random" {
        return Some(Box::new(RandomPlayer::new(
            format!("Random Player {player_position}",),
            None,
        )));
    }

    if !name.starts_with("random") {
        return None;
    }

    let Some(passed_options) = Regex::new(r"random\((?<options>.*)\)")
        .unwrap()
        .captures(name)
        .and_then(|o| o.name("options"))
        .map(|o| o.as_str())
    else {
        return None;
    };

    let mut options = RandomOptions::default();

    if let Some(seed) = Regex::new(r"seed:\s*(?<seed>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("seed"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.seed = seed;
    }

    Some(Box::new(RandomPlayer::new(
        format!("Random Player {player_position} (s: {})", options.seed),
        Some(options),
    )))
}

fn parse_greedy_player(name: &str, player_position: usize) -> Option<Box<dyn Player>> {
    if name == "greedy" {
        return Some(Box::new(GreedyPlayer::new(format!("Greedy Player {player_position}"))));
    }

    None
}

fn parse_minimax_player(name: &str, player_position: usize) -> Option<Box<dyn Player>> {
    if name == "minimax" {
        return Some(Box::new(MinimaxPlayer::new(
            format!("Minimax Player {player_position}"),
            None,
        )));
    }

    if !name.starts_with("minimax") {
        return None;
    }

    let Some(passed_options) = Regex::new(r"minimax\((?<options>.*)\)")
        .unwrap()
        .captures(name)
        .and_then(|o| o.name("options"))
        .map(|o| o.as_str())
    else {
        return None;
    };

    let mut options = MinimaxOptions::default();

    if let Some(depth) = Regex::new(r"depth:\s*(?<depth>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("depth"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.depth = depth;
    }

    if let Some(patches) = Regex::new(r"patches:\s*(?<patches>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("patches"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.amount_actions_per_piece = patches;
    }

    Some(Box::new(MinimaxPlayer::new(
        format!(
            "Minimax Player {player_position} (d: {}, p: {})",
            options.depth, options.amount_actions_per_piece
        ),
        Some(options),
    )))
}

fn parse_pvs_player(
    name: &str,
    player_position: usize,
    logging: Logging,
) -> (Option<Box<dyn Player>>, Option<Logging>) {
    fn create_player<Orderer: ActionOrderer + Default + 'static, Eval: Evaluator + Default + 'static>(
        player_position: usize,
        options: PVSOptions,
    ) -> Box<dyn Player> {
        Box::new(PVSPlayer::<Orderer, Eval>::new(
            format!("PVS Player {player_position}"),
            Some(options),
        ))
    }

    if name == "pvs" {
        let player: PVSPlayer = PVSPlayer::new(
            format!("PVS Player {player_position}"),
            Some(PVSOptions {
                logging,
                ..Default::default()
            }),
        );

        return (Some(Box::new(player)), None);
    }

    if !name.starts_with("pvs") {
        return (None, Some(logging));
    }

    let Some(passed_options) = Regex::new(r"pvs\((?<options>.*)\)")
        .unwrap()
        .captures(name)
        .and_then(|o| o.name("options"))
        .map(|o| o.as_str())
    else {
        return (None, Some(logging));
    };

    let mut options = PVSOptions::default();
    let mut orderer = "table";
    let mut evaluator = "static";
    options.logging = logging;

    if let Some(time_limit) = Regex::new(r"time:\s*(?<time>\d+(?:\.\d+)?)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("time"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.time_limit = std::time::Duration::from_secs_f64(time_limit);
    }

    if let Some(failing_strategy) = Regex::new(r"fail:\s*(?<fail>hard|soft)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("fail"))
        .map(|o| o.as_str())
    {
        if failing_strategy == "hard" {
            options.features.failing_strategy = FailingStrategy::FailHard;
        } else {
            options.features.failing_strategy = FailingStrategy::FailSoft;
        }
    }

    if let Some(aspiration_window) = Regex::new(r"asp:\s*(?<asp>yes|no)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("asp"))
        .map(|o| o.as_str())
    {
        options.features.aspiration_window = aspiration_window == "yes";
    }

    if let Some(late_move_reductions) = Regex::new(r"lmr:\s*(?<lmr>yes|no)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("lmr"))
        .map(|o| o.as_str())
    {
        options.features.late_move_reductions = late_move_reductions == "yes";
    }

    if let Some(late_move_pruning) = Regex::new(r"lmp:\s*(?<lmp>yes|no)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("lmp"))
        .map(|o| o.as_str())
    {
        options.features.late_move_pruning = late_move_pruning == "yes";
    }

    if let Some(search_extensions) = Regex::new(r"ext:\s*(?<ext>yes|no)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("ext"))
        .map(|o| o.as_str())
    {
        options.features.search_extensions = search_extensions == "yes";
    }

    if let Some(transposition_table) = Regex::new(r"tt:\s*(?<tt>enabled|disabled)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("tt"))
        .map(|o| o.as_str())
    {
        options.features.transposition_table = match transposition_table {
            "enabled" => TranspositionTableFeature::SymmetryEnabled {
                size: Size::MiB(10),
                strategy: options.features.failing_strategy,
            },
            "disabled" => TranspositionTableFeature::Disabled,
            _ => unreachable!(),
        };
    }

    if let Some(lazy_smp) = Regex::new(r"smp:\s*(?<smp>yes|no)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("smp"))
        .map(|o| o.as_str())
    {
        options.features.lazy_smp = match lazy_smp {
            "yes" => LazySMPFeature::default(),
            "no" => LazySMPFeature::No,
            _ => unreachable!(),
        };
    }

    if let Some(order) = Regex::new(r"ord:\s*(?<orderer>table)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("ord"))
        .map(|o| o.as_str())
    {
        orderer = order
    }

    if let Some(eval) = Regex::new(r"eval:\s*(?<eval>static|win|score|nn)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("eval"))
        .map(|o| o.as_str())
    {
        evaluator = eval
    }

    let player: Box<dyn Player> = match (orderer, evaluator) {
        ("table", "static") => create_player::<TableActionOrderer, StaticEvaluator>(player_position, options),
        ("table", "win") => create_player::<TableActionOrderer, WinLossEvaluator>(player_position, options),
        ("table", "score") => create_player::<TableActionOrderer, ScoreEvaluator>(player_position, options),
        ("table", "nn") => create_player::<TableActionOrderer, NeuralNetworkEvaluator>(player_position, options),
        _ => unreachable!(),
    };

    (Some(player), None)
}

fn parse_mcts_player(
    name: &str,
    player_position: usize,
    logging: Logging,
) -> (Option<Box<dyn Player>>, Option<Logging>) {
    fn create_player<Policy: TreePolicy + Default + 'static, Eval: Evaluator + Default + 'static>(
        player_position: usize,
        options: MCTSOptions,
    ) -> Box<dyn Player> {
        Box::new(MCTSPlayer::<Policy, Eval>::new(
            format!("MCTS Player {player_position}"),
            Some(options),
        ))
    }

    if name == "mcts" {
        let player: MCTSPlayer = MCTSPlayer::new(
            format!("MCTS Player {player_position}"),
            Some(MCTSOptions {
                logging,
                ..Default::default()
            }),
        );

        return (Some(Box::new(player)), None);
    }

    if !name.starts_with("mcts") {
        return (None, Some(logging));
    }

    let Some(passed_options) = Regex::new(r"mcts\((?<options>.*)\)")
        .unwrap()
        .captures(name)
        .and_then(|o| o.name("options"))
        .map(|o| o.as_str())
    else {
        return (None, Some(logging));
    };

    let mut options = MCTSOptions::default();
    let mut policy = "uct";
    let mut evaluator = "win";
    options.logging = logging;

    if let Some(time_limit) = Regex::new(r"time:\s*(?<time>\d+(?:\.\d+)?)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("time"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.end_condition = MCTSEndCondition::Time(std::time::Duration::from_secs_f64(time_limit));
    } else if let Some(iterations) = Regex::new(r"iter:\s*(?<iter>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("iter"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.end_condition = MCTSEndCondition::Iterations(iterations);
    }

    if let Some(reuse_tree) = Regex::new(r"tree:\s*(?<tree>reuse|new)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("tree"))
        .map(|o| o.as_str())
    {
        options.reuse_tree = reuse_tree == "reuse";
    }

    if let Some(root_parallelization) = Regex::new(r"root:\s*(?<root>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("root"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.root_parallelization = root_parallelization;
    }

    if let Some(leaf_parallelization) = Regex::new(r"leaf:\s*(?<leaf>\d+)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("leaf"))
        .and_then(|o| o.as_str().parse().ok())
    {
        options.leaf_parallelization = leaf_parallelization;
    }

    if let Some(pol) = Regex::new(r"policy:\s*(?<policy>uct|partial-score|score)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("policy"))
        .map(|o| o.as_str())
    {
        policy = pol;
    }

    if let Some(eval) = Regex::new(r"eval:\s*(?<eval>static|win|score|nn)")
        .unwrap()
        .captures(passed_options)
        .and_then(|o| o.name("eval"))
        .map(|o| o.as_str())
    {
        evaluator = eval;
    }

    #[rustfmt::skip]
    let player: Box<dyn Player> = match (policy, evaluator) {
        ("uct", "static") => create_player::<UCTPolicy, StaticEvaluator>(player_position, options),
        ("uct", "win") => create_player::<UCTPolicy, WinLossEvaluator>(player_position, options),
        ("uct", "score") => create_player::<UCTPolicy, ScoreEvaluator>(player_position, options),
        ("uct", "nn") => create_player::<UCTPolicy, NeuralNetworkEvaluator>(player_position, options),
        ("partial-score", "static") => create_player::<PartiallyScoredUCTPolicy, StaticEvaluator>(player_position, options),
        ("partial-score", "win") => create_player::<PartiallyScoredUCTPolicy, WinLossEvaluator>(player_position, options),
        ("partial-score", "score") => create_player::<PartiallyScoredUCTPolicy, ScoreEvaluator>(player_position, options),
        ("partial-score", "nn") => create_player::<PartiallyScoredUCTPolicy, NeuralNetworkEvaluator>(player_position, options),
        ("score", "static") => create_player::<ScoredUCTPolicy, StaticEvaluator>(player_position, options),
        ("score", "win") => create_player::<ScoredUCTPolicy, WinLossEvaluator>(player_position, options),
        ("score", "score") => create_player::<ScoredUCTPolicy, ScoreEvaluator>(player_position, options),
        ("score", "nn") => create_player::<ScoredUCTPolicy, NeuralNetworkEvaluator>(player_position, options),
        _ => unreachable!(),
    };

    (Some(player), None)
}

fn parse_alphazero_player(name: &str, player_position: usize) -> Option<Box<dyn Player>> {
    if name == "alphazero" {
        return Some(Box::new(AlphaZeroPlayer::new(format!(
            "AlphaZero Player {player_position}"
        ))));
    }

    None
}
