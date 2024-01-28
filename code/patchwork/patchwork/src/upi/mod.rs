use std::{io::Write, sync::mpsc::channel};

use anyhow::anyhow;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};
use upi::start_upi;

use crate::{
    exit::{handle_exit, handle_exit_with_error},
    CTRL_C_MESSAGE, CTRL_D_MESSAGE,
};

pub fn handle_upi(starting_cmd: impl Into<String>, rl: &mut Editor<(), FileHistory>) -> anyhow::Result<()> {
    let (sender, upi_receiver) = channel();
    let (upi_sender, receiver) = channel();

    let handle = std::thread::spawn(move || start_upi(upi_receiver, upi_sender));

    match sender.send(starting_cmd.into()).ok().and_then(|_| receiver.recv().ok()) {
        None => {} // channel closed
        Some(msg) => {
            print!("{}", msg);
            loop {
                let readline = rl.readline("upi> ");
                match readline {
                    Ok(line) => {
                        match sender.send(line) {
                            Ok(_) => {}
                            Err(_) => break, // channel closed, exit loop
                        };
                        match receiver.recv() {
                            Ok(msg) => {
                                print!("{}", msg);
                                std::io::stdout().flush().unwrap();
                            }
                            Err(_) => break, // channel closed, exit loop
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
    }

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!(format!("[handle_upi] Error joining thread"))),
    }?;

    handle_exit();
}
