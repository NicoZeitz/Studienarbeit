use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use patchwork_core::{ActionId, Patchwork};

use crate::mcts::{Node, NodeId};

/// A simple allocator for nodes in the search tree.
pub struct AreaAllocator {
    /// The nodes in the search tree.
    pub nodes: boxcar::Vec<RwLock<Node>>,
}

impl AreaAllocator {
    /// Create a new [`AreaAllocator`] with no nodes.
    pub fn new() -> Self {
        Self {
            nodes: boxcar::Vec::new(),
        }
    }

    /// Get the number of nodes in the allocator.
    ///
    /// # Returns
    ///
    /// The number of nodes in the allocator.
    pub fn size(&self) -> usize {
        self.nodes.count()
    }

    /// Create a new node in the search tree.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state of the new node.
    /// * `parent` - The parent node of the new node.
    /// * `action_taken` - The action taken to reach the new node.
    /// * `prior` - The prior belief of the new node.
    ///
    /// # Returns
    ///
    /// The ID of the new node.
    pub fn new_node(
        &self,
        game: Patchwork,
        parent: Option<NodeId>,
        action_taken: Option<ActionId>,
        prior: Option<f32>,
    ) -> NodeId {
        let dummy_node_id = NodeId(0);

        let node_id = NodeId(
            self.nodes
                .push(RwLock::new(Node::new(dummy_node_id, game, parent, action_taken, prior))),
        );

        self.nodes[node_id.0].write().unwrap().id = node_id;

        node_id
    }

    /// Get the node with the given ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to get.
    ///
    /// # Returns
    ///
    /// The reference to the node with the given ID.
    pub fn get_node_read(&self, node_id: NodeId) -> RwLockReadGuard<'_, Node> {
        self.nodes[node_id.0].read().unwrap()
    }

    /// Get a mutable reference to the node with the given ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to get.
    ///
    /// # Returns
    ///
    /// The mutable reference to the node with the given ID.
    pub fn get_node_write(&self, node_id: NodeId) -> RwLockWriteGuard<'_, Node> {
        self.nodes[node_id.0].write().unwrap()
    }
}

impl Default for AreaAllocator {
    fn default() -> Self {
        Self::new()
    }
}
