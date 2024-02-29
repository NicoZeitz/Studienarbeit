use std::io::Write;

use anyhow::Error;
use patchwork_lib::player::Logging;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};

use super::{CTRL_C_MESSAGE, CTRL_D_MESSAGE};

pub fn interactive_get_logging(
    rl: &mut Editor<(), FileHistory>,
    player_position: usize,
    logging: Option<String>,
) -> anyhow::Result<Logging> {
    if let Some(logging) = logging {
        let Some(logging) = parse_logging(&logging) else {
            println!(
                "Invalid logging configuration {logging}. Available options: disabled, enabled, verbose, verbose-only"
            );
            std::io::stdout().flush()?;
            return Err(Error::msg(format!("Invalid logging argument: {logging}")));
        };

        return Ok(logging);
    }

    loop {
        let prompt = format!("Player {player_position} logging: ");
        let input = if player_position == 1 {
            rl.readline_with_initial(prompt.as_str(), ("Enabled", ""))
        } else {
            rl.readline_with_initial(prompt.as_str(), ("Disabled", ""))
        };

        match input {
            Ok(logging) => {
                if let Some(logging) = parse_logging(&logging) {
                    return Ok(logging);
                }

                println!(
                    "Invalid logging configuration {logging}. Available options: disabled, enabled, verbose, verbose-only"
                );
                std::io::stdout().flush()?;
            }
            Err(ReadlineError::Interrupted) => return Err(Error::msg(CTRL_C_MESSAGE)),
            Err(ReadlineError::Eof) => return Err(Error::msg(CTRL_D_MESSAGE)),
            Err(err) => return Err(Error::from(err)),
        }
    }
}

pub fn get_logging(logging: &str) -> anyhow::Result<Logging> {
    if let Some(logging) = parse_logging(logging) {
        return Ok(logging);
    }
    println!("Invalid logging configuration {logging}. Available options: disabled, enabled, verbose, verbose-only");
    std::io::stdout().flush()?;
    Err(Error::msg(format!("Invalid logging argument: {logging}")))
}

pub fn parse_logging(logging: &str) -> Option<Logging> {
    fn create_debug_writer() -> Box<dyn std::io::Write> {
        Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!(
                    "{}/logging_{}.txt",
                    std::env::current_dir().unwrap().to_str().unwrap(),
                    chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
                ))
                .unwrap(),
        )
    }

    match logging.to_ascii_lowercase().as_str() {
        "disabled" => Some(Logging::Disabled),
        "enabled" => Some(Logging::Enabled {
            progress_writer: Box::new(std::io::stdout()),
        }),
        "verbose" => Some(Logging::Verbose {
            progress_writer: Box::new(std::io::stdout()),
            debug_writer: create_debug_writer(),
        }),
        "verbose-only" => Some(Logging::VerboseOnly {
            debug_writer: create_debug_writer(),
        }),
        _ => None,
    }
}
