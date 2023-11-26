pub use game::Game;
pub use patchwork_core::{
    Action, ActionPayload, Patch, PatchTransformation, Patchwork, PlayerState, QuiltBoard,
    Termination, TerminationType, TimeBoard,
};

pub mod player {
    pub use game::Player;
    pub use patchwork_alphazero_player::AlphaZeroPlayer;
    pub use patchwork_greedy_player::GreedyPlayer;
    pub use patchwork_human_player::HumanPlayer;
    pub use patchwork_mcts_player::MCTSPlayer;
    pub use patchwork_minimax_player::{MinimaxOptions, MinimaxPlayer};
    pub use patchwork_negamax_player::NegamaxPlayer;
    pub use patchwork_random_player::{RandomOptions, RandomPlayer};
}

pub mod prelude {
    pub use super::player::*;
    pub use game::Game;
    pub use patchwork_core::{Action, Patch, Patchwork, Termination, TerminationType};
}
