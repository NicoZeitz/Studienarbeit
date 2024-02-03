use std::num::NonZeroUsize;

use evaluator::ScoreEvaluator;
use patchwork_core::{ActionId, Diagnostics, Evaluator, Patchwork, Player, PlayerResult, TreePolicy};
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
    last_roots: Vec<Node>,
}

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
                let mut last_print = std::time::Instant::now();
                loop {
                    if time_passed >= *time_limit {
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

        let (best_action, _search_tree) = match &mut self.options {
            MCTSOptions {
                root_parallelization: NON_ZERO_USIZE_ONE,
                leaf_parallelization,
                end_condition,
                reuse_tree: _,
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

                (search_tree.pick_best_action(), search_tree)
            }
            // MCTSOptions {
            //     root_parallelization,
            //     leaf_parallelization: _,
            //     end_condition: MCTSEndCondition::Iterations(iterations),
            //     reuse_tree: _,
            // } => {
            //     let mut handles = Vec::with_capacity(root_parallelization.get());
            //     for _ in 0..root_parallelization.get() {
            //         let game = game.clone();

            //         handles.push(std::thread::spawn(move || {
            //             let search_tree = SearchTree::<Policy, Eval>::new(&game, &policy, &evaluator, &options);

            //             play_until_end!(MCTSEndCondition::Iterations(*iterations), {
            //                 search_tree.playout();
            //             });

            //             search_tree.search()
            //         }));
            //     }

            //     // let search_tree = SearchTree::<Policy, Eval>::new(game, &self.policy, &self.evaluator, &self.options);

            //     // play_until_end!(MCTSEndCondition::Iterations(iterations), {
            //     //     search_tree.playout();
            //     // });

            //     todo!();
            // }
            // MCTSOptions {
            //     root_parallelization,
            //     leaf_parallelization,
            //     end_condition: MCTSEndCondition::Time(duration),
            //     reuse_tree: _,
            // } => {}
            _ => todo!(),
        };

        // if self.options.reuse_tree {
        //     self.last_root = Some(search_tree.root.clone());
        // }

        write_verbose_diagnostics(&mut self.options.diagnostics)?;

        Ok(best_action)
    }
}

fn write_diagnostics<Policy: TreePolicy, Eval: Evaluator>(
    diagnostics: &mut Diagnostics,
    iteration: usize,
    time_passed: std::time::Duration,
    search_tree: &SearchTree<Policy, Eval>,
) -> Result<(), std::io::Error> {
    #[rustfmt::skip]
    match diagnostics {
        Diagnostics::Disabled => {}
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

#[rustfmt::skip]
fn write_verbose_diagnostics(diagnostics: &mut Diagnostics) -> Result<(), std::io::Error> {
    let Diagnostics::Verbose { debug_writer: writer, .. } = diagnostics else {
        return Ok(());
    };

    write!(writer, "TODO: print tree")?;
      // fs::write("test.txt", search_tree.tree_to_string()).expect("ERROR WRITING FILE"); // TODO: remove

    Ok(())
}
