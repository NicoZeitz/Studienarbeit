use crate::{node_id::NodeId, AreaAllocator};

pub struct Tree {
    /// The root node of the tree.
    pub root: NodeId,
    /// The allocator holding all nodes of the tree.
    pub allocator: AreaAllocator,
}

impl Tree {
    /// Create a new [`LastTree`].
    ///
    /// # Arguments
    ///
    /// * `root` - The root node of the tree.
    /// * `allocator` - The allocator holding all nodes of the tree.
    pub const fn new(root: NodeId, allocator: AreaAllocator) -> Self {
        Self { root, allocator }
    }
}
