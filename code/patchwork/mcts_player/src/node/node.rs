use std::sync::{Arc, RwLock, Weak};
use std::{fmt, thread};

use patchwork_core::{ActionId, Evaluator, Patchwork};
use rand::seq::SliceRandom;

use tree_policy::{TreePolicy, TreePolicyNode};

type Link = Arc<RwLock<Node>>;
type WeakLink = Weak<RwLock<Node>>;

#[derive(Clone)]
pub struct Node {
    /// The state of the game at this node.
    pub state: Patchwork,
    /// The parent node. None if this is the root node.
    pub parent: Option<WeakLink>,
    /// The children nodes.
    pub children: Vec<Link>,
    /// The maximum score of all the nodes in the subtree rooted at this node.
    pub max_score: i32,
    // The minimum score of all the nodes in the subtree rooted at this node.
    pub min_score: i32,
    // The sum of the scores of all the nodes in the subtree rooted at this node.
    pub score_sum: i32,
    // The number of times this node has been visited where the player whose turn it is to move
    pub wins: i32,
    // The number of times this node has been visited.
    pub visit_count: i32,
    /// The action that was taken to get to this node. None if this is the root node.
    pub action_taken: Option<ActionId>,
    /// The actions that can still be taken from this node
    pub expandable_actions: Vec<ActionId>,
}

impl Node {
    pub fn new(state: &Patchwork, parent: Option<WeakLink>, action_taken: Option<ActionId>) -> Self {
        let new_state = state.clone();
        let mut expandable_actions: Vec<ActionId> = new_state.get_valid_actions().into_iter().collect();
        expandable_actions.shuffle(&mut rand::thread_rng());

        Self {
            state: new_state,
            parent,
            children: vec![],
            max_score: i32::MIN,
            min_score: i32::MAX,
            wins: 0,
            score_sum: 0,
            visit_count: 0,
            action_taken,
            expandable_actions,
        }
    }

    pub fn is_fully_expanded(node: &Link) -> bool {
        Node::is_terminal(node) || Arc::clone(node).read().unwrap().expandable_actions.is_empty()
    }

    pub fn is_terminal(node: &Link) -> bool {
        Arc::clone(node).read().unwrap().state.is_terminated()
    }

    pub fn select<Policy: TreePolicy>(node: &Link, tree_policy: &Policy) -> Link {
        let cloned_parent = Arc::new(RwLock::new(node.read().unwrap().clone()));
        let parent = node.read().unwrap();
        let children = parent.children.iter().map(TreePolicyNodeWrapper);
        let selected_node = tree_policy.select_node(TreePolicyNodeWrapper(&cloned_parent), children);
        Arc::clone(selected_node.0)
    }

    ///  Expands this node by adding a child node.
    /// The child node is chosen randomly from the expandable actions.
    pub fn expand(node: &Link) -> Link {
        let action = node.write().unwrap().expandable_actions.remove(0);
        let mut next_state = node.read().unwrap().state.clone();
        next_state.do_action(action, false).unwrap();
        let child = Arc::new(RwLock::new(Node::new(
            &next_state,
            Some(Arc::downgrade(node)),
            Some(action),
        )));
        node.write().unwrap().children.push(Arc::clone(&child));
        child
    }

    /// Backpropagates the score of the game up until the parent node is reached.
    ///
    /// # Parameters
    ///
    /// - `node`: The node to backpropagate from.
    /// - `score`: The score at the end of the game that should be backpropagated.
    /// - `evaluator`: The evaluator to use to interpret the score.
    pub fn backpropagate(node: &Link, value: i32) {
        let mut mutable_node = node.write().unwrap();
        let player = &mutable_node.state.get_current_player();
        let maximizing_player = mutable_node.state.get_player_1_flag() == *player;
        let color = if maximizing_player { 1 } else { -1 };
        let score = color * value;

        mutable_node.visit_count += 1;
        mutable_node.score_sum += score;

        if score > 0 {
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
                Node::backpropagate(&parent_strong, value);
            }
        }
    }

    pub fn simulate<Eval: Evaluator>(node: &Link, evaluator: &Eval, leaf_parallelization: usize) -> i32 {
        let game = Arc::new({
            let node = node.read().unwrap();
            node.state.clone()
        });

        if Node::is_terminal(node) {
            return evaluator.evaluate_terminal_node(&game);
        }

        if leaf_parallelization == 1 {
            return evaluator.evaluate_intermediate_node(&game);
        }

        thread::scope(|s| {
            (0..leaf_parallelization)
                .map(|_| {
                    let game = Arc::clone(&game);
                    s.spawn(move || evaluator.evaluate_intermediate_node(&game))
                })
                .map(|handle| handle.join().unwrap())
                .sum()
        })
    }
}

struct TreePolicyNodeWrapper<'a>(&'a Link);

impl<'a> TreePolicyNode for TreePolicyNodeWrapper<'a> {
    fn max_score(&self) -> f64 {
        self.0.read().unwrap().max_score as f64
    }

    fn min_score(&self) -> f64 {
        self.0.read().unwrap().min_score as f64
    }

    fn score_sum(&self) -> f64 {
        self.0.read().unwrap().score_sum as f64
    }

    fn wins(&self) -> i32 {
        self.0.read().unwrap().wins
    }

    fn visit_count(&self) -> i32 {
        self.0.read().unwrap().visit_count
    }
}

// TODO: implement debug and so on for all structures when inner types support it as well
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Node")
            // .field("state", &self.state)
            .field("parent", &self.parent)
            // .field("children", &self.children)
            .field("max_score", &self.max_score)
            .field("min_score", &self.min_score)
            .field("score_sum", &self.score_sum)
            .field("wins", &self.wins)
            .field("visit_count", &self.visit_count)
            .field("action_taken", &self.action_taken) // TODO:
            // .field("expandable_actions", &self.expandable_actions)
            .finish()
    }
}
