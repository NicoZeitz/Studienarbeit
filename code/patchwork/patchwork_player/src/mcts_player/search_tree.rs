use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

use patchwork_core::Game;

use crate::{Evaluator, MCTSEndCondition, MCTSOptions, Node, TreePolicy};

pub trait MCTSSpecification: Clone {
    type Game: patchwork_core::Game;
}

// TreeParallelization (weg)
// Run MCTS while other player is thinking
// Tree reuse

pub struct SearchTree<
    'tree_lifetime,
    Spec: MCTSSpecification,
    Policy: TreePolicy,
    Eval: Evaluator<Game = Spec::Game>,
> {
    // The root node of the search tree.
    root_node: Arc<Mutex<Node<Spec>>>,
    /// The policy to select nodes during the selection phase.
    tree_policy: &'tree_lifetime Policy,
    evaluator: &'tree_lifetime Eval,
    options: &'tree_lifetime MCTSOptions,
}

impl<
        'tree_lifetime,
        Spec: MCTSSpecification,
        Policy: TreePolicy,
        Eval: Evaluator<Game = Spec::Game>,
    > SearchTree<'tree_lifetime, Spec, Policy, Eval>
{
    pub fn new(
        game: &Spec::Game,
        tree_policy: &'tree_lifetime Policy,
        evaluator: &'tree_lifetime Eval,
        options: &'tree_lifetime MCTSOptions,
    ) -> Self {
        let root_node = Arc::new(Mutex::new(Node::new(game, None, None)));

        SearchTree {
            root_node,
            tree_policy,
            evaluator,
            options,
        }
    }

    pub fn reset(self) -> Self {
        // TODO: real impl + other method for tree reuse
        SearchTree {
            root_node: self.root_node,
            tree_policy: self.tree_policy,
            evaluator: self.evaluator,
            options: self.options,
        }
    }

    pub fn search(&self) -> <Spec::Game as patchwork_core::Game>::Action {
        let root = Arc::clone(&self.root_node);

        // PERF: fastpath for when there is only one action
        if root.lock().unwrap().expandable_actions.len() == 1 {
            return root.lock().unwrap().expandable_actions[0].clone();
        }

        match &self.options {
            MCTSOptions {
                root_parallelization: 1,
                leaf_parallelization,
                end_condition,
            } => {
                match end_condition {
                    MCTSEndCondition::Iterations(iterations) =>
                    {
                        #[allow(unused_variables)]
                        for i in 0..*iterations {
                            #[cfg(debug_assertions)]
                            print!("\rMCTS Iteration {} ", i + 1);

                            SearchTree::playout(
                                Arc::clone(&root),
                                self.tree_policy,
                                self.evaluator,
                                *leaf_parallelization,
                            );
                        }
                    }
                    MCTSEndCondition::Time(time) => {
                        let start_time = std::time::Instant::now();
                        #[cfg(debug_assertions)]
                        let mut i = 0;
                        while std::time::Instant::now().duration_since(start_time) <= *time {
                            #[cfg(debug_assertions)]
                            print!("\rMCTS Iteration {} ", {
                                i += 1;
                                i
                            });

                            SearchTree::playout(
                                Arc::clone(&root),
                                self.tree_policy,
                                self.evaluator,
                                *leaf_parallelization,
                            );
                        }
                    }
                }

                let action = root
                    .lock()
                    .unwrap()
                    .children
                    .iter()
                    // TODO: allow generic implementation for selecting the best action
                    .max_by_key(|child| child.lock().unwrap().visit_count)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .action_taken
                    .clone()
                    .unwrap();
                action
            }
            // TODO: Zero for using all available cores.
            MCTSOptions {
                root_parallelization,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Iterations(iterations),
            } => {
                thread::scope(|s| {
                    let mut handles = Vec::with_capacity(*root_parallelization);
                    for _ in 0..*root_parallelization {
                        let root = Arc::new(Mutex::new(root.lock().unwrap().clone()));
                        let tree_policy = self.tree_policy;
                        let evaluator = self.evaluator;

                        handles.push(s.spawn(move || {
                            // while !stop.load(Ordering::SeqCst) {
                            for _ in 0..*iterations {
                                SearchTree::playout(
                                    Arc::clone(&root),
                                    tree_policy,
                                    evaluator,
                                    *leaf_parallelization,
                                );
                            }

                            let actions = root
                                .lock()
                                .unwrap()
                                .children
                                .iter()
                                .map(|child| {
                                    let child = child.lock().unwrap();
                                    (child.visit_count, child.action_taken.clone().unwrap())
                                })
                                .collect::<Vec<_>>();

                            let sum_of_visits =
                                actions.iter().map(|(visits, _)| visits).sum::<i32>() as f64;

                            actions
                                .iter()
                                .map(|(visits, action)| {
                                    (*visits as f64 / sum_of_visits, action.clone())
                                })
                                .collect::<Vec<_>>()
                        }));
                    }

                    let results = handles
                        .into_iter()
                        .map(|handle| handle.join().unwrap())
                        .collect::<Vec<_>>();

                    let actions = results[0]
                        .iter()
                        .map(|(_, action)| action)
                        .collect::<Vec<_>>();

                    let mut max_action = None;
                    let mut max_probability = 0.0;

                    for index in 0..results[0].len() {
                        let summed_probability =
                            results.iter().map(|result| result[index].0).sum::<f64>()
                                / actions.len() as f64;

                        if summed_probability > max_probability {
                            max_action = Some(actions[index].clone());
                            max_probability = summed_probability;
                        }
                    }

                    // TODO: re-attach root node
                    max_action.unwrap()
                })
            }
            MCTSOptions {
                root_parallelization,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Time(duration),
            } => {
                let stop = Arc::new(AtomicBool::new(false));

                thread::scope(|s| {
                    let mut handles = Vec::with_capacity(*root_parallelization);
                    for _ in 0..*root_parallelization {
                        let root = Arc::new(Mutex::new(root.lock().unwrap().clone()));
                        let tree_policy = self.tree_policy;
                        let evaluator = self.evaluator;
                        let stop = Arc::clone(&stop);

                        handles.push(s.spawn(move || {
                            // while !stop.load(Ordering::SeqCst) {
                            while !stop.load(Ordering::Acquire) {
                                SearchTree::playout(
                                    Arc::clone(&root),
                                    tree_policy,
                                    evaluator,
                                    *leaf_parallelization,
                                );
                            }

                            let actions = root
                                .lock()
                                .unwrap()
                                .children
                                .iter()
                                .map(|child| {
                                    let child = child.lock().unwrap();
                                    (child.visit_count, child.action_taken.clone().unwrap())
                                })
                                .collect::<Vec<_>>();

                            let sum_of_visits =
                                actions.iter().map(|(visits, _)| visits).sum::<i32>() as f64;

                            actions
                                .iter()
                                .map(|(visits, action)| {
                                    (*visits as f64 / sum_of_visits, action.clone())
                                })
                                .collect::<Vec<_>>()
                        }));
                    }

                    thread::sleep(*duration);
                    stop.store(true, Ordering::Release);

                    let results = handles
                        .into_iter()
                        .map(|handle| handle.join().unwrap())
                        .collect::<Vec<_>>();

                    let actions = results[0]
                        .iter()
                        .map(|(_, action)| action)
                        .collect::<Vec<_>>();

                    let mut max_action = None;
                    let mut max_probability = 0.0;

                    for index in 0..results[0].len() {
                        let summed_probability =
                            results.iter().map(|result| result[index].0).sum::<f64>()
                                / actions.len() as f64;

                        if summed_probability > max_probability {
                            max_action = Some(actions[index].clone());
                            max_probability = summed_probability;
                        }
                    }

                    // TODO: re-attach root node
                    max_action.unwrap()
                })
            }
        }
    }

    fn playout(
        root_node: Arc<Mutex<Node<Spec>>>,
        tree_policy: &Policy,
        evaluator: &Eval,
        leaf_parallelization: usize,
    ) {
        let mut node = root_node;

        // 1. Selection
        while Node::is_fully_expanded(&node) && !Node::is_terminal(&node) {
            node = Node::select(&node, tree_policy);
        }

        let value = if !&node.lock().unwrap().state.is_terminated() {
            // 2. Expansion
            let new_node = Node::expand(&node);
            node = new_node;

            // 3. Simulation
            Node::simulate(&node, evaluator, leaf_parallelization)
        } else {
            // 3. Leaf node -> direct evaluation
            evaluator.evaluate_terminal_node(&node)
        };

        // 4. Backpropagation
        Node::backpropagate(&node, value, evaluator);
    }

    fn internal_tree_to_string(&self, node: MutexGuard<'_, Node<Spec>>) -> Vec<String> {
        let mut result = Vec::new();

        result.push(format!("{:?}", node));

        for (index, child) in node.children.iter().enumerate() {
            let child = child.lock().unwrap();

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
                let front = if inner_index == 0 {
                    branching_front
                } else {
                    other_front
                };
                result.push(format!("{}{}", front, line));
            }
        }

        result
    }

    pub fn tree_to_string(&self) -> String {
        let root = self.root_node.lock().unwrap();
        self.internal_tree_to_string(root).join("\n")
    }
}
