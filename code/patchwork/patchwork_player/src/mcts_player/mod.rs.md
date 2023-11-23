# Mod.rs

<details>

```rs
pub use node::*;
pub use options::*;
use patchwork_core::{Action, Patchwork};
use patchwork_game::{Game, Player};

mod node;
mod options;
mod search_tree;
mod tree_policy;

/// A player that uses Monte Carlo Tree Search to choose an action.
#[derive(Debug, Clone, PartialEq)]
pub struct MCTSPlayer {
    pub name: String,
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
}

impl MCTSPlayer {
    /// Creates a new [`MCTSPlayer`].
    pub fn new(name: String, options: Option<MCTSOptions>) -> Self {
        MCTSPlayer {
            name,
            options: options.unwrap_or_else(|| MCTSOptions {
                number_of_simulations: 1000,
                c: 1.0,
            }),
        }
    }
}

impl Player for MCTSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork, state: &Patchwork) -> Action {
        let root = Node::root(state.clone(), self.options.clone());

        {
            // PERF: fastpath for when there is only one action
            let mut mutable_root = root.borrow_mut();
            if mutable_root.expandable_actions().len() == 1 {
                return mutable_root.first_action();
            }
        }

        for _ in 0..self.options.number_of_simulations {
            let mut node = root;
            let mut mutable_node = node.borrow();

            // 1. Selection
            while mutable_node.is_fully_expanded() && !mutable_node.is_terminal() {
                let selected_node = mutable_node.select();
                mutable_node = selected_node.borrow();
            }

            let value = if !game.is_terminated(&node.borrow().state) {
                // 2. Expansion
                node = Node::expand(node);
                // 3. Simulation
                node.borrow().simulate()
            } else {
                game.get_termination_result(&node.borrow().state).score() as f64
            };

            // 4. Backpropagation
            node.borrow()
                .backpropagate((node.borrow().state.current_player_flag as f64) * value)
        }

        return root
            .borrow()
            .children
            .iter()
            .max_by_key(|child| child.borrow().visit_count)
            .unwrap()
            .borrow()
            .action_taken
            .unwrap();
    }
}
```

</details>

# Game Specification.rs

<details>

```rs
pub trait GameSpecification {
    type Game: GameState;
    type TreePolicy;
}
```

</details>

# Tree Policy

<details>

```rs
pub trait TreePolicy {
    fn select_node(&self, current_node: &Node)
}

pub struct ActionInformation {
pub visits: AtomicUsize,
}
pub struct MoveInfo<Spec: MCTS> {
mov: Move<Spec>,
move_evaluation: MoveEvaluation<Spec>,
child: AtomicPtr<SearchNode<Spec>>,
owned: AtomicBool,
stats: NodeStats,
}

pub struct SearchNode<Spec: MCTS> {
moves: Vec<MoveInfo<Spec>>,
data: Spec::NodeData,
evaln: StateEvaluation<Spec>,
// stats
visits: AtomicUsize,
sum_evaluations: AtomicI64,
}

```

</details>

<details>

```rs
use super::NodeRef;

pub struct UCTPolicy {
pub c: f64,
}

impl UCTPolicy {
pub fn new(c: f64) -> Self {
UCTPolicy { c }
}
}
pub trait TreePolicy {
fn choose_child(&self, node: &Vec<NodeRef>) -> NodeRef;
}

impl TreePolicy for UCTPolicy {
fn choose_child(&self, node: &Vec<NodeRef>) -> NodeRef {
todo!()
}
}

```

</details>

# Untitled.rs

<details>

