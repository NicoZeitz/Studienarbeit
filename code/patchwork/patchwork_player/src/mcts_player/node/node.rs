use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

use patchwork_core::Game;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::mcts_player::search_tree::MCTSSpecification;
use crate::{EvaluationNode, Evaluator, TreePolicy, TreePolicyNode};

type Link<T> = Rc<RefCell<Node<T>>>;
type WeakLink<T> = Weak<RefCell<Node<T>>>;

#[derive(Clone)]
pub struct Node<Spec: MCTSSpecification> {
    /// The state of the game at this node.
    pub state: Spec::Game,
    /// The parent node. None if this is the root node.
    pub parent: Option<WeakLink<Spec>>,
    /// The children nodes.
    pub children: Vec<Link<Spec>>,
    /// The maximum score of all the nodes in the subtree rooted at this node.
    pub max_score: f64,
    // The minimum score of all the nodes in the subtree rooted at this node.
    pub min_score: f64,
    // The sum of the scores of all the nodes in the subtree rooted at this node.
    pub score_sum: f64,
    // The number of times this node has been visited where the player whose turn it is to move
    pub wins: i32,
    // The number of times this node has been visited.
    pub visit_count: i32,
    /// The action that was taken to get to this node. None if this is the root node.
    pub action_taken: Option<<Spec::Game as patchwork_core::Game>::Action>,
    /// The actions that can still be taken from this node
    pub expandable_actions: Vec<<Spec::Game as patchwork_core::Game>::Action>,
}

impl<Spec: MCTSSpecification> Node<Spec> {
    pub fn new(
        state: &Spec::Game,
        parent: Option<WeakLink<Spec>>,
        action_taken: Option<<Spec::Game as patchwork_core::Game>::Action>,
    ) -> Self {
        let new_state = state.clone();
        let mut expandable_actions: Vec<<Spec::Game as patchwork_core::Game>::Action> =
            new_state.get_valid_actions().into_iter().collect();
        expandable_actions.shuffle(&mut rand::thread_rng());

        Self {
            state: new_state,
            parent,
            children: vec![],
            max_score: f64::NEG_INFINITY,
            min_score: f64::INFINITY,
            wins: 0,
            score_sum: 0.0,
            visit_count: 0,
            action_taken,
            expandable_actions,
        }
    }

    pub fn is_fully_expanded(node: &Link<Spec>) -> bool {
        Node::is_terminal(node) || RefCell::borrow(node).expandable_actions.is_empty()
    }

    pub fn is_terminal(node: &Link<Spec>) -> bool {
        RefCell::borrow(node).state.is_terminated()
    }

    pub fn select<Policy: TreePolicy>(node: &Link<Spec>, tree_policy: &Policy) -> Link<Spec> {
        Rc::clone(tree_policy.select_node(RefCell::borrow(node).children.iter()))
    }

    ///  Expands this node by adding a child node.
    /// The child node is chosen randomly from the expandable actions.
    pub fn expand(node: &Link<Spec>) -> Link<Spec> {
        let action = RefCell::borrow_mut(node).expandable_actions.remove(0);
        let next_state = RefCell::borrow(node).state.get_next_state(&action);
        let child = Rc::new(RefCell::new(Node::new(
            &next_state,
            Some(Rc::downgrade(node)),
            Some(action),
        )));
        RefCell::borrow_mut(node).children.push(Rc::clone(&child));
        child
    }

    /// Backpropagates the score of the game up until the parent node is reached.
    ///
    /// # Parameters
    ///
    /// - `node`: The node to backpropagate from.
    /// - `score`: The score at the end of the game that should be backpropagated.
    /// - `evaluator`: The evaluator to use to interpret the score.
    pub fn backpropagate<Eval: Evaluator<Game = Spec::Game>>(
        node: &Link<Spec>,
        value: Eval::Evaluation,
        evaluator: &Eval,
    ) {
        let mut mutable_node = RefCell::borrow_mut(node);

        let state = &mutable_node.state;
        let player = &mutable_node.state.get_current_player();
        let score: f64 = evaluator
            .interpret_evaluation_for_player(state, player, &value)
            .into();

        mutable_node.visit_count += 1;
        mutable_node.score_sum += score;

        // TODO: correct for generic implementation?
        if score > 0.0 {
            mutable_node.wins += 1;
        }
        if score < mutable_node.min_score {
            mutable_node.min_score = score;
        }
        if score > mutable_node.max_score {
            mutable_node.max_score = score;
        }

        if let Some(parent) = &mutable_node.parent {
            if let Some(parent_strong) = parent.upgrade() {
                Node::backpropagate(&parent_strong, value, evaluator);
            }
        }
    }

