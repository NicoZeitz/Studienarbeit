use core::fmt;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, RwLockReadGuard,
    },
    thread,
};

use crate::{mcts_options::NON_ZERO_USIZE_ONE, MCTSEndCondition, MCTSOptions, Node};
use patchwork_core::{Action, Evaluator, Patchwork};
use patchwork_tree_policy::TreePolicy;

// TODO:
// TreeParallelization (weg)
// Run MCTS while other player is thinking
// Tree reuse

/// A Search Tree for the Monte Carlo Tree Search (MCTS) algorithm.
pub struct SearchTree<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> {
    // The root node of the search tree.
    root_node: Arc<RwLock<Node>>,
    /// The policy to select nodes during the selection phase.
    patchwork_tree_policy: &'tree_lifetime Policy,
    /// The evaluator to evaluate the game state.
    evaluator: &'tree_lifetime Eval,
    /// The options for the search.
    options: &'tree_lifetime MCTSOptions,
}

impl<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> SearchTree<'tree_lifetime, Policy, Eval> {
    #[allow(dead_code)]
    const AMOUNT_PROGRESS_REPORTS: usize = 10;

    /// Creates a new [`SearchTree`] with the given game, policy, evaluator and options.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search.
    /// * `patchwork_tree_policy` - The policy to select nodes during the selection phase.
    /// * `evaluator` - The evaluator to evaluate the game state.
    /// * `options` - The options for the search.
    pub fn new(
        game: &Patchwork,
        patchwork_tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
        options: &'tree_lifetime MCTSOptions,
    ) -> Self {
        let root_node = Arc::new(RwLock::new(Node::new(game, None, None)));

        SearchTree {
            root_node,
            patchwork_tree_policy,
            evaluator,
            options,
        }
    }

    /// Searches for the best action.
    pub fn search(&self) -> Action {
        // TODO: reuse old tree

        let root = Arc::clone(&self.root_node);

        // PERF: fastpath for when there is only one action
        if root.read().unwrap().expandable_actions.len() == 1 {
            return root.read().unwrap().expandable_actions[0].clone();
        }

        match &self.options {
            MCTSOptions {
                root_parallelization: NON_ZERO_USIZE_ONE,
                leaf_parallelization,
                end_condition,
                reuse_tree: _,
            } => {
                match end_condition {
                    MCTSEndCondition::Iterations(iterations) => {
                        #[cfg(debug_assertions)]
                        let start_time = std::time::Instant::now();
                        #[cfg(debug_assertions)]
                        let mut i = 0;

                        for _ in 0..*iterations {
                            #[cfg(debug_assertions)]
                            if i % (*iterations / SearchTree::<Policy, Eval>::AMOUNT_PROGRESS_REPORTS) == 0 {
                                self.report_progress(i, start_time, vec![&self.root_node]);
                            }
                            #[cfg(debug_assertions)]
                            let _ = i += 1;

                            SearchTree::playout(
                                Arc::clone(&root),
                                self.patchwork_tree_policy,
                                self.evaluator,
                                (*leaf_parallelization).into(),
                            );
                        }
                    }
                    MCTSEndCondition::Time(time) => {
                        let start_time = std::time::Instant::now();

                        #[cfg(debug_assertions)]
                        let mut i = 0;

                        while std::time::Instant::now().duration_since(start_time) <= *time {
                            #[cfg(debug_assertions)]
                            if std::time::Instant::now().duration_since(start_time).as_micros()
                                % (time.as_micros() / SearchTree::<Policy, Eval>::AMOUNT_PROGRESS_REPORTS as u128)
                                == 0
                            {
                                self.report_progress(i, start_time, vec![&self.root_node]);
                            }
                            #[cfg(debug_assertions)]
                            let _ = i += 1;

                            SearchTree::playout(
                                Arc::clone(&root),
                                self.patchwork_tree_policy,
                                self.evaluator,
                                (*leaf_parallelization).into(),
                            );
                        }
                    }
                }

                self.get_best_action(&root)
            }
            MCTSOptions {
                root_parallelization,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Iterations(iterations),
                reuse_tree: _,
            } => thread::scope(|s| {
                let root_parallelization: usize = (*root_parallelization).into();
                let leaf_parallelization: usize = (*leaf_parallelization).into();
                let mut handles = Vec::with_capacity(root_parallelization);
                for _ in 0..root_parallelization {
                    let root = Arc::new(RwLock::new(root.read().unwrap().clone()));
                    let patchwork_tree_policy = self.patchwork_tree_policy;
                    let evaluator = self.evaluator;

                    handles.push(s.spawn(move || {
                        for _ in 0..*iterations {
                            SearchTree::playout(
                                Arc::clone(&root),
                                patchwork_tree_policy,
                                evaluator,
                                leaf_parallelization,
                            );
                        }

                        let actions = root
                            .read()
                            .unwrap()
                            .children
                            .iter()
                            .map(|child| {
                                let child = child.read().unwrap();
                                (child.visit_count, child.action_taken.clone().unwrap())
                            })
                            .collect::<Vec<_>>();

                        let sum_of_visits = actions.iter().map(|(visits, _)| visits).sum::<i32>() as f64;

                        actions
                            .iter()
                            .map(|(visits, action)| (*visits as f64 / sum_of_visits, action.clone()))
                            .collect::<Vec<_>>()
                    }));
                }

                let results = handles
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .collect::<Vec<_>>();

                let actions = results[0].iter().map(|(_, action)| action).collect::<Vec<_>>();

                let mut max_action = None;
                let mut max_probability = 0.0;

                for index in 0..results[0].len() {
                    let summed_probability =
                        results.iter().map(|result| result[index].0).sum::<f64>() / actions.len() as f64;

                    if summed_probability > max_probability {
                        max_action = Some(actions[index].clone());
                        max_probability = summed_probability;
                    }
                }

                max_action.unwrap()
            }),
            MCTSOptions {
                root_parallelization,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Time(duration),
                reuse_tree: _,
            } => {
                let root_parallelization: usize = (*root_parallelization).into();
                let leaf_parallelization: usize = (*leaf_parallelization).into();

                let stop = Arc::new(AtomicBool::new(false));

                thread::scope(|s| {
                    let mut handles = Vec::with_capacity(root_parallelization);
                    for _ in 0..root_parallelization {
                        let root = Arc::new(RwLock::new(root.read().unwrap().clone()));
                        let patchwork_tree_policy = self.patchwork_tree_policy;
                        let evaluator = self.evaluator;
                        let stop = Arc::clone(&stop);

                        handles.push(s.spawn(move || {
                            // while !stop.load(Ordering::SeqCst) {
                            while !stop.load(Ordering::Acquire) {
                                SearchTree::playout(
                                    Arc::clone(&root),
                                    patchwork_tree_policy,
                                    evaluator,
                                    leaf_parallelization,
                                );
                            }

                            let actions = root
                                .read()
                                .unwrap()
                                .children
                                .iter()
                                .map(|child| {
                                    let child = child.read().unwrap();
                                    (child.visit_count, child.action_taken.clone().unwrap())
                                })
                                .collect::<Vec<_>>();

                            let sum_of_visits = actions.iter().map(|(visits, _)| visits).sum::<i32>() as f64;

                            actions
                                .iter()
                                .map(|(visits, action)| (*visits as f64 / sum_of_visits, action.clone()))
                                .collect::<Vec<_>>()
                        }));
                    }

                    thread::sleep(*duration);
                    stop.store(true, Ordering::Release);

                    let results = handles
                        .into_iter()
                        .map(|handle| handle.join().unwrap())
                        .collect::<Vec<_>>();

                    let actions = results[0].iter().map(|(_, action)| action).collect::<Vec<_>>();

                    let mut max_action = None;
                    let mut max_probability = 0.0;

                    for index in 0..results[0].len() {
                        let summed_probability =
                            results.iter().map(|result| result[index].0).sum::<f64>() / actions.len() as f64;

                        if summed_probability > max_probability {
                            max_action = Some(actions[index].clone());
                            max_probability = summed_probability;
                        }
                    }

                    max_action.unwrap()
                })
            }
        }
    }

    fn playout(
        root_node: Arc<RwLock<Node>>,
        patchwork_tree_policy: &Policy,
        evaluator: &Eval,
        leaf_parallelization: usize,
    ) {
        let mut node = root_node;

        // 1. Selection
        while Node::is_fully_expanded(&node) && !Node::is_terminal(&node) {
            node = Node::select(&node, patchwork_tree_policy);
        }

        let value = if !&node.read().unwrap().state.is_terminated() {
            // 2. Expansion
            let new_node = Node::expand(&node);
            node = new_node;

            // 3. Simulation
            Node::simulate(&node, evaluator, leaf_parallelization)
        } else {
            // 3. Leaf node -> direct evaluation
            let game = {
                let node = node.read().unwrap();
                node.state.clone()
            };
            evaluator.evaluate_terminal_node(&game)
        };

        // 4. Backpropagation
        Node::backpropagate(&node, value);
    }

    #[cfg(debug_assertions)]
    fn report_progress(&self, _iteration: usize, _start_time: std::time::Instant, _nodes: Vec<&Arc<RwLock<Node>>>) {}

    pub fn get_best_action(&self, root: &Arc<RwLock<Node>>) -> Action {
        root.read()
            .unwrap()
            .children
            .iter()
            // POSSIBLE IMPROVEMENT: allow for generic implementation for choosing the best action
            .max_by_key(|child| child.read().unwrap().visit_count)
            .unwrap()
            .read()
            .unwrap()
            .action_taken
            .clone()
            .unwrap()
    }

    #[allow(clippy::only_used_in_recursion)]
    fn internal_tree_to_string(&self, node: RwLockReadGuard<'_, Node>) -> Vec<String> {
        let mut result = Vec::new();

        result.push(format!("{:?}", node));

        for (index, child) in node.children.iter().enumerate() {
            let child = child.read().unwrap();

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

            for (inner_index, line) in &mut self.internal_tree_to_string(child).iter().enumerate() {
                let front = if inner_index == 0 { branching_front } else { other_front };
                result.push(format!("{}{}", front, line));
            }
        }

        result
    }

    pub fn tree_to_string(&self) -> String {
        let root = self.root_node.read().unwrap();
        self.internal_tree_to_string(root).join("\n")
    }
}

impl<'tree_lifetime, Policy: TreePolicy, Eval: Evaluator> fmt::Display for SearchTree<'tree_lifetime, Policy, Eval> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tree_to_string())
    }
}
