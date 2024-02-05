use std::{cell::RefCell, num::NonZeroUsize, ops::Sub, rc::Rc, thread};

use evaluator::ScoreEvaluator;
use patchwork_core::{ActionId, Diagnostics, Evaluator, Patchwork, Player, PlayerResult, TreePolicy, TreePolicyNode};
use tree_policy::ScoredUCTPolicy;

pub(crate) const NON_ZERO_USIZE_ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
pub(crate) const NON_ZERO_USIZE_FOUR: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(4) };

use crate::{MCTSEndCondition, MCTSOptions, Node, SearchTree};

// TODO:
// print tree to verbose
// allow tree reuse
// allow root parallelization

/// A computer player that uses the Monte Carlo Tree Search (MCTS) algorithm to choose an action.
pub struct MCTSPlayer<Policy: TreePolicy = ScoredUCTPolicy, Eval: Evaluator = ScoreEvaluator> {
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The name of the player.
    pub name: String,
    /// The policy to select nodes during the selection phase.
    pub policy: Policy,
    /// The evaluator to evaluate the game state.
    pub evaluator: Eval,
    /// The root nodes where the last search was started from. Used to reuse the tree.
    last_roots: Vec<Rc<RefCell<Node>>>,
}

struct NodeWrapper {
    node: Option<Rc<RefCell<Node>>>,
}

/// SAFETY: Node cannot be send as it contains an Rc. This wrapper is used to
/// allow sending a Node to another thread. We only modify the Rc's in node while
/// doing mcts. During the search the node is fully owned by a search thread.
/// This wrapper only allows sending the complete tree back to the main
/// search-tree thread, where it is parked (and not modified) until another
/// search is started and the wrapper is used again to send the node to the mcts
/// search thread.
/// I hereby promise that I *know* Node is in a state where it is safe when
/// wrapping it into a node wrapper.
unsafe impl Send for NodeWrapper {}

impl<Policy: TreePolicy + Default, Eval: Evaluator + Default> MCTSPlayer<Policy, Eval> {
    /// Creates a new [`MCTSPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<MCTSOptions>) -> Self {
        let options = options.unwrap_or_default();
        let last_roots = if options.reuse_tree {
            Vec::with_capacity(options.root_parallelization.get())
        } else {
            Vec::new()
        };

        MCTSPlayer {
            name: format!(
                "{} (R: {}, L: {})",
                name.into(),
                options.root_parallelization,
                options.leaf_parallelization
            ),
            policy: Default::default(),
            evaluator: Default::default(),
            options,
            last_roots,
        }
    }
}

impl<Policy: TreePolicy + Default, Eval: Evaluator + Default> Default for MCTSPlayer<Policy, Eval> {
    fn default() -> Self {
        Self::new("MCTS Player".to_string(), Default::default())
    }
}

macro_rules! play_until_end_worker_thread {
    ($end_condition:expr, $playout:expr) => {
        match $end_condition {
            MCTSEndCondition::Iterations(iterations) => {
                let mut iteration = 0;
                loop {
                    if iteration == iterations {
                        break;
                    }

                    $playout;

                    iteration += 1;
                }
            }
            MCTSEndCondition::Time(time_limit) => {
                // add safety margin to time limit
                let time_limit = time_limit.sub(std::time::Duration::from_millis(50));
                let start_time = std::time::Instant::now();
                let mut time_passed = std::time::Duration::from_secs(0);
                loop {
                    if time_passed >= time_limit {
                        break;
                    }

                    $playout;

                    time_passed = std::time::Instant::now().duration_since(start_time);
                }
            }
        }
    };
}

macro_rules! play_until_end {
    ($end_condition:expr, $playout:expr, $diagnostics:expr) => {
        let mut iteration = 0;
        let mut time_passed = std::time::Duration::from_secs(0);
        let start_time = std::time::Instant::now();

        match $end_condition {
            MCTSEndCondition::Iterations(iterations) => {
                loop {
                    if iteration == *iterations {
                        break;
                    }

                    $playout;

                    iteration += 1;
                    time_passed = std::time::Instant::now().duration_since(start_time);

                    // Write diagnostics every 1000 iterations
                    if iteration % 1000 == 0 {
                        #[allow(clippy::redundant_closure_call)]
                        $diagnostics(iteration, time_passed)?;
                    }
                }

                #[allow(clippy::redundant_closure_call)]
                $diagnostics(iteration, time_passed)?;
            }
            MCTSEndCondition::Time(time_limit) => {
                // add safety margin to time limit
                let time_limit = time_limit.sub(std::time::Duration::from_millis(50));
                let mut last_print = std::time::Instant::now();
                loop {
                    if time_passed >= time_limit {
                        break;
                    }

                    $playout;

                    iteration += 1;
                    time_passed = std::time::Instant::now().duration_since(start_time);

                    // Write diagnostics every 10 seconds
                    if last_print.elapsed() >= std::time::Duration::from_secs(1) {
                        #[allow(clippy::redundant_closure_call)]
                        $diagnostics(iteration, time_passed)?;

                        last_print = std::time::Instant::now();
                    }
                }

                #[allow(clippy::redundant_closure_call)]
                $diagnostics(iteration, time_passed)?;
            }
        }
    };
}

