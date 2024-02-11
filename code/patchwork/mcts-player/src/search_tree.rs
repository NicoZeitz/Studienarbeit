use std::{cmp::Reverse, collections::VecDeque, num::NonZeroUsize, thread};

use itertools::Itertools;

use patchwork_core::{Evaluator, Notation, Patchwork, PatchworkError, PlayerResult, TreePolicy, TreePolicyNode};

use crate::{AreaAllocator, NodeDebug, NodeId, Tree};

/// A Search Tree for the Monte Carlo Tree Search (MCTS) algorithm.
pub struct SearchTree<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> {
    // The root node were to start searching for.
    pub(crate) root: NodeId,
    /// The allocator to allocate new nodes.
    pub(crate) allocator: AreaAllocator,
    /// The policy to select nodes during the selection phase.
    tree_policy: &'tree_lifetime Policy,
    /// The evaluator to evaluate the game state.
    evaluator: &'tree_lifetime Eval,
    /// The depth of the current search tree.
    depth: usize,
    /// Whether the search tree is reused.
    reused: bool,
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
    ///
    /// # Returns
    ///
    /// The new [`SearchTree`].
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn new(game: &Patchwork, tree_policy: &'tree_lifetime Policy, evaluator: &'tree_lifetime Eval) -> Self {
        let mut allocator = AreaAllocator::new();

        let root = allocator.new_node(game.clone(), None, None);

        SearchTree {
            root,
            allocator,
            tree_policy,
            evaluator,
            depth: 0,
            reused: false,
        }
    }

