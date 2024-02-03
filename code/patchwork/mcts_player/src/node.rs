use std::{
    cell::RefCell,
    fmt,
    num::NonZeroUsize,
    rc::{Rc, Weak},
    thread,
};

use patchwork_core::{ActionId, Evaluator, Patchwork, PatchworkError, TreePolicy, TreePolicyNode};
use rand::seq::SliceRandom;



type Link = Rc<RefCell<Node>>;
type WeakLink = Weak<RefCell<Node>>;

#[derive(Clone)]
pub struct Node {
    /// The state of the game at this node.
    pub state: Patchwork,
    /// The parent node. None if this is the root node.
    pub parent: Option<WeakLink>,
    /// The action that was taken to get to this node. None if this is the root node.
    pub action_taken: Option<ActionId>,
    /// The children nodes.
    pub children: Vec<Link>,
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
    pub fn new(state: Patchwork, parent: Option<WeakLink>, action_taken: Option<ActionId>) -> Self {
        let mut expandable_actions: Vec<ActionId> = state.get_valid_actions().into_iter().collect();
        expandable_actions.shuffle(&mut rand::thread_rng());

        Self {
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
    /// # Arguments
    ///
    /// * `node` - The node to check if it is fully expanded.
    ///
    /// # Returns
    ///
    /// `true` if the node is fully expanded, `false` otherwise.
    pub fn is_fully_expanded(node_link: &Link) -> bool {
        let node = RefCell::borrow(node_link);
        node.state.is_terminated() || node.expandable_actions.is_empty()
    }

    /// Whether the node is a terminal node.
    ///
    /// A node is terminal if the game state of the node is a terminal state.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check if it is terminal.
    ///
    /// # Returns
    ///
    /// `true` if the node is terminal, `false` otherwise.
    pub fn is_terminal(node_link: &Link) -> bool {
        RefCell::borrow(node_link).state.is_terminated()
    }

    /// Selects the best child node of the given parent node using the given tree policy.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to select the best child from.
    /// * `tree_policy` - The tree policy to use for selecting the best child.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    pub fn select(node_link: &Link, tree_policy: &impl TreePolicy) -> Link {
        /// A wrapper around a link to a node that implements the `TreePolicyNode` trait. This is
        /// used to circumvent the orphan rule for implementing traits for types from other crates.
        struct LinkNode<'a>(pub &'a Link);
        #[rustfmt::skip]
        impl TreePolicyNode for LinkNode<'_> {
            type Player = bool;
            fn visit_count(&self) -> i32 { RefCell::borrow(self.0).visit_count() }
            fn current_player(&self) -> Self::Player { RefCell::borrow(self.0).current_player() }
            fn wins_for(&self, player: Self::Player) -> i32 { RefCell::borrow(self.0).wins_for(player) }
            fn maximum_score_for(&self, player: Self::Player) -> i32 { RefCell::borrow(self.0).maximum_score_for(player) }
            fn minimum_score_for(&self, player: Self::Player) -> i32 { RefCell::borrow(self.0).minimum_score_for(player) }
            fn score_range(&self) -> i32 { RefCell::borrow(self.0).score_range() }
            fn score_sum_for(&self, player: Self::Player) -> i64 { RefCell::borrow(self.0).score_sum_for(player) }
        }

        let parent = LinkNode(node_link);
        let children_vec = RefCell::borrow(node_link).children.clone();
        let children = children_vec.iter().map(LinkNode).collect::<Vec<_>>();

        let selected_child = tree_policy.select_node(&parent, children.iter());
        Rc::clone(selected_child.0)
    }

    /// Expands this node by adding a child node.
    /// The child node is chosen randomly from the expandable actions.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to expand.
    ///
    /// # Returns
    ///
    /// The new child node.
    pub fn expand(node_link: &Link) -> Result<Link, PatchworkError> {
        let mut node = RefCell::borrow_mut(node_link);
        let action = node.expandable_actions.remove(0);

        let mut next_state = node.state.clone();
        next_state.do_action(action, false)?;

        let child = Rc::new(RefCell::new(Node::new(
            next_state,
            Some(Rc::downgrade(node_link)),
            Some(action),
        )));

        node.children.push(Rc::clone(&child));
        Ok(child)
    }

    /// Simulates the game from this node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to simulate from.
    /// * `evaluator` - The evaluator to evaluate the game state.
    ///
    /// # Returns
    ///
    /// The score of the game from this node derived from the simulation with the evaluator.
    pub fn simulate(node_link: &Link, evaluator: &impl Evaluator) -> i32 {
        let state = &RefCell::borrow(node_link).state;

        evaluator.evaluate_node(state)
    }

    /// Simulates the game from this node in parallel.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to simulate from.
    /// * `evaluator` - The evaluator to evaluate the game state.
    /// * `leaf_parallelization` - The number of games that are played in parallel to get a more
    ///  accurate score for the node.
    ///
    /// # Returns
    ///
    /// The scores of the games from this node derived from the simulation with the evaluator.
    pub fn leaf_parallelized_simulate(
        node_link: &Link,
        evaluator: &impl Evaluator,
        leaf_parallelization: NonZeroUsize,
    ) -> Vec<i32> {
        let state = &RefCell::borrow(node_link).state;

        if Node::is_terminal(node_link) {
            return vec![evaluator.evaluate_terminal_node(state)];
        }

        if leaf_parallelization.get() == 1 {
            return vec![evaluator.evaluate_intermediate_node(state)];
        }

        thread::scope(|s| {
            (0..leaf_parallelization.get())
                .map(|_| s.spawn(|| evaluator.evaluate_intermediate_node(state)))
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        })
    }

    /// Backpropagates the score of the game up until the parent node is reached.
    ///
    /// # Parameters
    ///
    /// * `node` - The node to backpropagate from.
    /// * `value` - The value to backpropagate.
    pub fn backpropagate(node_link: &Link, value: i32) {
        let mut node = RefCell::borrow_mut(node_link);
        node.neutral_max_score = node.neutral_max_score.max(value);
        node.neutral_min_score = node.neutral_min_score.min(value);
        node.neutral_score_sum += value as i64;
        node.neutral_wins += if value > 0 { 1 } else { -1 };
        node.visit_count += 1;

        if let Some(ref parent) = node.parent.as_ref().and_then(|p| p.upgrade()) {
            Node::backpropagate(parent, value);
        }
    }

    /// Backpropagates the scores of the games up until the parent node is reached.
    ///
    /// # Parameters
    ///
    /// * `node` - The node to backpropagate from.
    /// * `values` - The values to backpropagate.
    pub fn leaf_parallelized_backpropagate(node_link: &Link, values: Vec<i32>) {
        let mut node = RefCell::borrow_mut(node_link);
        for value in values.iter() {
            node.neutral_max_score = node.neutral_max_score.max(*value);
            node.neutral_min_score = node.neutral_min_score.min(*value);
            node.neutral_score_sum += *value as i64;
            node.neutral_wins += if *value > 0 { 1 } else { -1 };
            node.visit_count += 1;
        }

        if let Some(ref parent) = node.parent.as_ref().and_then(|p| p.upgrade()) {
            Node::leaf_parallelized_backpropagate(parent, values);
        }
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

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("parent", &self.parent)
            // .field("children", &self.children)
            .field("max_score", &self.neutral_max_score)
            .field("min_score", &self.neutral_min_score)
            .field("score_sum", &self.neutral_score_sum)
            .field("wins", &self.neutral_wins)
            .field("visit_count", &self.visit_count)
            .field("action_taken", &self.action_taken)
            .field("expandable_actions", &self.expandable_actions)
            .finish()
    }
}