    pub fn simulate<Eval: Evaluator<Game = Spec::Game>>(
        node: &Link<Spec>,
        evaluator: &Eval,
        _leaf_parallelization: usize, // TODO: leaf_parallelization
    ) -> Eval::Evaluation {
        evaluator.evaluate_intermediate_node(node)
    }

    pub fn random_rollout(node: &Link<Spec>) -> &Link<Spec> {
        if Node::is_terminal(node) {
            return node;
        }

        let mut rollout_state = RefCell::borrow(node).state.clone(); // TODO: can we not clone here?
        let valid_actions: Vec<_> = rollout_state.get_valid_actions().into_iter().collect();
        let index = rand::thread_rng().gen_range(0..valid_actions.len());
        let mut action = valid_actions[index].clone();

        loop {
            rollout_state = rollout_state.get_next_state(&action);

            if rollout_state.is_terminated() {
                return node;
            }

            action = rollout_state.get_random_action();
        }
    }
}

impl<Spec: MCTSSpecification> TreePolicyNode for &Link<Spec> {
    fn max_score(&self) -> f64 {
        RefCell::borrow(self).max_score
    }

    fn min_score(&self) -> f64 {
        RefCell::borrow(self).min_score
    }

    fn score_sum(&self) -> f64 {
        RefCell::borrow(self).score_sum
    }

    fn wins(&self) -> i32 {
        RefCell::borrow(self).wins
    }

    fn visit_count(&self) -> i32 {
        RefCell::borrow(self).visit_count
    }

    fn parent_visit_count(&self) -> i32 {
        RefCell::borrow(self)
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .map(|parent| RefCell::borrow(&parent).visit_count)
            .unwrap_or(0)
    }
}

impl<Spec: MCTSSpecification> EvaluationNode for &Link<Spec> {
    type Game = Spec::Game;

    fn game(&self) -> Self::Game {
        let state = RefCell::borrow(self).state.clone(); // TODO: can we not clone here?
        state
    }

    fn random_rollout(&self) -> Self {
        Node::random_rollout(self)
    }
}

impl<Spec: MCTSSpecification> fmt::Debug for Node<Spec> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Node")
            // .field("state", &self.state)
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("max_score", &self.max_score)
            .field("min_score", &self.min_score)
            .field("score_sum", &self.score_sum)
            .field("wins", &self.wins)
            .field("visit_count", &self.visit_count)
            // .field("action_taken", &self.action_taken) // TODO:
            // .field("expandable_actions", &self.expandable_actions)
            .finish()
    }
}
// impl<Spec: MCTSSpecification + fmt::Display> fmt::Display for Node<Spec> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         fmt::Display::fmt(&*self.borrow(), f)
//     }
// }

// impl<Spec: MCTSSpecification> Node<Spec> {
//     /// Detaches a node from its parent and siblings. Children are not affected.
//     pub fn detach(node: &Node<Spec>) {
//         let mut borrowed_node = RefCell::borrow_mut(&node.0);
//         let parent_weak = borrowed_node.parent.take();

//         if let Some(parent_ref) = parent_weak.as_ref() {
//             if let Some(parent_strong) = parent_ref.upgrade() {
//                 let mut parent_borrow = RefCell::borrow_mut(&parent_strong);

//                 let index_in_parent = parent_borrow
//                     .children
//                     .iter()
//                     .position(|child| Rc::ptr_eq(&child.0, &node.0))
//                     .expect("Node is not a child of its parent");

//                 parent_borrow.children.remove(index_in_parent);
//             }
//         }
//     }
