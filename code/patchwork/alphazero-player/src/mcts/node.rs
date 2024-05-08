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

        let children = policies
            .iter()
            .zip(corresponding_actions)
            .filter(|(p, _)| **p > 0.0)
            .map(|(probability, action)| {
                let mut child_state = child_state.clone();
                child_state.do_action(*action, false)?;
                let child_id = allocator.new_node(child_state, Some(node_id), Some(*action), Some(*probability));

                Ok(child_id)
            })
            .collect::<Result<Vec<_>, PatchworkError>>()?;

        let mut node = allocator.get_node_write(node_id);
        // it could be that another thread already expanded the node. Just ignore the created children in that case
        if node.children.is_empty() {
            node.children = children;
        }
        drop(node);

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
        let parent = allocator.get_node_read(node_id);
        let parent_data = NodeDataSnapshot {
            id: node_id,
            current_player: parent.current_player(),
            visit_count: parent.visit_count(),
            neutral_max_score: f64::from(parent.neutral_max_score),
            neutral_min_score: f64::from(parent.neutral_min_score),
            neutral_score_sum: parent.neutral_score_sum,
            neutral_wins: parent.neutral_wins,
            virtual_loss: parent.virtual_loss.load(Ordering::Relaxed),
            prior: f64::from(parent.prior),
        };
        let children_ids = parent.children.clone();
        drop(parent);
        let children = children_ids
            .iter()
            .map(|child| {
                let child = allocator.get_node_read(*child);
                NodeDataSnapshot {
                    id: child.id,
                    current_player: child.current_player(),
                    visit_count: child.visit_count(),
                    neutral_max_score: f64::from(child.neutral_max_score),
                    neutral_min_score: f64::from(child.neutral_min_score),
                    neutral_score_sum: child.neutral_score_sum,
                    neutral_wins: child.neutral_wins,
                    virtual_loss: child.virtual_loss.load(Ordering::Relaxed),
                    prior: f64::from(child.prior),
                }
            })
            .collect::<Vec<_>>();

        let selected_child = tree_policy.select_node(&parent_data, children.iter());

        selected_child.id
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

    pub fn backpropagate(mut node_id: NodeId, value: f32, allocator: &AreaAllocator, amount: u32) {
        loop {
            let mut node = allocator.get_node_write(node_id);

            node.neutral_max_score = node.neutral_max_score.max(value);
            node.neutral_min_score = node.neutral_min_score.min(value);
            node.neutral_score_sum += f64::from(value) * f64::from(amount);
            node.neutral_wins += if value > 0.0 { amount as i32 } else { -(amount as i32) };
            node.visit_count += amount as usize;
            node.decrement_virtual_loss_by(amount as i32);

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
            f64::from(self.neutral_max_score)
        } else {
            f64::from(-self.neutral_min_score)
        }
    }

    fn minimum_score_for(&self, player: Self::Player) -> f64 {
        // == -self.maximum_score_for(!player)
        if player {
            f64::from(self.neutral_min_score)
        } else {
            f64::from(-self.neutral_max_score)
        }
    }

    fn score_range(&self) -> f64 {
        f64::from(self.neutral_max_score - self.neutral_min_score)
    }

    fn score_sum_for(&self, player: Self::Player) -> f64 {
        if player {
            self.neutral_score_sum - f64::from(self.virtual_loss.load(Ordering::Relaxed))
        } else {
            -self.neutral_score_sum - f64::from(self.virtual_loss.load(Ordering::Relaxed))
        }
    }

    fn prior_value(&self) -> f64 {
        f64::from(self.prior)
    }
}

struct NodeDataSnapshot {
    pub id: NodeId,
    current_player: bool,
    visit_count: usize,
    neutral_max_score: f64,
    neutral_min_score: f64,
    neutral_score_sum: f64,
    neutral_wins: i32,
    virtual_loss: i32,
    prior: f64,
}

impl TreePolicyNode for NodeDataSnapshot {
    type Player = bool;
    #[inline]
    fn visit_count(&self) -> usize {
        self.visit_count
    }
    #[inline]
    fn current_player(&self) -> Self::Player {
        self.current_player
    }
    #[inline]
    fn wins_for(&self, player: Self::Player) -> i32 {
        let wins = if player { self.neutral_wins } else { -self.neutral_wins };

        wins - self.virtual_loss
    }
    #[inline]
    fn maximum_score_for(&self, player: Self::Player) -> f64 {
        if player {
            self.neutral_max_score
        } else {
            -self.neutral_min_score
        }
    }
    #[inline]
    fn minimum_score_for(&self, player: Self::Player) -> f64 {
        if player {
            self.neutral_min_score
        } else {
            -self.neutral_max_score
        }
    }
    #[inline]
    fn score_range(&self) -> f64 {
        self.neutral_max_score - self.neutral_min_score
    }
    #[inline]
    fn score_sum_for(&self, player: Self::Player) -> f64 {
        if player {
            self.neutral_score_sum
        } else {
            -self.neutral_score_sum
        }
    }
    #[inline]
    fn prior_value(&self) -> f64 {
        self.prior
    }
}
