use std::io::Write;

use anyhow::Error;
use patchwork_lib::player::Diagnostics;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};

use super::{CTRL_C_MESSAGE, CTRL_D_MESSAGE};

pub fn interactive_get_diagnostics(
    rl: &mut Editor<(), FileHistory>,
    player_position: usize,
    diagnostics: Option<String>,
) -> anyhow::Result<Diagnostics> {
    if let Some(diagnostics) = diagnostics {
        let Some(diagnostic) = parse_diagnostics(&diagnostics) else {
            println!(
                "Invalid diagnostics {}. Available options: disabled, enabled, verbose, verbose-only",
                diagnostics
            );
            std::io::stdout().flush()?;
            return Err(Error::msg(format!("Invalid diagnostics argument: {}", diagnostics)));
        };

        return Ok(diagnostic);
    }

    loop {
        match rl.readline(format!("Player {} diagnostics: ", player_position).as_str()) {
            Ok(diagnostics) => {
                if let Some(diagnostic) = parse_diagnostics(&diagnostics) {
                    return Ok(diagnostic);
                }

                println!(
                    "Invalid diagnostics {}. Available options: disabled, enabled, verbose, verbose-only",
                    diagnostics
                );
                std::io::stdout().flush()?;
            }
            Err(ReadlineError::Interrupted) => return Err(Error::msg(CTRL_C_MESSAGE)),
            Err(ReadlineError::Eof) => return Err(Error::msg(CTRL_D_MESSAGE)),
            Err(err) => return Err(Error::from(err)),
        }
    }
}

pub fn parse_diagnostics(diagnostics: &str) -> Option<Diagnostics> {
    fn create_debug_writer() -> Box<dyn std::io::Write> {
        Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!(
                    "{}/diagnostics_{}.log",
                    std::env::current_dir().unwrap().to_str().unwrap(),
                    chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
                ))
                .unwrap(),
        )
    }

    match diagnostics.to_ascii_lowercase().as_str() {
        "disabled" => Some(Diagnostics::Disabled),
        "enabled" => Some(Diagnostics::Enabled {
            progress_writer: Box::new(std::io::stdout()),
        }),
        "verbose" => Some(Diagnostics::Verbose {
            progress_writer: Box::new(std::io::stdout()),
            debug_writer: create_debug_writer(),
        }),
        "verbose-only" => Some(Diagnostics::VerboseOnly {
            debug_writer: create_debug_writer(),
        }),
        _ => None,
    }
}
