use std::fmt;

use patchwork_core::{ActionId, Patchwork, TreePolicyNode};
use rand::seq::SliceRandom;

use crate::{node_id::NodeId, AreaAllocator};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Node {
    /// The unique identifier of the node.
    pub id: NodeId,
    /// The state of the game at this node.
    pub state: Patchwork,
    /// The parent node. None if this is the root node.
    pub parent: Option<NodeId>,
    /// The action that was taken to get to this node. None if this is the root node.
    pub action_taken: Option<ActionId>,
    /// The children nodes.
    pub children: Vec<NodeId>,
    /// The actions that can still be taken from this node
    pub expandable_actions: Vec<ActionId>,
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
    ///
    /// # Returns
    ///
    /// The new node.
    pub fn new(node_id: NodeId, state: Patchwork, parent: Option<NodeId>, action_taken: Option<ActionId>) -> Self {
        let mut expandable_actions: Vec<ActionId> = state.get_valid_actions().into_iter().collect();
        expandable_actions.shuffle(&mut rand::thread_rng());

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
            expandable_actions,
        }
    }

    /// Whether the node is fully expanded.
    ///
    /// A node is fully expanded if all of its children have been created or if
    /// the game state of the node is a terminal state.
    ///
    /// # Returns
    ///
    /// `true` if the node is fully expanded, `false` otherwise.
    pub fn is_fully_expanded(&self) -> bool {
        self.state.is_terminated() || self.expandable_actions.is_empty()
    }

    /// Whether the node is a terminal node.
    ///
    /// A node is terminal if the game state of the node is a terminal state.
    ///
    /// # Returns
    ///
    /// `true` if the node is terminal, `false` otherwise.
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminated()
    }
}

impl TreePolicyNode for Node {
    type Player = bool;

    fn visit_count(&self) -> i32 {
        self.visit_count
    }

    fn current_player(&self) -> Self::Player {
        self.state.is_player_1()
    }

    fn wins_for(&self, player: Self::Player) -> i32 {
        if player {
            self.neutral_wins
        } else {
            -self.neutral_wins
        }
    }

    fn maximum_score_for(&self, player: Self::Player) -> i32 {
        // == -self.minimum_score_for(!player)
        if player {
            self.neutral_max_score
        } else {
            -self.neutral_min_score
        }
    }

    fn minimum_score_for(&self, player: Self::Player) -> i32 {
        // == -self.maximum_score_for(!player)
        if player {
            self.neutral_min_score
        } else {
            -self.neutral_max_score
        }
    }

    fn score_range(&self) -> i32 {
        self.neutral_max_score - self.neutral_min_score
    }

    fn score_sum_for(&self, player: Self::Player) -> i64 {
        if player {
            self.neutral_score_sum
        } else {
            -self.neutral_score_sum
        }
    }
}

pub struct NodeDebug<'a> {
    pub node: &'a Node,
    pub allocator: &'a AreaAllocator,
}

impl fmt::Debug for NodeDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let wins_from_parent = if let Some(parent_id) = self.node.parent {
            let parent = self.allocator.get_node(parent_id);
            if parent.state.is_player_1() {
                self.node.neutral_wins
            } else {
                -self.node.neutral_wins
            }
        } else {
            0
        };

        f.debug_struct("Node")
            // .field("state", &self.state)
            .field("visit_count", &self.node.visit_count)
            .field("wins_from_parent", &wins_from_parent)
            .field("neutral_wins", &self.node.neutral_wins)
            .field("neutral_max_score", &self.node.neutral_max_score)
            .field("neutral_min_score", &self.node.neutral_min_score)
            .field("neutral_score_sum", &self.node.neutral_score_sum)
            .field("action_taken", &self.node.action_taken)
            .field("parent", &self.node.parent)
            .field("children", &self.node.children.len())
            .field("expandable_actions", &self.node.expandable_actions.len())
            .finish()
    }
}
