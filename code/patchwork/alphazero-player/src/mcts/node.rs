use std::sync::atomic::{AtomicI32, Ordering};

use patchwork_core::{ActionId, Patchwork, PatchworkError, TreePolicy, TreePolicyNode};

use crate::mcts::{AreaAllocator, NodeId};

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
    pub virtual_loss: AtomicI32,
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
            virtual_loss: AtomicI32::new(0),
        }
    }

    /// Whether the node is fully expanded.
    ///
    /// A node is fully expanded if all of its children have been created.
    ///
    /// # Returns
    ///
    /// `true` if the node is fully expanded, `false` otherwise.
    #[inline]
    pub fn is_fully_expanded(&self) -> bool {
        !self.children.is_empty()
    }

    /// Increments the visit count of the node.
    #[inline]
    pub fn increment_virtual_loss(&self) {
        self.virtual_loss.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the virtual loss of the node.
    #[inline]
    pub fn decrement_virtual_loss(&self) {
        self.virtual_loss.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increments the virtual loss of the node by the given amount.
    #[inline]
    pub fn increment_virtual_loss_by(&self, amount: i32) {
        self.virtual_loss.fetch_add(amount, Ordering::Relaxed);
    }

    /// Decrements the virtual loss of the node by the given amount.
    #[inline]
    pub fn decrement_virtual_loss_by(&self, amount: i32) {
        self.virtual_loss.fetch_sub(amount, Ordering::Relaxed);
    }
}

/// Implementation of the methods for the Monte Carlo Tree Search (MCTS) algorithm.
impl Node {
    /// Expands the given node by adding all possible child nodes. For each of the given actions a new child node is
    /// created with the given probability. If the node is already fully expanded, nothing happens.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to expand.
    /// * `policies` - The policy for each action.
    /// * `corresponding_actions` - The actions that correspond to the policies.
    /// * `allocator` - The allocator to use for the expansion.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the node was expanded successfully, `Err(PatchworkError)` otherwise.
    pub fn expand(
        node_id: NodeId,
        policies: &[f32],
        corresponding_actions: &[ActionId],
        allocator: &AreaAllocator,
    ) -> Result<(), PatchworkError> {
        if allocator.get_node_read(node_id).is_fully_expanded() {
            return Ok(());
        }
        let child_state = allocator.get_node_read(node_id).state.clone();

        for (probability, action) in policies.iter().zip(corresponding_actions).filter(|(p, _)| **p > 0.0) {
            let mut child_state = child_state.clone();

            #[cfg(debug_assertions)]
            if action.is_null() {
                panic!(
                    "[SearchTree::node_expand] Action with non zero probability ({:?}) is null",
                    probability
                );
            }

            child_state.do_action(*action, false)?;
            let child_id = allocator.new_node(child_state, Some(node_id), Some(*action), Some(*probability));

            allocator.get_node_write(node_id).children.push(child_id);
        }

        Ok(())
    }

    /// Selects the best child node of the given parent node using the tree policy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the parent node to select the best child node from.
    /// * `allocator` - The allocator to use for the selection.
    /// * `tree_policy` - The tree policy to use for the selection.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    pub fn select(node_id: NodeId, allocator: &AreaAllocator, tree_policy: &impl TreePolicy) -> NodeId {
        let parent = NodeWrapper { node_id, allocator };
        let children = allocator
            .get_node_read(node_id)
            .children
            .iter()
            .map(|node_id| NodeWrapper {
                node_id: *node_id,
                allocator,
            })
            .collect::<Vec<_>>();

        debug_assert!(!children.is_empty(), "[Node::select] Node has no children");

        let selected_child = tree_policy.select_node(&parent, children.iter());

        selected_child.node_id
    }

    /// Backpropagates the score of the game up from the given node until the root node is reached.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to backpropagate from.
    /// * `value` - The value to backpropagate.
    /// * `allocator` - The allocator to use for the backpropagation.
    /// * `amount` - The amount of times the value should be backpropagated.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the depth of the current node as the chain until the root needs to be traversed
    pub fn backpropagate(mut node_id: NodeId, value: f32, allocator: &AreaAllocator, amount: i32) {
        loop {
            let mut node = allocator.get_node_write(node_id);

            node.neutral_max_score = node.neutral_max_score.max(value);
            node.neutral_min_score = node.neutral_min_score.min(value);
            node.neutral_score_sum += value as f64 * amount as f64;
            node.neutral_wins += if value > 0.0 { amount } else { -amount };
            node.visit_count += amount as usize;
            node.decrement_virtual_loss_by(amount);

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
                drop(node);
            } else {
                break;
            }
        }
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

        wins - self.virtual_loss.load(Ordering::Relaxed)
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

struct NodeWrapper<'a> {
    node_id: NodeId,
    allocator: &'a AreaAllocator,
}

impl TreePolicyNode for NodeWrapper<'_> {
    type Player = bool;
    #[inline(always)]
    fn visit_count(&self) -> usize {
        self.allocator.get_node_read(self.node_id).visit_count()
    }
    #[inline(always)]
    fn current_player(&self) -> Self::Player {
        self.allocator.get_node_read(self.node_id).current_player()
    }
    #[inline(always)]
    fn wins_for(&self, player: Self::Player) -> i32 {
        self.allocator.get_node_read(self.node_id).wins_for(player)
    }
    #[inline(always)]
    fn maximum_score_for(&self, player: Self::Player) -> f64 {
        self.allocator.get_node_read(self.node_id).maximum_score_for(player)
    }
    #[inline(always)]
    fn minimum_score_for(&self, player: Self::Player) -> f64 {
        self.allocator.get_node_read(self.node_id).minimum_score_for(player)
    }
    #[inline(always)]
    fn score_range(&self) -> f64 {
        self.allocator.get_node_read(self.node_id).score_range()
    }
    #[inline(always)]
    fn score_sum_for(&self, player: Self::Player) -> f64 {
        self.allocator.get_node_read(self.node_id).score_sum_for(player)
    }
    #[inline(always)]
    fn prior_value(&self) -> f64 {
        self.allocator.get_node_read(self.node_id).prior_value()
    }
}
