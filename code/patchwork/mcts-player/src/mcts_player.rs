use std::{
    num::NonZeroUsize,
    ops::Sub,
    sync::{atomic::AtomicUsize, Arc},
    thread,
};

use evaluator::WinLossEvaluator;
use patchwork_core::{ActionId, Evaluator, Logging, Patchwork, Player, PlayerResult, TreePolicy, TreePolicyNode};
use tree_policy::UCTPolicy;

pub const NON_ZERO_USIZE_ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

use crate::{node_id::NodeId, AreaAllocator, MCTSEndCondition, MCTSOptions, SearchTree, Tree};

const REUSE_TREE_SEARCH_ABORT: Option<std::time::Duration> = Some(std::time::Duration::from_millis(2));
const TIME_LIMIT_SAFETY_MARGIN: std::time::Duration = std::time::Duration::from_millis(75);

/// A computer player that uses the Monte Carlo Tree Search (MCTS) algorithm to choose an action.
pub struct MCTSPlayer<Policy: TreePolicy = UCTPolicy, Eval: Evaluator = WinLossEvaluator> {
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The name of the player.
    pub name: String,
    /// The policy to select nodes during the selection phase.
    pub policy: Policy,
    /// The evaluator to evaluate the game state.
    pub evaluator: Eval,
    /// The full trees of the last run with the action that was taken to speed up the later search.
    last_trees: Vec<Tree>,
}

impl<Policy: TreePolicy + Default, Eval: Evaluator + Default> MCTSPlayer<Policy, Eval> {
    /// Creates a new [`MCTSPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<MCTSOptions>) -> Self {
        let options = options.unwrap_or_default();
        let last_trees = if options.reuse_tree {
            Vec::with_capacity(options.root_parallelization.get())
        } else {
            Vec::new()
        };

        Self {
            // name: format!(
            //     "{} [R{}|L{}|T{}]",
            //     name.into(),
            //     options.root_parallelization,
            //     options.leaf_parallelization,
            //     if options.reuse_tree { "R" } else { "N" }
            // ),
            name: name.into(),
            policy: Default::default(),
            evaluator: Default::default(),
            options,
            last_trees,
        }
    }
}

impl<Policy: TreePolicy + Default, Eval: Evaluator + Default> Default for MCTSPlayer<Policy, Eval> {
    fn default() -> Self {
        Self::new("MCTS Player".to_string(), Option::default())
    }
}

macro_rules! play_until_end_worker_thread {
    ($start_time:ident, $end_condition:expr, $playout:expr) => {
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
                let time_limit = time_limit.sub(TIME_LIMIT_SAFETY_MARGIN);
                let mut time_passed = std::time::Instant::now().duration_since($start_time);
                loop {
                    if time_passed >= time_limit {
                        break;
                    }

                    $playout;

                    time_passed = std::time::Instant::now().duration_since($start_time);
                }
            }
            MCTSEndCondition::Flag(flag) => {
                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    $playout;
                }
            }
        }
    };
}

macro_rules! play_until_end {
    ($start_time:ident, $end_condition:expr, $playout:expr, $logger_expr:expr, $logging_enabled:expr) => {
        let mut iteration = 0;
        let mut time_passed = std::time::Instant::now().duration_since($start_time);
        let logging_enabled = $logging_enabled;

        match $end_condition {
            MCTSEndCondition::Iterations(iterations) => {
                loop {
                    if iteration == *iterations {
                        break;
                    }

                    $playout;

                    iteration += 1;
                    time_passed = std::time::Instant::now().duration_since($start_time);

                    // Write logging information every 1000 iterations
                    if logging_enabled && iteration % 1000 == 0 {
                        #[allow(clippy::redundant_closure_call)]
                        $logger_expr(iteration, time_passed)?;
                    }
                }

                #[allow(clippy::redundant_closure_call)]
                $logger_expr(iteration, time_passed)?;
            }
            MCTSEndCondition::Time(time_limit) => {
                // add safety margin to time limit
                let time_limit = time_limit.sub(TIME_LIMIT_SAFETY_MARGIN);
                let mut last_print = std::time::Instant::now();
                loop {
                    if time_passed >= time_limit {
                        break;
                    }

                    $playout;

                    iteration += 1;
                    time_passed = std::time::Instant::now().duration_since($start_time);

                    // Write logging information every second
                    if logging_enabled && last_print.elapsed() >= std::time::Duration::from_secs(1) {
                        #[allow(clippy::redundant_closure_call)]
                        $logger_expr(iteration, time_passed)?;

                        last_print = std::time::Instant::now();
                    }
                }

                #[allow(clippy::redundant_closure_call)]
                $logger_expr(iteration, time_passed)?;
            }
            MCTSEndCondition::Flag(flag) => {
                let mut last_print = std::time::Instant::now();

                while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                    $playout;

                    iteration += 1;
                    time_passed = std::time::Instant::now().duration_since($start_time);

                    // Write logging information every second
                    if logging_enabled && last_print.elapsed() >= std::time::Duration::from_secs(1) {
                        #[allow(clippy::redundant_closure_call)]
                        $logger_expr(iteration, time_passed)?;

                        last_print = std::time::Instant::now();
                    }
                }

                #[allow(clippy::redundant_closure_call)]
                $logger_expr(iteration, time_passed)?;
            }
        }
    };
}

