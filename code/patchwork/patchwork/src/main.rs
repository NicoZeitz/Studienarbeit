mod common;
mod compare;
mod console;
mod exit;
mod help;
mod server;
mod upi;

use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};

use crate::common::{CTRL_C_MESSAGE, CTRL_D_MESSAGE};
use crate::compare::handle_compare;
use crate::console::handle_console;
use crate::exit::{handle_exit, handle_exit_with_error};
#[cfg(debug_assertions)]
use crate::help::print_debug;
use crate::help::{print_help, print_welcome};
use crate::server::handle_server;
use crate::upi::handle_upi;

fn main() {
    if std::env::args().len() > 1 {
        match handle_args() {
            Ok(()) => handle_exit(0),
            Err(err) => handle_exit_with_error(err),
        }
    }

    // enter REPL mode
    print_welcome();
    println!("{} Type \"help\" for more information.", env!("CARGO_PKG_DESCRIPTION"));

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(err) => handle_exit_with_error(err.into()),
    };
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if let Err(error) = match_line(&line, &mut rl) {
                    handle_exit_with_error(error);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", CTRL_C_MESSAGE);
                handle_exit(0);
            }
            Err(ReadlineError::Eof) => {
                println!("{}", CTRL_D_MESSAGE);
                handle_exit(0);
            }
            Err(err) => handle_exit_with_error(err.into()),
        }
    }
}

fn handle_args() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap();
    let args = std::env::args().skip(2).collect::<Vec<_>>();

    let mut rl = Editor::<(), FileHistory>::new()?;

    match cmd.as_str() {
        "help" | "h" => print_help(),
        "exit" | "quit" | "q" => handle_exit(0),
        #[cfg(debug_assertions)]
        "debug" => print_debug(),
        "upi" => handle_upi(&mut rl, args)?,
        "console" => handle_console(&mut rl, args)?,
        "compare" => handle_compare(&mut rl, args)?,
        "server" => handle_server(&mut rl, args)?,
        _ => {
            print_help();
            handle_exit(1);
        }
    };
    Ok(())
}

fn match_line(line: &str, rl: &mut Editor<(), FileHistory>) -> anyhow::Result<()> {
    let mut input = line.split_whitespace();
    let cmd = input.next();
    let args = input.map(|s| s.to_string()).collect::<Vec<_>>();
    match cmd {
        Some("help" | "h") => print_help(),
        Some("exit" | "quit" | "q") => handle_exit(0),
        Some("clear") => rl.clear_screen()?,
        #[cfg(debug_assertions)]
        Some("debug") => print_debug(),
        Some("upi") => {
            if let Err(err) = handle_upi(rl, args) {
                println!("UPI exited with error: {}", err);
            }
        }
        Some("console") => {
            if let Err(err) = handle_console(rl, args) {
                println!("Console exited with error: {}", err);
            }
        }
        Some("compare") => {
            if let Err(err) = handle_compare(rl, args) {
                println!("Compare exited with error: {}", err);
            }
        }
        Some("server") => {
            if let Err(err) = handle_server(rl, args) {
                println!("Server exited with error: {}", err);
            }
        }
        _ => println!("Unknown command \"{}\". Type \"help\" for more information.", line),
    }

    Ok(())
}
