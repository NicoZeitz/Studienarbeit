use std::io::Write;

use patchwork_core::{ActionId, Patchwork};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct Game {
    pub turns: Vec<GameTurn>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct GameTurn {
    pub state: Patchwork,
    pub action: Option<ActionId>,
}

#[allow(clippy::never_loop)]
fn get_game_statistics(path: &std::path::PathBuf) {
    let mut lengths = vec![];
    let mut min_length = std::usize::MAX;
    let mut max_length = std::usize::MIN;
    let mut sum_length = 0;
    let mut games = 0;

    let mut last_round = std::time::Instant::now();

    for game in GameIterator::new(path, true) {
        if last_round.elapsed().as_nanos() >= 1_000_000_000 {
            println!("Time since last round: {:?}", last_round.elapsed());
        }

        let length = game.turns.iter().filter(|turn| turn.action.is_some()).count();

        lengths.push(length);
        min_length = std::cmp::min(min_length, length);
        max_length = std::cmp::max(max_length, length);
        sum_length += length;
        games += 1;

        let avg_length = sum_length as f64 / games as f64;

        // print!("\x1b[4A\r");
        // println!("================= Game {} =================", games);
        // println!("Current Minimum game length: {}", min_length);
        // println!("Current Maximum game length: {}", max_length);
        // println!("Current Average game length: {}", avg_length);
        std::io::stdout().flush().unwrap();

        last_round = std::time::Instant::now();
    }

    let avg_length = sum_length as f64 / games as f64;

    println!("================= Ending statistics {} =================", games);
    println!("Getting game statistics from {:?}", path);
    println!("Minimum game length: {}", min_length);
    println!("Maximum game length: {}", max_length);
    println!("Average game length: {}", avg_length);
}

fn get_action_statistics(path: &std::path::PathBuf) {
    GameIterator::new(path, true);

    println!("Getting action statistics from {:?}", path);
}

struct GameIterator {
    multithreaded: bool,
    dir: std::fs::ReadDir,
    game_chucks: Vec<Game>,
}

impl GameIterator {
    fn new(path: &std::path::PathBuf, multithreaded: bool) -> Self {
        let dir: std::fs::ReadDir = std::fs::read_dir(path).unwrap();
        Self {
            dir,
            game_chucks: vec![],
            multithreaded,
        }
    }
}

impl Iterator for GameIterator {
    type Item = Game;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.game_chucks.is_empty() {
                return self.game_chucks.pop();
            }

            if self.multithreaded {
                std::thread::scope(|s| {
                    let mut handles = vec![];
                    self.game_chucks = vec![];

                    for _ in 0..(std::thread::available_parallelism().unwrap().get() - 1) {
                        let Some(entry) = self.dir.next() else {
                            break;
                        };

                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_dir() {
                            continue;
                        }

                        let file = std::fs::File::open(path).unwrap();
                        handles.push(s.spawn(move || {
                            let game_chucks: Vec<Game> = bincode::deserialize_from(file).unwrap();
                            game_chucks
                        }));
                    }

                    for handle in handles {
                        self.game_chucks.extend(handle.join().unwrap());
                    }
                });
            } else {
                let Some(entry) = self.dir.next() else {
                    return None;
                };

                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }

                let file = std::fs::File::open(path).unwrap();
                let game_chucks: Vec<Game> = bincode::deserialize_from(file).unwrap();
                self.game_chucks = game_chucks;
            }
        }
    }
}

fn main() {
    let cmd = clap::Command::new("empirical-measurement")
        .bin_name("empirical-measurement")
        .about("Gets different empirical measurements from a set of recorded games")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("game")
                .about("Gets the minimum, maximum and average number of plys in a set of recorded games")
                .arg(
                    clap::Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("The path to the directory where the recorded games are")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                ),
        )
        .subcommand(
            clap::Command::new("action")
            .about("Gets the minimum, maximum and average number of available actions in a set of recorded games (special patch placement actions excluded)")
                .arg(
                    clap::Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("The path to the directory where the recorded games are")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                ),
        );

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("game", matches)) => get_game_statistics(matches.get_one::<std::path::PathBuf>("path").unwrap()),
        Some(("action", matches)) => get_action_statistics(matches.get_one::<std::path::PathBuf>("path").unwrap()),
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