```rs
pub trait GameEvaluator<Spec: GameSpecification<Evaluator=Self>> {
    type Evaluation;

    fn evaluate_intermediate_state(&self, state: &Spec::State, player: &Spec::Player) -> &Self::Evaluation;
    fn evaluate_terminal_state(&self, state: &Spec::State, player: &Spec::Player) -> &Self::Evaluation;

}

pub trait TreePolicy<Spec: GameSpecification<TreePolicy=Self>> {
fn select_node(&self, root: &Node<Spec>) -> Node<Spec>;
}

pub trait GameSpecification {
type Game : Game<State=Self::State, Action=Self::Action, Player=Self::Player>;
type State: Clone;
type Action: Clone;
type Player;
type Evaluator: GameEvaluator<Self>;
type TreePolicy: TreePolicy<Self>;
}

pub trait StateEvaluator<State, Player> {
type Evaluation: Sync + Send;

    /// Evaluates the given state for the given
    fn evaluate_tree_node(&self, state: &State, player: &Player) -> &Self::Evaluation;
    fn evaluate_leaf_node(&self, state: &State, player: &Player) -> &Self::Evaluation;

}

pub trait MCTSSpecification {
/// The game that is being played.
type Game : MCTSGame<State, Action, Player>;
/// The state of the game.
type State: Sync;
/// The type of actions that can be taken.
type Action: Sync + Send + Clone;
/// The type of a player.
type Player;

    /// A policy that is used to select the next node to expand.
    type TreePolicy;

    type Evaluation;
    type Evaluator;

}

pub struct SearchTree<Spec: MCTSSpecification> {
pub options: MCTSOptions,
pub root: Node<Spec>,
}

pub struct Node<Spec: MCTSSpecification> {
pub state: State,
pub children: Vec<Node>,
pub parent: Option<Node>,

    pub visit_count: u32,
    pub visits: u32,
    pub wins: u32,

}

searchTree<Stuff> {
root: Node

    search() / playout()
    -> times
    -> amout
    -> threaded (leaf, tree, ...)

}

stuff {
state,
}

game {
availabe_moves
make_move
current_player
}

interpret_evaluation_for_player

node {
select(treepolicy)
expand()
simulate(evaluator(state)) -> value
backpropagate()
}

policy

tree = searchTree::new()
tree.search(state)

```

</details>

# Search Tree.rs

<details>

```rs
// pub trait TreePolicy {
//     fn choose_child(&self, )
// }

// pub trait MCTS {
// type TreePolicy: TreePolicy;
// }

pub struct SearchTree {}

impl SearchTree {
pub fn new() -> Self {
SearchTree {}
}

    pub fn search(&mut self, state: &State) -> Action {
        let root = Node::root(state.clone());
    }

}

// pub struct SearchTree<Spec: MCTS> {
// // selection
// // expansion
// // simulation
// // backpropagation
// }

```

</details>

# Node

<details>