impl<Policy: TreePolicy, Eval: Evaluator> Player for MCTSPlayer<Policy, Eval> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        // PERF: shortcut for only one available action
        let valid_action = game.get_valid_actions();
        if valid_action.len() == 1 {
            return Ok(valid_action[0]);
        }

        Ok(match &mut self.options {
            MCTSOptions {
                root_parallelization: NON_ZERO_USIZE_ONE,
                leaf_parallelization,
                end_condition,
                reuse_tree,
                diagnostics,
            } => {
                let last_root = if !self.last_roots.is_empty() {
                    Some(self.last_roots.swap_remove(0))
                } else {
                    None
                };

                let mut search_tree =
                    SearchTree::<Policy, Eval>::from_root(last_root, game, &self.policy, &self.evaluator);

                play_until_end!(
                    end_condition,
                    {
                        search_tree.playout(*leaf_parallelization)?;
                    },
                    |iteration, time_passed| { write_diagnostics(diagnostics, iteration, time_passed, &search_tree) }
                );

                write_verbose_diagnostics(diagnostics, &search_tree)?;

                let action = pick_best_action(&search_tree);

                if *reuse_tree {
                    self.last_roots = vec![get_tree_for_reuse(action, search_tree.root)]
                } else {
                    drop(search_tree);
                }

                action
            }
            MCTSOptions {
                root_parallelization,
                leaf_parallelization,
                end_condition,
                reuse_tree,
                diagnostics,
            } => {
                let roots = thread::scope::<'_, _, PlayerResult<Vec<Rc<RefCell<Node>>>>>(|s| {
                    let root_parallelization = (*root_parallelization).get();
                    let mut handles: Vec<thread::ScopedJoinHandle<'_, PlayerResult<NodeWrapper>>> =
                        Vec::with_capacity(root_parallelization);

                    for _ in 0..(root_parallelization - 1) {
                        // check for len > 2 to always keep at least the first root for the main search thread
                        let last_root = if self.last_roots.len() > 2 {
                            Some(self.last_roots.remove(self.last_roots.len() - 1))
                        } else {
                            None
                        };

                        let wrapper = NodeWrapper { node: last_root };
                        let evaluator = &self.evaluator;
                        let policy = &self.policy;
                        let leaf_parallel = *leaf_parallelization;
                        let end_cond = end_condition.clone();

                        // start worker search thread
                        handles.push(s.spawn(move || {
                            let wrapper = wrapper;
                            let last_root = wrapper.node;

                            let mut search_tree =
                                SearchTree::<Policy, Eval>::from_root(last_root, game, policy, evaluator);

                            play_until_end_worker_thread!(end_cond, {
                                search_tree.playout(leaf_parallel)?;
                            });

                            let wrapper = NodeWrapper {
                                node: Some(search_tree.root),
                            };
                            Ok(wrapper)
                        }));
                    }

                    let last_root = if !self.last_roots.is_empty() {
                        Some(self.last_roots.swap_remove(0))
                    } else {
                        None
                    };

                    let mut search_tree =
                        SearchTree::<Policy, Eval>::from_root(last_root, game, &self.policy, &self.evaluator);

                    play_until_end!(
                        end_condition,
                        {
                            search_tree.playout(*leaf_parallelization)?;
                        },
                        |iteration, time_passed| {
                            write_diagnostics(diagnostics, iteration, time_passed, &search_tree)
                        }
                    );

                    let mut roots = vec![Rc::clone(&search_tree.root)];

                    for handle in handles {
                        match handle.join() {
                            // safe to unwrap as the thread always puts the root into the wrapper before exiting
                            Ok(Ok(wrapper)) => roots.push(wrapper.node.unwrap()),
                            Err(error) => {
                                write_worker_error(
                                    diagnostics,
                                    format!("[MCTS] Error in worker thread: {:?}", error).as_str(),
                                )?;
                                continue; // Work with data from other threads
                            }
                            Ok(Err(error)) => {
                                if let Some(error) = error.downcast_ref::<String>() {
                                    write_worker_error(
                                        diagnostics,
                                        format!("[MCTS] Error in worker thread: {}", error).as_str(),
                                    )?;
                                } else {
                                    write_worker_error(
                                        diagnostics,
                                        format!("[MCTS] Error in worker thread: {:?}", error).as_str(),
                                    )?;
                                }
                                continue; // Work with data from other threads
                            }
                        }
                    }

                    write_verbose_diagnostics(diagnostics, &search_tree)?;
                    Ok(roots)
                })?;

                let action = pick_best_action_from_multiple(&roots);

                if *reuse_tree {
                    self.last_roots = roots.into_iter().map(|root| get_tree_for_reuse(action, root)).collect();
                }

                action
            }
        })
    }
}

