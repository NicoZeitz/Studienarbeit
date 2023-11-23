use std::{cell::RefCell, rc::Rc};

use patchwork_core::Game;

use crate::{Evaluator, MCTSEndCondition, MCTSOptions, Node, TreePolicy, TreePolicyNode};

pub trait MCTSSpecification {
    type Game: patchwork_core::Game;
}

pub struct SearchTree<
    'tree_lifetime,
    Spec: MCTSSpecification,
    Policy: TreePolicy,
    Eval: Evaluator<Game = Spec::Game>,
> {
    // The root node of the search tree.
    root_node: Rc<RefCell<Node<Spec>>>,
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
        let root_node = Rc::new(RefCell::new(Node::new(game, None, None)));

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
        // PERF: fastpath for when there is only one action
        if RefCell::borrow(&self.root_node).expandable_actions.len() == 1 {
            return RefCell::borrow(&self.root_node).expandable_actions[0].clone();
        }

        match &self.options {
            MCTSOptions {
                root_parallelization: 1,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Iterations(n),
            } =>
            {
                #[allow(unused_variables)]
                for i in 0..*n {
                    #[cfg(debug_assertions)]
                    print!("\rMCTS Iteration {}", i + 1);
                    self.playout(*leaf_parallelization);
                }
            }
            MCTSOptions {
                root_parallelization: 1,
                leaf_parallelization,
                end_condition: MCTSEndCondition::Time(time),
            } => {
                let start_time = std::time::Instant::now();
                #[cfg(debug_assertions)]
                let mut i = 0;
                while std::time::Instant::now().duration_since(start_time) <= *time {
                    #[cfg(debug_assertions)]
                    print!("\rMCTS Iteration {} ", {
                        i += 1;
                        i
                    });
                    self.playout(*leaf_parallelization);
                }
            }
            // TODO: Zero for using all available cores.
            MCTSOptions {
                root_parallelization: _, // TODO: root_parallelization,
                leaf_parallelization: _, // TODO: leaf_parallelization,
                end_condition: MCTSEndCondition::Iterations(_n),
            } => {
                todo!()
            }
            MCTSOptions {
                root_parallelization: _, // TODO: root_parallelization,
                leaf_parallelization: _, // TODO: leaf_parallelization,
                end_condition: MCTSEndCondition::Time(_time),
            } => {
                todo!()
            }
        }

        let root = RefCell::borrow(&self.root_node);
        return RefCell::borrow(
            root.children
                .iter()
                // TODO: allow generic implementation
                .max_by_key(|child| child.visit_count())
                .unwrap(),
        )
        .action_taken
        .clone()
        .unwrap();
    }

    fn playout(&self, leaf_parallelization: usize) {
        let mut node = Rc::clone(&self.root_node);

        // 1. Selection
        while Node::is_fully_expanded(&node) && !Node::is_terminal(&node) {
            node = Node::select(&node, self.tree_policy);
        }

        let value = if !RefCell::borrow(&node).state.is_terminated() {
            // 2. Expansion
            let new_node = Node::expand(&node);
            node = new_node;

            // 3. Simulation
            Node::simulate(&node, self.evaluator, leaf_parallelization)
        } else {
            // 3. Leaf node -> direct evaluation
            self.evaluator.evaluate_terminal_node(&node)
        };
        // 4. Backpropagation
        Node::backpropagate(&node, value, self.evaluator);
    }
}
