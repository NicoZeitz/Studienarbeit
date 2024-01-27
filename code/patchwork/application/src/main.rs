mod console;
mod exit;
mod help;
mod server;
mod upi;

use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};

use crate::exit::{handle_exit, handle_exit_with_error};
use crate::help::{print_cmd_help, print_debug, print_repl_help, print_welcome};
use crate::server::{start_server_from_cmd, start_server_from_repl};

const CTRL_C_MESSAGE: &str = "Received CTRL-C command. Exiting application...";
const CTRL_D_MESSAGE: &str = "Received CTRL-D command. Exiting application...";
fn main() -> anyhow::Result<()> {
    if std::env::args().len() > 1 {
        return handle_args();
    }

    print_welcome();
    println!("{} Type \"help\" for more information.", env!("CARGO_PKG_DESCRIPTION"));

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if let Err(error) = match_line(&line, &mut rl) {
                    handle_exit_with_error(error);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", CTRL_C_MESSAGE);
                handle_exit();
            }
            Err(ReadlineError::Eof) => {
                println!("{}", CTRL_D_MESSAGE);
                handle_exit();
            }
            Err(err) => {
                handle_exit_with_error(err.into());
            }
        }
    }
}

fn handle_args() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap();
    let args = std::env::args().skip(2).collect::<Vec<_>>();

    match cmd.as_str() {
        "help" | "h" => print_cmd_help(),
        "exit" | "quit" | "q" => handle_exit(),
        #[cfg(debug_assertions)]
        "echo" => {
            let mut output = String::new();
            for arg in args.iter().skip(1) {
                output.push_str(arg);
                output.push(' ');
            }
            println!("{}", output);
        }
        #[cfg(debug_assertions)]
        "debug" => print_debug(),
        "upi" => {
            unimplemented!("TODO: UPI is not yet implemented.");
        }
        "console" => {
            unimplemented!("TODO: Console mode is not yet implemented.");
        }
        "server" => start_server_from_cmd(args)?,
        _ => print_cmd_help(),
    }

    Ok(())
}

fn match_line(line: &str, rl: &mut Editor<(), FileHistory>) -> anyhow::Result<()> {
    let mut args = line.split_whitespace();
    match args.next() {
        Some("help" | "h") => print_repl_help(),
        Some("exit" | "quit" | "q") => handle_exit(),
        Some("clear") => rl.clear_screen()?,
        #[cfg(debug_assertions)]
        Some("echo") => {
            let mut output = String::new();
            for arg in args {
                output.push_str(arg);
                output.push(' ');
            }
            println!("{}", output);
        }
        #[cfg(debug_assertions)]
        Some("debug") => print_debug(),
        Some("upi") => {
            println!("Starting Universal Patchwork Interface (UPI) in console mode...");
            rl.clear_screen()?;
            unimplemented!("TODO: UPI is not yet implemented.");
        }
        Some("console") => {
            println!("Starting an interactive console game of patchwork...");
            rl.clear_screen()?;
            unimplemented!("TODO: Console mode is not yet implemented.");
        }
        Some("server") => start_server_from_repl(args.collect::<Vec<_>>(), rl)?,
        _ => {
            println!("Unknown command \"{}\". Type \"help\" for more information.", line);
        }
    }

    Ok(())
}
