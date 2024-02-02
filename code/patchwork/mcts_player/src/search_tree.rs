use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

use itertools::Itertools;

use patchwork_core::{ActionId, Evaluator, Notation, Patchwork, PatchworkError, TreePolicy, TreePolicyNode};

use crate::Node;

/// A Search Tree for the Monte Carlo Tree Search (MCTS) algorithm.
pub struct SearchTree<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> {
    // The root node were to start searching for.
    root: Rc<RefCell<Node>>,
    /// The policy to select nodes during the selection phase.
    tree_policy: &'tree_lifetime Policy,
    /// The evaluator to evaluate the game state.
    evaluator: &'tree_lifetime Eval,
    /// The depth of the current search tree.
    depth: usize,
}

impl<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> SearchTree<'tree_lifetime, Policy, Eval> {
    /// Creates a new [`SearchTree`] with the given game, policy, evaluator and options.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search.
    /// * `tree_policy` - The policy to select nodes during the selection phase.
    /// * `evaluator` - The evaluator to evaluate the game state.
    /// * `options` - The options for the search.
    pub fn new(game: &Patchwork, tree_policy: &'tree_lifetime Policy, evaluator: &'tree_lifetime Eval) -> Self {
        let root = Rc::new(RefCell::new(Node::new(game.clone(), None, None)));

        SearchTree {
            root,
            tree_policy,
            evaluator,
            depth: 0,
        }
    }

    /// Creates a new [`SearchTree`] from an old root node.
    /// This is useful to reuse the tree from the last search.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node from the last search if there was one.
    /// * `game` - The game to search.
    /// * `tree_policy` - The policy to select nodes during the selection
    ///  phase.
    /// * `evaluator` - The evaluator to evaluate the game state.
    /// * `options` - The options for the search.
    ///
    /// # Returns
    ///
    /// The new [`SearchTree`].
    pub fn from_root(
        root: Option<Node>,
        game: &Patchwork,
        tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
    ) -> Self {
        if root.is_none() {
            return Self::new(game, tree_policy, evaluator);
        }

        // TODO:

        let root = Rc::new(RefCell::new(Node::new(game.clone(), None, None)));

        SearchTree {
            root,
            tree_policy,
            evaluator,
            depth: 0,
        }
    }

    /// Plays out a single iteration of the MCTS algorithm.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the playout was successful, otherwise a `PatchworkError`.
    pub fn playout(&mut self, leaf_parallelization: NonZeroUsize) -> Result<(), PatchworkError> {
        let mut node = Rc::clone(&self.root);

        // 1. Selection
        let mut new_depth = 0;
        while Node::is_fully_expanded(&node) && !Node::is_terminal(&node) {
            node = Node::select(&node, self.tree_policy);
            new_depth += 1;
        }
        self.depth = self.depth.max(new_depth);

        let value = if Node::is_terminal(&node) {
            // 3. Leaf/Terminal Node → Direct Evaluation
            let state = &RefCell::borrow(&node).state;

            self.evaluator.evaluate_terminal_node(state)
        } else {
            // 2. Expansion
            let new_node = Node::expand(&node)?;
            node = new_node;

            // 3. Simulation
            Node::simulate(&node, self.evaluator, leaf_parallelization)
        };

        // 4. Backpropagation
        Node::backpropagate(&node, value);

        Ok(())
    }

    /// Picks the best action from the root node.
    /// This is done by selecting the child node with the highest number of visits.
    /// If there are multiple child nodes with the same number of visits, the action with the
    /// greater amount of wins is chosen. If there are still multiple actions with the same amount
    /// of wins, one of them is chosen randomly.
    ///
    /// # Returns
    ///
    /// The best action from the root node.
    ///
    /// # Panics
    ///
    /// Panics if the root node is not fully expanded.
    pub fn pick_best_action(&self) -> ActionId {
        if !Node::is_fully_expanded(&self.root) {
            panic!("[SearchTree::pick_best_action] The root node is not fully expanded.")
        }

        let root = RefCell::borrow(&self.root);
        let root_player = root.state.is_player_1();

        let best_action = root
            .children
            .iter()
            .max_by_key(|child| {
                let child = RefCell::borrow(child);
                (child.visit_count, child.wins_for(root_player))
            })
            .unwrap()
            .borrow()
            .action_taken
            .unwrap();

        best_action
    }

    /// Gets the depth for how many plys all actions have been expanded.
    ///
    /// # Returns
    ///
    /// The depth of the search tree.
    #[inline(always)]
    pub const fn get_expanded_depth(&self) -> usize {
        self.depth
    }

    /// Gets the win prediction for the root node.
    ///
    /// # Returns
    ///
    /// The win prediction for the root node.
    pub fn get_win_prediction(&self) -> f64 {
        let root = RefCell::borrow(&self.root);
        let root_player = root.state.is_player_1();

        let root_wins = root.wins_for(root_player).abs() as f64;
        let root_visits = root.visit_count as f64;

        root_wins / root_visits
    }

    pub fn get_pv_action_line(&self) -> String {
        fn traverse(node: &Rc<RefCell<Node>>, pv_line: &mut Vec<ActionId>) {
            let node = RefCell::borrow(node);

            if let Some(action) = node.action_taken {
                pv_line.push(action);
            }

            if node.state.is_terminated() || !node.expandable_actions.is_empty() {
                return;
            }

            let next = node
                .children
                .iter()
                .max_by_key(|child| RefCell::borrow(child).visit_count)
                .unwrap();

            traverse(next, pv_line);
        }

        let mut actions = vec![];

        traverse(&self.root, &mut actions);

        let action_line = actions
            .iter()
            .map(|action| match action.save_to_notation() {
                Ok(notation) => notation,
                Err(_) => "######".to_string(),
            })
            .join(" → ");

        action_line
    }
}
