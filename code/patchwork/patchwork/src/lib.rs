pub use patchwork_core::{
    Action, ActionPayload, Game, Patch, PatchTransformation, Patchwork, PlayerState, QuiltBoard,
    Termination, TerminationType, TimeBoard,
};

pub mod player {
    pub use patchwork_core::Player;
    pub use patchwork_player::*;
}

pub mod prelude {
    pub use super::player::*;
    pub use patchwork_core::{Action, Game, Patch, Patchwork, Termination, TerminationType};
}
