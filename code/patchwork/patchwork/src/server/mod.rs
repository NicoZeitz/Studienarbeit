use clap::Parser;
use rustyline::{history::FileHistory, Editor};
use server::start_server;

#[derive(Debug, Parser, Default)]
#[command(no_binary_name(true))]
struct CmdArgs {
    #[arg(long, short)]
    port: Option<u16>,
    #[arg(long, default_value_t = false)]
    public: bool,
}

pub fn handle_server(rl: &mut Editor<(), FileHistory>, args: Vec<String>) -> anyhow::Result<()> {
    let args = CmdArgs::parse_from(args);

    rl.clear_screen()?;
    start_server(args.port, args.public)?;
    Ok(())
}
