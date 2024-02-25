use std::collections::VecDeque;

use patchwork_core::{ActionId, Patchwork};

use crate::{Node, NodeId};

/// A simple allocator for nodes in the search tree.
pub struct AreaAllocator {
    /// The nodes in the search tree.
    pub nodes: Vec<Node>,
}

impl AreaAllocator {
    /// Create a new [`AreaAllocator`] with no nodes.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Clear all nodes from the allocator.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Get the number of nodes in the allocator.
    ///
    /// # Returns
    ///
    /// The number of nodes in the allocator.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    /// Create a new node in the search tree.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state of the new node.
    /// * `parent` - The parent node of the new node.
    /// * `action_taken` - The action taken to reach the new node.
    ///
    /// # Returns
    ///
    /// The ID of the new node.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn new_node(&mut self, game: Patchwork, parent: Option<NodeId>, action_taken: Option<ActionId>) -> NodeId {
        let next_node_id = self.nodes.len();
        let node_id = NodeId(next_node_id);

        self.nodes.push(Node::new(node_id, game, parent, action_taken));

        if let Some(parent_id) = parent {
            self.nodes[parent_id.0].children.push(node_id);
        }

        node_id
    }

    /// Reallocate the nodes in the search tree to a new root node.
    ///
    /// # Arguments
    ///
    /// * `root` - The new root node.
    ///
    /// # Returns
    ///
    /// The node id of the new root node.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `𝑛` is the number of nodes in the current search tree
    /// and `𝑚` is the number of children of each node.
    pub fn realloc_to_new_root(&mut self, root: NodeId) -> NodeId {
        let mut to_keep = vec![false; self.nodes.len()];

        let mut queue = VecDeque::new();
        queue.push_back(root);
        while let Some(node_id) = queue.pop_front() {
            to_keep[node_id.0] = true;

            let node = &self.nodes[node_id.0];
            queue.extend(node.children.iter());
        }

        self.nodes.retain(|node| to_keep[node.id.0]);

        let mut id_map = vec![0; to_keep.len()];
        for (new_id, node) in self.nodes.iter().enumerate() {
            id_map[node.id.0] = new_id;
        }

        for node in &mut self.nodes {
            node.id.0 = id_map[node.id.0];
            node.parent = node.parent.map(|id| NodeId(id_map[id.0]));
            for child in &mut node.children {
                child.0 = id_map[child.0];
            }
        }

        // Reset parent and action take of new root
        self.nodes[0].parent = None;
        self.nodes[0].action_taken = None;

        NodeId(0)
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
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id.0]
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
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node {
        &mut self.nodes[node_id.0]
    }
}

impl Default for AreaAllocator {
    fn default() -> Self {
        Self::new()
    }
}
