use clap::Parser;
use rustyline::{history::FileHistory, Editor};
use server::start_server;

use crate::exit::handle_exit;

#[derive(Debug, Parser, Default)]
#[command(no_binary_name(true))]
struct CmdArgs {
    #[arg(long, short)]
    port: Option<u16>,
    #[arg(long, default_value_t = false)]
    public: bool,
}

pub fn start_server_from_cmd(args: Vec<String>) -> anyhow::Result<()> {
    let args = CmdArgs::parse_from(args);
    start_server(args.port, args.public)?;
    handle_exit();
}

pub fn start_server_from_repl(args: Vec<&'_ str>, rl: &mut Editor<(), FileHistory>) -> anyhow::Result<()> {
    let mut port = None;
    let mut public = false;

    for i in 0..args.len() {
        match args[i] {
            "public" => public = true,
            "port" => {
                if i + 1 < args.len() {
                    port = Some(args[i + 1].parse::<u16>()?);
                } else {
                    println!("Missing port number: server [port <number>] [public]");
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    rl.clear_screen()?;
    start_server(port, public)?;
    handle_exit();
}
