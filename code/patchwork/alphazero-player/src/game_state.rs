use crate::mcts::{AreaAllocator, NodeId};

/// The game state that is shared between the search threads.
pub struct GameState {
    /// The allocator to use for the nodes.
    pub allocator: AreaAllocator,
    /// The root node of the search tree.
    pub root: NodeId,
}
