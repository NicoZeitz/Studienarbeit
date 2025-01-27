mod logging;
mod player;

pub const CTRL_C_MESSAGE: &str = "Received CTRL-C command.";
pub const CTRL_D_MESSAGE: &str = "Received CTRL-D command.";

pub use logging::*;
pub use player::*;
