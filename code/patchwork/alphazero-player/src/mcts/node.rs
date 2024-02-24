use patchwork_core::{ActionId, Patchwork, TreePolicyNode};

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
    pub neutral_max_score: f32,
    // The minimum neutral score of all the nodes in the subtree rooted at this node.
    pub neutral_min_score: f32,
    // The sum of the neutral scores of all the nodes in the subtree rooted at this node.
    pub neutral_score_sum: f64,
    // The number of times this node has been won by player 1 (wins from a neutral perspective)
    pub neutral_wins: i32,
    // The number of times this node has been visited.
    pub visit_count: usize,
    // The virtual loss of the node.
    pub virtual_loss: i32,
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
            neutral_max_score: f32::NEG_INFINITY,
            neutral_min_score: f32::INFINITY,
            neutral_wins: 0,
            neutral_score_sum: 0.0,
            visit_count: 0,
            action_taken,
            prior: prior.unwrap_or(0.0),
            virtual_loss: 0,
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
        !self.children.is_empty()
    }

    /// Increments the visit count of the node.
    pub fn increment_virtual_loss(&mut self) {
        self.virtual_loss += 1;
    }

    /// Decrements the virtual loss of the node.
    pub fn decrement_virtual_loss(&mut self) {
        self.virtual_loss -= 1;
    }
}

impl TreePolicyNode for Node {
    type Player = bool;

    fn visit_count(&self) -> usize {
        self.visit_count
    }

    fn current_player(&self) -> Self::Player {
        self.state.is_player_1()
    }

    fn wins_for(&self, player: Self::Player) -> i32 {
        let wins = if player { self.neutral_wins } else { -self.neutral_wins };

        wins - self.virtual_loss
    }

    fn maximum_score_for(&self, player: Self::Player) -> f64 {
        // == -self.minimum_score_for(!player)
        if player {
            self.neutral_max_score as f64
        } else {
            -self.neutral_min_score as f64
        }
    }

    fn minimum_score_for(&self, player: Self::Player) -> f64 {
        // == -self.maximum_score_for(!player)
        if player {
            self.neutral_min_score as f64
        } else {
            -self.neutral_max_score as f64
        }
    }

    fn score_range(&self) -> f64 {
        (self.neutral_max_score - self.neutral_min_score) as f64
    }

    fn score_sum_for(&self, player: Self::Player) -> f64 {
        if player {
            self.neutral_score_sum
        } else {
            -self.neutral_score_sum
        }
    }

    fn prior_value(&self) -> f64 {
        self.prior as f64
    }
}