/// Picks the best action from the root node.
/// This is done by selecting the child node with the highest number of visits.
/// If there are multiple child nodes with the same number of visits, the action with the
/// greater amount of wins is chosen. If there are still multiple actions with the same amount
/// of wins, one of them is chosen randomly.
///
/// # Arguments
///
/// * `search_tree` - The search tree to pick the best action from.
///
/// # Returns
///
/// The best action from the root node.
pub fn pick_best_action(search_tree: &SearchTree<impl TreePolicy, impl Evaluator>) -> ActionId {
    let root = RefCell::borrow(&search_tree.root);
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

/// Picks the best action from the root nodes of multiple trees.
/// This is done by merging all the root nodes into one and then selecting the child node with the
/// highest number of visits. If there are multiple child nodes with the same number of visits, the
/// action with the greater amount of wins is chosen. If there are still multiple actions with the
/// same amount of wins, one of them is chosen randomly.
///
/// # Arguments
///
/// * `nodes` - The root nodes to pick the best action from.
///
/// # Returns
///
/// The best action from the root nodes.
///
/// # Complexity
///
/// `𝒪(𝑚 · 𝑛)` where `𝑚` is the number of nodes and `𝑛` is the number of children of each root node.
pub fn pick_best_action_from_multiple(nodes: &[Rc<RefCell<Node>>]) -> ActionId {
    let mut action_map = std::collections::HashMap::new();

    for root in nodes {
        for child in RefCell::borrow(root).children.iter() {
            let child = RefCell::borrow(child);
            if let Some(action) = child.action_taken {
                let entry = action_map.entry(action).or_insert((0, 0));
                entry.0 += child.visit_count;
                entry.1 += child.wins_for(child.state.is_player_1());
            }
        }
    }

    *action_map
        .iter()
        .max_by_key(|(_, (visits, wins))| (*visits, *wins))
        .unwrap()
        .0
}

/// Gets the tree to reuse for the given action.
/// Searches the children of the given root node (only depth 1) for having taken the given action to
/// arrive at the child node. If the action was taken, the child node is returned.
/// If no child with the given action was found, the root node is returned.
///
/// # Arguments
///
/// * `action` - The action to search for.
/// * `root` - The root node to search in.
///
/// # Returns
///
/// The child node with the given action or the root node if no child with the given action was found.
///
/// # Complexity
///
/// `𝒪(𝑛)` where `𝑛` is the number of children of the root node.
fn get_tree_for_reuse(action: ActionId, root: Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
    // default to current
    let mut new_root = Rc::clone(&root);

    for child in RefCell::borrow(&root).children.iter() {
        if let Some(action_taken) = RefCell::borrow(child).action_taken {
            if action_taken == action {
                new_root = Rc::clone(child);
                break;
            }
        }
    }

    new_root
}

fn write_diagnostics(
    diagnostics: &mut Diagnostics,
    iteration: usize,
    time_passed: std::time::Duration,
    search_tree: &SearchTree<impl TreePolicy, impl Evaluator>,
) -> Result<(), std::io::Error> {
    #[rustfmt::skip]
    match diagnostics {
        Diagnostics::Disabled | Diagnostics::VerboseOnly { .. } => {}
        Diagnostics::Enabled {
            progress_writer: writer,
        }
        | Diagnostics::Verbose {
            progress_writer: writer,
            ..
        } => {
            writeln!(writer, "──────────────────────── MCTS Player ────────────────────────")?;
            writeln!(writer, "Duration:            {:.3?}", time_passed)?;
            writeln!(writer, "Iterations:          {}", iteration)?;
            writeln!(writer, "Expanded Depth:      {}", search_tree.get_expanded_depth())?;
            writeln!(writer, "Win Percentage:      {:.2}%", search_tree.get_win_prediction() * 100.0)?;
            writeln!(writer, "Principal Variation: {}", search_tree.get_pv_action_line())?;
            writeln!(writer, "Min/Max Evaluation:  {}/{}", search_tree.get_min_score(), search_tree.get_max_score())?;
            writeln!(writer, "─────────────────────────────────────────────────────────────")?;
        }
    };
    Ok(())
}

fn write_worker_error(diagnostics: &mut Diagnostics, message: &str) -> Result<(), std::io::Error> {
    match diagnostics {
        Diagnostics::Disabled => {}
        Diagnostics::Enabled { progress_writer } => {
            writeln!(progress_writer, "{}", message)?;
        }
        Diagnostics::Verbose {
            progress_writer,
            debug_writer,
        } => {
            writeln!(progress_writer, "{}", message)?;
            writeln!(debug_writer, "{}", message)?;
        }
        Diagnostics::VerboseOnly { debug_writer } => {
            writeln!(debug_writer, "{}", message)?;
        }
    }
    Ok(())
}

#[rustfmt::skip]
fn write_verbose_diagnostics(diagnostics: &mut Diagnostics, search_tree: &SearchTree<impl TreePolicy, impl Evaluator>) -> Result<(), std::io::Error> {
    match diagnostics {
        Diagnostics::Verbose { debug_writer: ref mut writer, .. } |
        Diagnostics::VerboseOnly { debug_writer: ref mut writer } => {
            if search_tree.get_expanded_depth() == 0 {
                writeln!(writer, "[MCTS] Could not expand all actions at depth 0")?;
            }

            search_tree.write_tree(writer)?;
        },
        _ => {}
    }

    Ok(())
}
