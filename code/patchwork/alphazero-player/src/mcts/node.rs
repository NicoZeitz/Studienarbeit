use patchwork_core::{ActionId, Patchwork};

use crate::mcts::NodeId;

#[derive(Clone, PartialEq)]
pub struct Node {
    /// The unique identifier of the node.
    pub id: NodeId,
    /// The state of the game at this node.
    pub state: Patchwork,
    /// The parent node. None if this is the root node.
    pub parent: Option<NodeId>,
    /// The action that was taken to get to this node. None if this is the root node.
    pub action_taken: Option<ActionId>,
    /// The prior belief of the node.
    pub prior: f32,
    /// The children nodes.
    pub children: Vec<NodeId>,
    /// The maximum neutral score of all the nodes in the subtree rooted at this node.
    pub neutral_max_score: i32,
    // The minimum neutral score of all the nodes in the subtree rooted at this node.
    pub neutral_min_score: i32,
    // The sum of the neutral scores of all the nodes in the subtree rooted at this node.
    pub neutral_score_sum: i64,
    // The number of times this node has been won by player 1 (wins from a neutral perspective)
    pub neutral_wins: i32,
    // The number of times this node has been visited.
    pub visit_count: i32,
}

impl Node {
    /// Creates a new node with the given game state, parent node and action taken to get to this node.
    ///
    /// # Arguments
    ///
    /// * `state` - The game state of the node.
    /// * `parent` - The parent node. None if this is the root node.
    /// * `action_taken` - The action that was taken to get to this node. None if this is the root node.
    /// * `prior` - The prior belief of the node.
    ///
    /// # Returns
    ///
    /// The new node.
    pub fn new(
        node_id: NodeId,
        state: Patchwork,
        parent: Option<NodeId>,
        action_taken: Option<ActionId>,
        prior: Option<f32>,
    ) -> Self {
        Self {
            id: node_id,
            state,
            parent,
            children: vec![],
            neutral_max_score: i32::MIN,
            neutral_min_score: i32::MAX,
            neutral_wins: 0,
            neutral_score_sum: 0,
            visit_count: 0,
            action_taken,
            prior: prior.unwrap_or(0.0),
        }
    }

    /// Whether the node is fully expanded.
    ///
    /// A node is fully expanded if all of its children have been created.
    ///
    /// # Returns
    ///
    /// `true` if the node is fully expanded, `false` otherwise.
    pub fn is_fully_expanded(&self) -> bool {
        self.children.len() > 0
    }
}
