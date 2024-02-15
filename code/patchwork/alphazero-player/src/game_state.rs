use patchwork_core::Patchwork;

use crate::mcts::NodeId;

pub struct GameState {
    pub game: Patchwork,
    pub memory: Vec<usize>,
    pub root: Option<NodeId>, // NODE
}

impl GameState {
    pub fn new(game: Patchwork) -> Self {
        Self {
            game,
            memory: vec![],
            root: None,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            game: Patchwork::get_initial_state(None),
            memory: vec![],
            root: None,
        }
    }
}