    /// Creates a new [`SearchTree`] with the given allocator, game, policy, evaluator and options.
    ///
    /// # Arguments
    ///
    /// * `allocator` - The allocator to allocate new nodes.
    /// * `game` - The game to search.
    /// * `tree_policy` - The policy to select nodes during the selection phase.
    /// * `evaluator` - The evaluator to evaluate the game state.
    /// * `options` - The options for the search.
    ///
    /// # Returns
    ///
    /// The new [`SearchTree`].
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    fn new_with_allocator(
        mut allocator: AreaAllocator,
        game: &Patchwork,
        tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
    ) -> Self {
        allocator.clear();
        let root = allocator.new_node(game.clone(), None, None);

        SearchTree {
            root,
            allocator,
            tree_policy,
            evaluator,
            depth: 0,
            reused: false,
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
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `𝑛` is the number of nodes in the current search tree
    /// and `𝑚` is the number of children of each node.
    pub fn from_root(
        last_tree: Option<Tree>,
        game: &Patchwork,
        tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
        abort_search_after: Option<std::time::Duration>,
    ) -> PlayerResult<Self> {
        let Some(mut last_tree) = last_tree else {
            return Ok(Self::new(game, tree_policy, evaluator));
        };

        let mut queue = VecDeque::new();
        queue.push_back((0, last_tree.root));

        let start_time = std::time::Instant::now();

        loop {
            if queue.is_empty() || abort_search_after.map_or(false, |time| start_time.elapsed() > time) {
                break;
            }

            let (depth, node_id) = queue.pop_front().unwrap();
            if depth >= 8 {
                // After the ply by MCTS-Player the other player can only play a maximum of 7 consecutive actions before
                // the MCTS-Player has to play again.
                // As the depth is greater than this amount we can stop the search here.
                break;
            }

            let node = last_tree.allocator.get_node(node_id);

            if node.state == *game {
                // found the correct node
                let node_id = last_tree.allocator.realloc_to_new_root(node_id);

                return Ok(SearchTree {
                    root: node_id,
                    tree_policy,
                    evaluator,
                    depth: 0,
                    reused: true,
                    allocator: last_tree.allocator,
                });
            }

            for child in node.children.iter() {
                queue.push_back((depth + 1, *child));
            }
        }

        // The root node was not found in the tree.
        // This means that the tree is not reusable.
        Ok(Self::new_with_allocator(
            last_tree.allocator,
            game,
            tree_policy,
            evaluator,
        ))
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
        let mut node_id = self.root;

        // 1. Selection
        let mut new_depth = 0;                                                               // Statistics
        while self.should_be_selected(node_id) {
            node_id = self.node_select(node_id);
            new_depth += 1;
        }
        self.depth = self.depth.max(new_depth);                                                     // Statistics

        if leaf_parallelization.get() == 1 {
            let value = if self.is_terminal(node_id) {
                // 3. Leaf/Terminal Node → Direct Evaluation
                let node = self.allocator.get_node(node_id);
                self.evaluator.evaluate_terminal_node(&node.state)
            } else {
                // 2. Expansion
                node_id = self.node_expand(node_id)?;

                // 3. Simulation
                self.node_simulate(node_id)
            };

            // 4. Backpropagation
            self.node_backpropagate(node_id, value);
        } else {
            let values = if self.is_terminal(node_id) {
                // 3. Leaf/Terminal Node → Direct Evaluation
                let node = self.allocator.get_node(node_id);
                vec![self.evaluator.evaluate_terminal_node(&node.state)]
            } else {
                // 2. Expansion
                node_id = self.node_expand(node_id)?;

                // 3. Simulation
                self.node_leaf_parallelized_simulate(node_id, leaf_parallelization)
            };

            // 4. Backpropagation
            self.node_leaf_parallelized_backpropagate(node_id, values);
        }

        Ok(())
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
    pub fn get_nodes(&self) -> usize {
        self.allocator.size()
    }

    /// Gets the win prediction for the root node.
    ///
    /// # Returns
    ///
    /// The win prediction for the root node.
    pub fn get_win_prediction(&self) -> f64 {
        let root = self.allocator.get_node(self.root);
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
        let root = self.allocator.get_node(self.root);
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
        let root = self.allocator.get_node(self.root);
        let root_player = root.state.is_player_1();

        root.maximum_score_for(root_player)
    }

    /// Gets the amount of actions inside the root node
    ///
    /// # Returns
    ///
    /// The amount of actions inside the root node
    pub fn get_root_actions(&self) -> usize {
        let root = self.allocator.get_node(self.root);
        root.expandable_actions.len() + root.children.len()
    }

    /// Gets the action line of the principal variation (PV) [The child nodes with the most amount
    /// of visits] from the root node.
    ///
    /// # Returns
    ///
    /// The action line of the principal variation.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the number of nodes in the current search tree
    pub fn get_pv_action_line(&self) -> String {
        let mut actions = vec![];
        let mut current_node = self.root;
        loop {
            let node = self.allocator.get_node(current_node);

            if let Some(action) = node.action_taken {
                actions.push(action);
            }

            if node.is_terminal() || node.children.is_empty() {
                break;
            }

            let next = node
                .children
                .iter()
                .max_by_key(|child| self.allocator.get_node(**child).visit_count)
                .unwrap();

            current_node = *next;
        }

        let action_line = actions
            .iter()
            .map(|action| match action.save_to_notation() {
                Ok(notation) => notation,
                Err(_) => "######".to_string(),
            })
            .join(" → ");

        action_line
    }

    /// Writes the whole search tree to the given writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write the search tree to.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the writing was successful, otherwise a `std::io::Error`.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the number of nodes in the current search tree
    pub fn write_tree(&self, writer: &mut Box<dyn std::io::Write>) -> std::io::Result<()> {
        let lines = self.tree_to_string(self.root);
        writeln!(writer, "{}", lines.join("\n"))?;
        Ok(())
    }

    fn tree_to_string(&self, node_id: NodeId) -> Vec<String> {
        let mut result = Vec::new();

        let node = self.allocator.get_node(node_id);
        result.push(format!(
            "{:?}",
            NodeDebug {
                node,
                allocator: &self.allocator
            }
        ));

        for (index, child) in node
            .children
            .iter()
            .sorted_by_key(|child_id| Reverse(self.allocator.get_node(**child_id).visit_count))
            .enumerate()
        {
            let child = self.allocator.get_node(*child);

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

            for (inner_index, line) in self.tree_to_string(child.id).iter().enumerate() {
                let front = if inner_index == 0 { branching_front } else { other_front };
                result.push(format!("{}{}", front, line));
            }
        }

        result
    }
}

/// Implementation of the methods for the Monte Carlo Tree Search (MCTS) algorithm.
///
/// 1. `Select`
/// 2. `Expand`
/// 3. `Simulate`
/// 4. `Backpropagate`
///
/// as well as some utility methods
impl<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> SearchTree<'tree_lifetime, Policy, Eval> {
    /// Selects the best child node of the given parent node using the tree policy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the parent node to select the best child node from.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    fn node_select(&self, node_id: NodeId) -> NodeId {
        let node = self.allocator.get_node(node_id);
        let children = node.children.iter().map(|node_id| self.allocator.get_node(*node_id));

        let selected_child = self.tree_policy.select_node(node, children);
        selected_child.id
    }

    /// Expands the given node by adding a child node.
    /// The child node is chosen randomly from the expandable actions.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to expand.
    ///
    /// # Returns
    ///
    /// The new child node.
    fn node_expand(&mut self, node_id: NodeId) -> Result<NodeId, PatchworkError> {
        let node = self.allocator.get_node_mut(node_id);
        let action = node.expandable_actions.remove(0);

        let mut next_state = node.state.clone();
        next_state.do_action(action, false)?;

        let child_id = self.allocator.new_node(next_state, Some(node_id), Some(action));

        Ok(child_id)
    }

    /// Simulates the game from the given node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to simulate the game from.
    ///
    /// # Returns
    ///
    /// The score of the game from this node derived from the simulation with the evaluator.
    pub fn node_simulate(&self, node_id: NodeId) -> i32 {
        let node = self.allocator.get_node(node_id);
        self.evaluator.evaluate_node(&node.state)
    }

    /// Simulates the game from the given node in parallel.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to simulate the game from.
    /// * `leaf_parallelization` - The number of games that are played in parallel to get a more
    ///  accurate score for the node.
    ///
    /// # Returns
    ///
    /// The scores of the games from this node derived from the simulation with the evaluator.
    pub fn node_leaf_parallelized_simulate(&self, node_id: NodeId, leaf_parallelization: NonZeroUsize) -> Vec<i32> {
        let node = self.allocator.get_node(node_id);

        if node.is_terminal() || leaf_parallelization.get() == 1 {
            return vec![self.evaluator.evaluate_terminal_node(&node.state)];
        }

        thread::scope(|s| {
            (0..leaf_parallelization.get())
                .map(|_| s.spawn(|| self.evaluator.evaluate_intermediate_node(&node.state)))
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        })
    }

    /// Backpropagates the score of the game up from the given node until the parent node is reached.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to backpropagate from.
    /// * `value` - The value to backpropagate.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the depth of the current node as the chain until the parent needs to be traversed
    pub fn node_backpropagate(&mut self, mut node_id: NodeId, value: i32) {
        loop {
            let node = self.allocator.get_node_mut(node_id);

            node.neutral_max_score = node.neutral_max_score.max(value);
            node.neutral_min_score = node.neutral_min_score.min(value);
            node.neutral_score_sum += value as i64;
            node.neutral_wins += if value > 0 { 1 } else { -1 };
            node.visit_count += 1;

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    /// Backpropagates the scores of the games up from the given node until the parent node is reached.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to backpropagate from.
    /// * `values` - The values to backpropagate.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `𝑛` is the depth of the current node as the chain until the parent needs to be traversed and
    /// `𝑚` is the amount of values that need to be propagated
    pub fn node_leaf_parallelized_backpropagate(&mut self, mut node_id: NodeId, values: Vec<i32>) {
        loop {
            let node = self.allocator.get_node_mut(node_id);

            for value in values.iter() {
                node.neutral_max_score = node.neutral_max_score.max(*value);
                node.neutral_min_score = node.neutral_min_score.min(*value);
                node.neutral_score_sum += *value as i64;
                node.neutral_wins += if *value > 0 { 1 } else { -1 };
                node.visit_count += 1;
            }

            if let Some(parent_id) = node.parent {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    /// Whether the given node is terminal.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to check.
    ///
    /// # Returns
    ///
    /// Whether the given node is terminal.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn is_terminal(&self, node_id: NodeId) -> bool {
        self.allocator.get_node(node_id).is_terminal()
    }

    /// Whether the given node is the end of the selection phase or nodes further down should be selected.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node to check.
    ///
    /// # Returns
    ///
    /// Whether the given node is the end of the selection phase or nodes further down should be selected.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn should_be_selected(&self, node_id: NodeId) -> bool {
        let node = self.allocator.get_node(node_id);

        node.is_fully_expanded() && !node.is_terminal()
    }
}
