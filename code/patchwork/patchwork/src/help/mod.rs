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

pub fn print_repl_help() {
    print_welcome();
    println!("Commands:");
    println!("    help, h   - Print this help message");
    println!("    exit      - Exit the application");
    println!("    clear     - Clear the screen");
    #[cfg(debug_assertions)]
    println!("    echo      - Print the arguments");
    #[cfg(debug_assertions)]
    println!("    debug     - Print different debug information useful for debugging. Only works in debug builds.");
    println!("    upi       - Start Universal Patchwork Interface (UPI) in console mode");
    println!("    console   - Start an interactive console game of patchwork");
    println!("    server    - Start the patchwork game server");
    println!("                port <number>  - the port the server should start on. Default 3000");
    println!("                public         - if present listens on 0.0.0.0 else on 127.0.0.1");
}

pub fn print_cmd_help() {
    print_welcome();
    println!("Usage: {} [Cmd] [Options]", env!("CARGO_BIN_NAME"));
    println!("Commands:");
    println!("    help, h   - Print this help message");
    println!("    exit      - Exit the application");
    println!("    clear     - Clear the screen");
    #[cfg(debug_assertions)]
    println!("    echo      - Print the arguments");
    #[cfg(debug_assertions)]
    println!("    debug     - Print different debug information useful for debugging. Only works in debug builds.");
    println!("    upi       - Start Universal Patchwork Interface (UPI) in console mode");
    println!("    console   - Start an interactive console game of patchwork");
    println!("    server    - Start the patchwork game server");
    println!("                --port <number>  - the port the server should start on. Default 3000");
    println!("                --public         - if present listens on 0.0.0.0 else on 127.0.0.1");
}

#[cfg(debug_assertions)]
pub fn print_debug() {
    print_welcome();
    println!("pwd:  {}", std::env::current_dir().unwrap().display());
    println!("cwd:  {}", std::env::current_exe().unwrap().display());
    println!("args: {:?}", std::env::args().collect::<Vec<_>>());
}
