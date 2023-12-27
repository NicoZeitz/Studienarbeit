mod game_loop;

use clap::Parser;

use game_loop::GameLoop;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'c', long = "compare", help = "Runs a comparison with the given players")]
    compare: Option<usize>,

    #[arg(
        short = 'u',
        long = "update",
        help = "How often to update (in ms; only in comparison)"
    )]
    update: Option<usize>,

    #[arg(short = 'p', long = "par", help = "How many cores to use (only in comparison)")]
    parallelization: Option<usize>,

    #[arg(
        short = '1',
        long = "player-1",
        help = "The player that plays first",
        default_value = "greedy"
    )]
    player_1: String,

    #[arg(
        short = '2',
        long = "player-2",
        help = "The player that plays second",
        default_value = "pvs"
    )]
    player_2: String,
}

pub fn main() {
    let args = Args::parse();

    if args.compare.is_some() {
        GameLoop::compare(
            args.compare.unwrap(),
            &args.player_1,
            &args.player_2,
            args.update,
            args.parallelization,
        );
    } else {
        GameLoop::run(&args.player_1, &args.player_2);
    }
}