impl<Policy: TreePolicy, Eval: Evaluator> Player for MCTSPlayer<Policy, Eval> {
    fn name(&self) -> &str {
        &self.name
    }

    #[allow(clippy::too_many_lines)]
    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let start_time = std::time::Instant::now();

        Ok(match &mut self.options {
            MCTSOptions {
                root_parallelization: NON_ZERO_USIZE_ONE,
                leaf_parallelization,
                end_condition,
                reuse_tree,
                logging,
            } => {
                let last_tree = if self.last_trees.is_empty() {
                    None
                } else {
                    Some(self.last_trees.swap_remove(0))
                };

                let mut search_tree = SearchTree::<Policy, Eval>::from_root(
                    last_tree,
                    game,
                    &self.policy,
                    &self.evaluator,
                    REUSE_TREE_SEARCH_ABORT,
                );

                play_until_end!(
                    start_time,
                    end_condition,
                    search_tree.playout(*leaf_parallelization)?,
                    |iteration, time_passed| {
                        write_statistics(
                            logging,
                            iteration,
                            iteration,
                            time_passed,
                            1,
                            leaf_parallelization.get(),
                            *reuse_tree,
                            &search_tree,
                        )
                    },
                    matches!(logging, Logging::Enabled { .. } | Logging::Verbose { .. })
                );

                log_verbose_information(logging, &search_tree)?;

                let action = pick_best_action(&search_tree);

                if *reuse_tree {
                    self.last_trees = vec![get_tree_for_reuse(action, search_tree.root, search_tree.allocator)];
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
                logging,
            } => {
                let other_iterations = Arc::new(AtomicUsize::new(0));

                let trees = thread::scope::<'_, _, PlayerResult<Vec<Tree>>>(|s| {
                    let root_parallelization = (*root_parallelization).get();
                    let mut handles: Vec<thread::ScopedJoinHandle<'_, PlayerResult<Tree>>> =
                        Vec::with_capacity(root_parallelization);

                    for _ in 0..(root_parallelization - 1) {
                        // check for len > 2 to always keep at least the first root for the main search thread
                        let last_tree = if self.last_trees.len() > 2 {
                            Some(self.last_trees.remove(self.last_trees.len() - 1))
                        } else {
                            None
                        };

                        let evaluator = &self.evaluator;
                        let policy = &self.policy;
                        let leaf_parallel = *leaf_parallelization;
                        let end_cond = end_condition.clone();
                        let iterations = Arc::clone(&other_iterations);

                        // start worker search thread
                        handles.push(s.spawn(move || {
                            let mut search_tree = SearchTree::<Policy, Eval>::from_root(
                                last_tree,
                                game,
                                policy,
                                evaluator,
                                REUSE_TREE_SEARCH_ABORT,
                            );

                            play_until_end_worker_thread!(start_time, end_cond, {
                                search_tree.playout(leaf_parallel)?;
                                iterations.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                            });
                            Ok(Tree::new(search_tree.root, search_tree.allocator))
                        }));
                    }

                    let last_tree = if self.last_trees.is_empty() {
                        None
                    } else {
                        Some(self.last_trees.swap_remove(0))
                    };

                    let mut search_tree = SearchTree::<Policy, Eval>::from_root(
                        last_tree,
                        game,
                        &self.policy,
                        &self.evaluator,
                        REUSE_TREE_SEARCH_ABORT,
                    );

                    play_until_end!(
                        start_time,
                        end_condition,
                        search_tree.playout(*leaf_parallelization)?,
                        |iteration, time_passed| write_statistics(
                            logging,
                            iteration + other_iterations.load(std::sync::atomic::Ordering::Relaxed),
                            iteration,
                            time_passed,
                            root_parallelization,
                            leaf_parallelization.get(),
                            *reuse_tree,
                            &search_tree
                        ),
                        matches!(logging, Logging::Enabled { .. } | Logging::Verbose { .. })
                    );

                    log_verbose_information(logging, &search_tree)?;

                    let mut trees = vec![Tree::new(search_tree.root, search_tree.allocator)];

                    for handle in handles {
                        match handle.join() {
                            // safe to unwrap as the thread always puts the root into the wrapper before exiting
                            Ok(Ok(last_tree)) => trees.push(last_tree),
                            Err(error) => {
                                log_worker_error(
                                    logging,
                                    format!("[MCTSPlayer::get_action] Error in worker thread: {error:?}").as_str(),
                                )?;
                                continue; // Work with data from other threads
                            }
                            Ok(Err(error)) => {
                                if let Some(error) = error.downcast_ref::<String>() {
                                    log_worker_error(
                                        logging,
                                        format!("[MCTSPlayer::get_action] Error in worker thread: {error}").as_str(),
                                    )?;
                                } else {
                                    log_worker_error(
                                        logging,
                                        format!("[MCTSPlayer::get_action] Error in worker thread: {error:?}").as_str(),
                                    )?;
                                }
                                continue; // Work with data from other threads
                            }
                        }
                    }

                    Ok(trees)
                })?;

                let action = pick_best_action_from_multiple(&trees);

                if *reuse_tree {
                    self.last_trees =
                        trees.into_iter().map(|tree| get_tree_for_reuse(action, tree.root, tree.allocator)).collect();
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
pub fn pick_best_action(search_tree: &SearchTree<'_, impl TreePolicy, impl Evaluator>) -> ActionId {
    let root_id = search_tree.root;
    let root = search_tree.allocator.get_node(root_id);
    let root_player = root.state.is_player_1();

    let best_action_node_id = *root
        .children
        .iter()
        .max_by_key(|child_id| {
            let child = search_tree.allocator.get_node(**child_id);
            (child.visit_count, child.wins_for(root_player))
        })
        .unwrap();

    let best_action = search_tree.allocator.get_node(best_action_node_id).action_taken.unwrap();

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
/// * `allocator` - The allocator holding all nodes of the trees.
///
/// # Returns
///
/// The best action from the root nodes.
///
/// # Complexity
///
/// `𝒪(𝑚 · 𝑛)` where `𝑚` is the number of nodes and `𝑛` is the number of children of each root node.
pub fn pick_best_action_from_multiple(nodes: &[Tree]) -> ActionId {
    let mut action_map = std::collections::HashMap::new();

    for tree in nodes {
        let allocator = &tree.allocator;

        let parent = allocator.get_node(tree.root);
        for child_id in &parent.children {
            let child = allocator.get_node(*child_id);

            if let Some(action) = child.action_taken {
                let entry = action_map.entry(action).or_insert((0, 0));
                entry.0 += child.visit_count;
                entry.1 += child.wins_for(parent.state.is_player_1());
            }
        }
    }

    *action_map.iter().max_by_key(|(_, (visits, wins))| (*visits, *wins)).unwrap().0
}

/// Gets the tree to reuse for the given action.
/// Searches the children of the given root node (only depth 1) for having taken the given action to
/// arrive at the child node. If the action was taken, the child node is returned.
/// If no child with the given action was found, the root node is returned.
///
/// # Arguments
///
/// * `action` - The action to search for.
/// * `root` - The id of the root node.
/// * `allocator` - The allocator holding all nodes of the tree.
///
/// # Returns
///
/// The child node with the given action or the root node if no child with the given action was found.
///
/// # Complexity
///
/// `𝒪(𝑛)` where `𝑛` is the number of children of the root node.
fn get_tree_for_reuse(action: ActionId, root: NodeId, allocator: AreaAllocator) -> Tree {
    // default to current
    let mut next_root = root;

    for child_id in &allocator.get_node(next_root).children {
        let child = allocator.get_node(*child_id);

        if let Some(action_taken) = child.action_taken {
            if action_taken == action {
                next_root = *child_id;
                break;
            }
        }
    }

    Tree::new(next_root, allocator)
}

/// Writes the logging information of the search tree to the given writer.
/// The logs include:
/// * The duration of the search
/// * The number of iterations
/// * The expanded depth of the search tree
/// * The win prediction of the search tree
/// * The principal variation of the search tree
/// * The minimum and maximum evaluation of the search tree
///
/// # Arguments
///
/// * `logging` - The logging configuration with the write targets.
/// * `iteration` - The current iteration of the search.
/// * `time_passed` - The time passed since the start of the search.
/// * `search_tree` - The search tree where to get the logging information from.
///
/// # Returns
///
/// The result of the write operation.
#[allow(clippy::too_many_arguments)]
fn write_statistics(
    logging: &mut Logging,
    total_iterations: usize,
    iterations: usize,
    time_passed: std::time::Duration,
    root_parallelization: usize,
    leaf_parallelization: usize,
    reuse_tree: bool,
    search_tree: &SearchTree<'_, impl TreePolicy, impl Evaluator>,
) -> Result<(), std::io::Error> {
    #[rustfmt::skip]
    match logging {
        Logging::Disabled | Logging::VerboseOnly { .. } => {}
        Logging::Enabled {
            progress_writer: writer,
        }
        | Logging::Verbose {
            progress_writer: writer,
            ..
        } => {
            let mut features = vec![];
            if root_parallelization > 1 {
                features.push(format!("RP({root_parallelization})"));
            }
            if leaf_parallelization > 1 {
                features.push(format!("LP({leaf_parallelization})"));
            }
            if reuse_tree {
                features.push("RT".to_string());
            }

            writeln!(writer, "──────────────────────── MCTS Player ────────────────────────")?;
            writeln!(writer, "Features:            [{}]", features.join(", "))?;
            writeln!(writer, "Duration:            {time_passed:.3?}")?;
            if root_parallelization > 1 {
                writeln!(writer, "Total Iterations:    {total_iterations}")?;
            }
            writeln!(writer, "Iterations:          {iterations}")?;
            writeln!(writer, "Nodes:               {}", search_tree.get_nodes())?;
            if reuse_tree {
                writeln!(writer, "Reused Tree:         {}", search_tree.is_reused())?;
            }
            writeln!(writer, "Root actions:        {}", search_tree.get_root_actions())?;
            writeln!(writer, "Expanded Depth:      {}", search_tree.get_expanded_depth())?;
            writeln!(writer, "Win Percentage:      {:.2}%", search_tree.get_win_prediction() * 100.0)?;
            writeln!(writer, "Principal Variation: {}", search_tree.get_pv_action_line())?;
            writeln!(writer, "Min/Max Evaluation:  {}/{}", search_tree.get_min_score(), search_tree.get_max_score())?;
            writeln!(writer, "─────────────────────────────────────────────────────────────")?;
        }
    };
    Ok(())
}
/// Writes the error message to the logging writer.
///
/// # Arguments
///
/// * `logging` - The logging configuration with the write targets.
/// * `message` - The message to write.
///
/// # Returns
///
/// The result of the write operation.
fn log_worker_error(logging: &mut Logging, message: &str) -> Result<(), std::io::Error> {
    match logging {
        Logging::Disabled => {}
        Logging::Enabled { progress_writer } => {
            writeln!(progress_writer, "{message}")?;
        }
        Logging::Verbose {
            progress_writer,
            debug_writer,
        } => {
            writeln!(progress_writer, "{message}")?;
            writeln!(debug_writer, "{message}")?;
        }
        Logging::VerboseOnly { debug_writer } => {
            writeln!(debug_writer, "{message}")?;
        }
    }
    Ok(())
}

/// Writes the verbose logging information of the search tree to the given
/// writer. This is a full printout of the search tree to the debug writer.
///
/// # Arguments
///
/// * `logging` - The logging configuration with the write targets.
/// * `search_tree` - The search tree where to get the logging information from.
///
/// # Returns
///
/// The result of the write operation.
fn log_verbose_information(
    logging: &mut Logging,
    search_tree: &SearchTree<'_, impl TreePolicy, impl Evaluator>,
) -> Result<(), std::io::Error> {
    #[rustfmt::skip]
    match logging {
        Logging::Verbose { debug_writer: ref mut writer, .. } |
        Logging::VerboseOnly { debug_writer: ref mut writer } => {
            if search_tree.get_expanded_depth() == 0 {
                writeln!(writer, "[MCTS] Could not expand all actions at depth 0")?;
            }

            search_tree.write_tree(writer)?;
        },
        _ => {}
    };

    Ok(())
}
