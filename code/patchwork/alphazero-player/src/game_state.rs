use patchwork_core::Patchwork;

use crate::mcts::{AreaAllocator, NodeId};

pub struct GameState {
    pub game: Patchwork,
    pub allocator: AreaAllocator,
    pub root: Option<NodeId>,
}

impl GameState {
    pub fn new(game: Patchwork) -> Self {
        Self {
            game,
            allocator: AreaAllocator::new(),
            root: None,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            game: Patchwork::get_initial_state(None),
            allocator: AreaAllocator::new(),
            root: None,
        }
    }
}
