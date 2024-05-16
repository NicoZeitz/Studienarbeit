use std::{
    io::Write,
    sync::{atomic::AtomicBool, mpsc::channel, Arc},
};

use anyhow::Error;
use clap::Parser;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};
use upi::start_upi;

use crate::{CTRL_C_MESSAGE, CTRL_D_MESSAGE};

#[derive(Debug, Parser, Default)]
#[command(no_binary_name(true))]
struct CmdArgs {
    #[arg(long = "no-prompt", short = 'n', default_value = "false")]
    no_prompt: bool,
}

pub fn handle_upi(rl: &mut Editor<(), FileHistory>, args: Vec<String>) -> anyhow::Result<()> {
    let args = CmdArgs::parse_from(args);
    let prompt = if args.no_prompt { "" } else { "upi> " };

    let (sender, upi_receiver) = channel();
    let (upi_sender, receiver) = channel();

    std::thread::scope(|s| {
        let close_flag = Arc::new(AtomicBool::new(false));
        // upi thread
        let close_flag_clone = close_flag.clone();
        s.spawn(move || {
            let _ = start_upi(upi_receiver, upi_sender);
            close_flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        // upi incoming message thread
        let close_flag_clone = close_flag.clone();
        s.spawn(move || {
            loop {
                if close_flag_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                match receiver.recv() {
                    Ok(msg) => {
                        print!("{msg}");
                        std::io::stdout().flush().unwrap();
                    }
                    Err(_) => break, // channel closed, exit loop
                }
            }
        });

        loop {
            if close_flag.load(std::sync::atomic::Ordering::SeqCst) {
                return Ok(());
            }

            let readline = rl.readline(prompt);
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if sender.send(line.to_string()).is_err() {
                        // channel closed, exit loop
                        close_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                Err(ReadlineError::Interrupted) => {
                    close_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    return Err(Error::msg(CTRL_C_MESSAGE));
                }
                Err(ReadlineError::Eof) => {
                    close_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    return Err(Error::msg(CTRL_D_MESSAGE));
                }
                Err(err) => {
                    close_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    return Err(Error::from(err));
                }
            }
        }
    })
}
