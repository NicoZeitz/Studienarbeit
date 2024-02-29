// https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use std::sync::mpsc::{Receiver, Sender};

/// Starts the UPI server
///
/// # Errors
///
/// This function will return an error if sending or receiving messages over the channels fails.
pub fn start_upi(message_receiver: Receiver<String>, message_sender: Sender<String>) -> anyhow::Result<()> {
    while let Ok(msg) = message_receiver.recv() {
        let msg = msg.trim().to_lowercase();
        let mut split_message = msg.split_whitespace();
        match split_message.next() {
            Some("upi") => {
                let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>().join(" & ");
                let message = format!("id name {}\nid author {}\nupiok\n", env!("CARGO_PKG_NAME"), authors);
                message_sender.send(message)?;
            }
            Some("isready") => {
                message_sender.send("readyok\n".to_string())?;
            }
            // debug [on|off]
            // setoption name [value]
            // ucinewgame
            // position [fen  | startpos ]  moves  ....
            // go
            // stop
            Some("quit") => {
                break;
            }
            _ => {
                message_sender.send("unknown command\n".to_string())?;
            }
        }
    }

    drop(message_receiver);
    drop(message_sender);

    Ok(())
}
