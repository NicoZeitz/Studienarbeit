use std::{
    cell::{Ref, RefCell},
    cmp::Reverse,
    collections::VecDeque,
    num::NonZeroUsize,
    rc::Rc,
};

use itertools::Itertools;

use patchwork_core::{
    ActionId, Evaluator, Notation, Patchwork, PatchworkError, PlayerResult, TreePolicy, TreePolicyNode,
};

use crate::Node;

/// A Search Tree for the Monte Carlo Tree Search (MCTS) algorithm.
pub struct SearchTree<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> {
    // The root node were to start searching for.
    pub(crate) root: Rc<RefCell<Node>>,
    /// The policy to select nodes during the selection phase.
    tree_policy: &'tree_lifetime Policy,
    /// The evaluator to evaluate the game state.
    evaluator: &'tree_lifetime Eval,
    /// The depth of the current search tree.
    depth: usize,
    /// Whether the search tree is reused.
    reused: bool,
    /// The amount of nodes in this search tree.
    nodes: usize,
}

macro_rules! playout {
    ($self:expr, $evaluate:expr, $simulate:expr, $backpropagate:expr) => {{
        let mut node = Rc::clone(&$self.root);

        // 1. Selection
        let mut new_depth = 0;
        while Node::is_fully_expanded(&node) && !Node::is_terminal(&node) {
            node = Node::select(&node, $self.tree_policy);
            new_depth += 1;
        }
        $self.depth = $self.depth.max(new_depth);

        #[allow(clippy::redundant_closure_call)]
        let value = if Node::is_terminal(&node) {
            // 3. Leaf/Terminal Node → Direct Evaluation
            let state = &RefCell::borrow(&node).state;

            $evaluate(state)
        } else {
            // 2. Expansion
            let new_node = Node::expand(&node)?;
            node = new_node;

            // 3. Simulation
            $simulate(&node)
        };

        // 4. Backpropagation
        #[allow(clippy::redundant_closure_call)]
        $backpropagate(&node, value);

        Ok(())
    }};
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
            reused: false,
            nodes: 0,
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
        root: Option<Rc<RefCell<Node>>>,
        game: &Patchwork,
        tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
        abort_search_after: Option<std::time::Duration>,
    ) -> PlayerResult<Self> {
        let Some(root) = root else {
            return Ok(Self::new(game, tree_policy, evaluator));
        };

        let mut queue = VecDeque::new();
        queue.push_back((0, root));

        let start_time = std::time::Instant::now();

        loop {
            if queue.is_empty() || abort_search_after.map_or(false, |time| start_time.elapsed() > time) {
                break;
            }

            let (depth, node_ref) = queue.pop_front().unwrap();
            if depth >= 8 {
                // After the ply by MCTS-Player the other player can only play a maximum of 7 consecutive actions before
                // the MCTS-Player has to play again.
                // As the depth is greater than this amount we can stop the search here.
                break;
            }

            let mut node = RefCell::borrow_mut(&node_ref);

            if node.state == *game {
                // found the correct node
                node.parent = None;
                node.action_taken = None;
                let nodes = calculate_nodes(&node);
                drop(node);
                return Ok(SearchTree {
                    root: node_ref,
                    tree_policy,
                    evaluator,
                    depth: 0,
                    reused: true,
                    nodes,
                });
            }

            for child in &node.children {
                queue.push_back((depth + 1, Rc::clone(child)));
            }
        }

        // The root node was not found in the tree.
        // This means that the tree is not reusable.
        Ok(Self::new(game, tree_policy, evaluator))
    }

    /// Plays out a single iteration of the MCTS algorithm. The random playouts can be done in
    /// parallel. This is controlled by the given `leaf_parallelization`.
    ///
    /// # Arguments
    ///
    /// * `leaf_parallelization` - The number of parallel playouts to run.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the playout was successful, otherwise a `PatchworkError`.
    #[rustfmt::skip]
    pub fn playout(&mut self, leaf_parallelization: NonZeroUsize) -> Result<(), PatchworkError> {
        if leaf_parallelization.get() == 1 {
            playout!(
                self,
                |state| self.evaluator.evaluate_terminal_node(state),
                |node| {
                    self.nodes += 1;
                    Node::simulate(node, self.evaluator)
                },
                Node::backpropagate
            )
        } else {
            playout!(
                self,
                |state| vec![self.evaluator.evaluate_terminal_node(state)],
                |node| {
                    self.nodes += 1;
                    Node::leaf_parallelized_simulate(node, self.evaluator, leaf_parallelization)
                },
                Node::leaf_parallelized_backpropagate
            )
        }

    }

    /// Gets the depth of the principal variation as long as all actions are expanded.
    ///
    /// # Returns
    ///
    /// The depth of the search tree.
    #[inline(always)]
    pub const fn get_expanded_depth(&self) -> usize {
        self.depth
    }

    /// Whether the search tree is reused.
    ///
    /// # Returns
    ///
    /// Whether the search tree is reused.
    #[inline(always)]
    pub const fn is_reused(&self) -> bool {
        self.reused
    }

    /// Gets the amount of nodes in this search tree.
    ///
    /// # Returns
    ///
    /// The amount of nodes in this search tree.
    #[inline(always)]
    pub const fn get_nodes(&self) -> usize {
        self.nodes
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

    /// Gets the minimum score of all games played from the root node from the perspective of the
    /// current player at the root node.
    ///
    /// # Returns
    ///
    /// The minimum score of all games played from the root node from the perspective of the current
    /// player.
    pub fn get_min_score(&self) -> i32 {
        let root = RefCell::borrow(&self.root);
        let root_player = root.state.is_player_1();

        root.minimum_score_for(root_player)
    }

    /// Gets the maximum score of all games played from the root node from the perspective of the
    /// current player at the root node.
    ///
    /// # Returns
    ///
    /// The maximum score of all games played from the root node from the perspective of the current
    /// player.
    pub fn get_max_score(&self) -> i32 {
        let root = RefCell::borrow(&self.root);
        let root_player = root.state.is_player_1();

        root.maximum_score_for(root_player)
    }

    /// Gets the action line of the principal variation (PV) [The child nodes with the most amount
    /// of visits] from the root node.
    ///
    /// # Returns
    ///
    /// The action line of the principal variation.
    pub fn get_pv_action_line(&self) -> String {
        fn traverse(node: &Rc<RefCell<Node>>, pv_line: &mut Vec<ActionId>) {
            let node = RefCell::borrow(node);

            if let Some(action) = node.action_taken {
                pv_line.push(action);
            }

            if node.state.is_terminated() || node.children.is_empty() {
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

    pub fn write_tree(&self, writer: &mut Box<dyn std::io::Write>) -> std::io::Result<()> {
        fn tree_to_string(node: Ref<'_, Node>) -> Vec<String> {
            let mut result = Vec::new();

            result.push(format!("{:?}", node));

            for (index, child) in node
                .children
                .iter()
                .sorted_by_key(|child| Reverse(RefCell::borrow(child).visit_count))
                .enumerate()
            {
                let child = RefCell::borrow(child);

                let branching_front = if index == node.children.len() - 1 {
                    "└───"
                } else {
                    "├───"
                };
                let other_front = if index == node.children.len() - 1 {
                    "    "
                } else {
                    "│   "
                };

                for (inner_index, line) in tree_to_string(child).iter().enumerate() {
                    let front = if inner_index == 0 { branching_front } else { other_front };
                    result.push(format!("{}{}", front, line));
                }
            }

            result
        }

        let root = RefCell::borrow(&self.root);
        let lines = tree_to_string(root);
        writeln!(writer, "{}", lines.join("\n"))?;
        Ok(())
    }
}

fn calculate_nodes(node: &Node) -> usize {
    let mut nodes = 1;

    for child in &node.children {
        nodes += calculate_nodes(&RefCell::borrow(child));
    }

    nodes
}
