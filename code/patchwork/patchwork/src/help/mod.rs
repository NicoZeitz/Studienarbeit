use std::env;

use titlecase::titlecase;

pub fn print_welcome() {
    let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>().join(" & ");
    let build_profile = if cfg!(debug_assertions) { "(Debug Build) " } else { "" };
    println!(
        "{} {} {}| {} ",
        titlecase(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        build_profile,
        authors,
    );
}

pub fn print_help() {
    print_welcome();
    println!();
    println!("Usage: {} [Cmd] [Options]", env!("CARGO_BIN_NAME"));
    println!();
    println!("Commands:");
    println!("    help, h   Print this help message");
    println!("    exit      Exit the application");
    println!("    clear     Clear the screen");
    #[cfg(debug_assertions)]
    println!("    debug     Print different debug information useful for debugging. Only works in debug builds.");
    println!("    console   Start an interactive console game of patchwork");
    println!("                -1,   --player-1      The name of the first player");
    println!("                -2,   --player-2      The name of the second player");
    println!("                --l1, --logging-1     The logging configuration of the first player");
    println!("                --l2, --logging-2     The logging configuration of the second player");
    println!("                -s,   --seed          The seed for the initial state");
    println!("    compare   Compare different patchwork ai's against each other");
    println!("                -1,   --player-1      The name of the first player");
    println!("                -2,   --player-2      The name of the second player");
    println!("                --l1, --logging-1     The logging configuration of the first player");
    println!("                --l2, --logging-2     The logging configuration of the second player");
    println!("                -g,   --games         The number of games the players should be compared in");
    println!("                -u,   --update        How often the comparison information should be updated (in ms)");
    println!("                -p,   --parallel      How many games to play in parallel");
    println!("    upi       Start Universal Patchwork Interface (UPI) in console mode");
    println!("                -n,   --no-prompt     Do not print the prompt");
    println!("    server    Start the patchwork game server");
    println!("                -p,  --port           The port the server should start on. Default 3000");
    println!("                --public             If present listens on 0.0.0.0 else on 127.0.0.1");
}

#[cfg(debug_assertions)]
pub fn print_debug() {
    print_welcome();
    println!("pwd:  {}", std::env::current_dir().unwrap().display());
    println!("cwd:  {}", std::env::current_exe().unwrap().display());
    println!("args: {:?}", std::env::args().collect::<Vec<_>>());
}