```rs
use core::panic;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use patchwork_core::{Action, Patchwork};
use patchwork_game::Game;
use rand::seq::SliceRandom;

use super::MCTSOptions;

pub type NodeRef = Rc<RefCell<Node>>;

/// A node in the Monte Carlo Tree Search algorithm.
#[derive(Debug, Clone)]
pub struct Node {
    /// The state of the game.
    pub state: Patchwork,
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The parent node. None if this is the root node.
    pub parent: Option<Weak<RefCell<Node>>>,
    /// The children nodes.
    pub children: Vec<NodeRef>,
    /// The maximum score of all the nodes in the subtree rooted at this node.
    pub max_score: f64,
    // The minimum score of all the nodes in the subtree rooted at this node.
    pub min_score: f64,
    // The sum of the scores of all the nodes in the subtree rooted at this node.
    pub score_sum: f64,
    // The number of times this node has been visited.
    pub visit_count: i32,
    /// The action that was taken to get to this node.
    pub action_taken: Option<Action>,
    expandable_actions: Option<Vec<Action>>,
}

impl Node {
    pub fn root(state: Patchwork, options: MCTSOptions) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node {
            state,
            options,
            expandable_actions: None,
            children: Vec::new(),
            parent: None,
            max_score: f64::NEG_INFINITY,
            min_score: f64::INFINITY,
            score_sum: 0f64,
            visit_count: 0,
            action_taken: None,
        }))
    }

    pub fn child(
        state: Patchwork,
        options: MCTSOptions,
        parent: Weak<RefCell<Node>>,
        action_taken: Action,
    ) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node {
            state,
            options,
            expandable_actions: None,
            children: Vec::new(),
            parent: Some(parent),
            max_score: f64::NEG_INFINITY,
            min_score: f64::INFINITY,
            score_sum: 0f64,
            visit_count: 0,
            action_taken: Some(action_taken),
        }))
    }

    /// The moves that can be expanded.
    pub fn expandable_actions(&mut self) -> &'_ Vec<Action> {
        let mut actions = Patchwork::get_valid_actions(&self.state);
        actions.shuffle(&mut rand::thread_rng());
        self.expandable_actions = Some(actions);
        self.expandable_actions.as_ref().unwrap()
    }

    pub fn first_action(&self) -> Action {
        self.expandable_actions().first().unwrap().clone()
    }

    fn ensure_expandable_actions(&mut self) {
        if self.expandable_actions.is_none() {
            self.expandable_actions();
        }
    }

    /// Checks whether this node is fully expanded.
    /// A node is fully expanded if all its children are expanded or if it is a terminal node.
    pub fn is_fully_expanded(&self) -> bool {
        self.is_terminal() || self.expandable_actions().is_empty()
    }

    /// Checks whether this node is a terminal node.
    /// A node is a terminal node if the game is over.
    pub fn is_terminal(&self) -> bool {
        Patchwork::is_terminated(&self.state)
    }

    /// Selects the child node with the highest upper confidence bound.
    ///
    /// # Returns
    ///
    /// - `Node`: The child node with the highest upper confidence bound.
    pub fn select(&self) -> Rc<RefCell<Node>> {
        Rc::clone(
            self.children
                .iter()
                .map(|child| (child, self.get_upper_confidence_bound(child)))
                .max_by(|(_, ucb1), (_, ucb2)| ucb1.partial_cmp(ucb2).unwrap())
                .map(|(child, _)| child)
                .unwrap(),
        )
    }

    /// Calculates the upper confidence bound of this node.
    /// Formula: Q(s, a) + C * sqrt(ln(N(s)) / N(s, a))
    /// Where:
    ///     Q(s, a) is the average value of the node
    ///     C is the exploration parameter
    ///     N(s) is the visit count of the node
    ///     N(s, a) is the visit count of the child node
    ///
    /// # Parameters
    ///
    /// - `child`: The child node.
    ///
    /// # Returns
    ///
    /// - `f64`: The upper confidence bound of this node.
    pub fn get_upper_confidence_bound(&self, child: &Rc<RefCell<Node>>) -> f64 {
        let child = (*child).borrow();

        let child_visit_count = child.visit_count as f64;
        let score_range = self.max_score - self.min_score;

        let exploitation_multiplier = self.state.current_player_flag as f64;
        let exploitation = exploitation_multiplier * child.score_sum / child_visit_count;

        let exploration = self.options.c
            * score_range
            * ((self.visit_count as f64).ln() / child_visit_count).sqrt();

        exploitation + exploration
    }

    ///  Expands this node by adding a child node.
    /// The child node is chosen randomly from the expandable actions.
    pub fn expand(_self: Rc<RefCell<Self>>) -> Rc<RefCell<Node>> {
        let mut this = _self.borrow_mut();
        this.ensure_expandable_actions();
        let action = this.expandable_actions.unwrap().remove(0);
        let next_state = Patchwork::get_next_state(&this.state, &action);
        let child = Node::child(next_state, this.options, Rc::downgrade(&_self), action);
        this.children.push(Rc::clone(&child));
        child
    }

    ///  Simulates a game from this node until the end.
    ///
    /// # Returns
    ///
    /// - `f64`: The score of the game.
    pub fn simulate(&mut self) -> f64 {
        if self.is_terminal() {
            return Patchwork::get_score(&self.state, self.state.current_player_flag) as f64;
        }

        self.ensure_expandable_actions();

        let mut rollout_state = self.state;
        let valid_actions = self.expandable_actions.unwrap();
        let mut action = valid_actions[0];

        loop {
            rollout_state = Patchwork::get_next_state(&rollout_state, &action);

            if Patchwork::is_terminated(&rollout_state) {
                return Patchwork::get_score(&rollout_state, self.state.current_player_flag) as f64;
            }

            action = Patchwork::sample_random_action(&rollout_state);
        }
    }

    /// Backpropagates the score of the game to the parent nodes.
    ///
    /// # Parameters
    ///
    /// - `score`: The score at the end of the game that should be backpropagated.
    pub fn backpropagate(&mut self, score: f64) {
        let mut score = score;
        self.score_sum += score;
        self.visit_count += 1;

        match score.partial_cmp(&self.max_score) {
            Some(ordering) => match ordering {
                std::cmp::Ordering::Greater => self.max_score = score,
                std::cmp::Ordering::Less => self.min_score = score,
                std::cmp::Ordering::Equal => {}
            },
            None => panic!("[Node][backpropagate] Score is NaN"),
        }

        if let Some(parent) = self.parent {
            if let Some(parent) = parent.upgrade() {
                let mut parent = (*parent).borrow_mut();

                if parent.state.current_player_flag != self.state.current_player_flag {
                    score = -score;
                }
                parent.backpropagate(score);
            }
        }
    }
}
```

</details>
````
