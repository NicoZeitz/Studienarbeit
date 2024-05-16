use std::{
    env,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{Scope, ScopedJoinHandle},
};

use action_orderer::{ActionOrderer, TableActionOrderer};
use evaluator::StaticEvaluator;

use patchwork_core::{ActionId, Evaluator, Logging, Patchwork, Player, PlayerResult};
use transposition_table::TranspositionTable;

use crate::{
    constants::{
        DEFAULT_ENABLE_ASPIRATION_WINDOWS, DEFAULT_ENABLE_LATE_MOVE_PRUNING, DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS,
        DEFAULT_ENABLE_SEARCH_EXTENSIONS, DEFAULT_ENABLE_SEARCH_STATISTICS, DEFAULT_SOFT_FAILING_STRATEGY,
        DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    },
    pvs_options::FailingStrategy,
    pvs_worker::DefaultPVSWorker,
    LazySMPFeature, PVSFeatures, PVSOptions, TranspositionTableFeature,
};

/// A computer player that uses the Principal Variation Search (PVS) algorithm to choose an action.
///
/// # Features
/// - [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening)
/// - [Alpha-Beta Pruning](https://www.chessprogramming.org/Alpha-Beta)
/// - [Negamax](https://www.chessprogramming.org/Negamax)
/// - [Principal Variation Search (PVS)](https://www.chessprogramming.org/Principal_Variation_Search)
/// - [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows)
/// - [Transposition Table](https://www.chessprogramming.org/Transposition_Table)
/// - [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions)
/// - [Late Move Pruning](https://disservin.github.io/stockfish-docs/stockfish-wiki/Terminology.html#late-move-pruning)
/// - [Search Extension](https://www.chessprogramming.org/Extensions) - Win-seeking search extensions for special patch placements
/// - [Move Ordering](https://www.chessprogramming.org/Move_Ordering)
///     - With PV-Action via Transposition Table
pub struct PVSPlayer<
    const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char = DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    const SOFT_FAILING_STRATEGY: bool = DEFAULT_SOFT_FAILING_STRATEGY,
    const ENABLE_LATE_MOVE_REDUCTIONS: bool = DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS,
    const ENABLE_LATE_MOVE_PRUNING: bool = DEFAULT_ENABLE_LATE_MOVE_PRUNING,
    const ENABLE_ASPIRATION_WINDOWS: bool = DEFAULT_ENABLE_ASPIRATION_WINDOWS,
    const ENABLE_SEARCH_EXTENSIONS: bool = DEFAULT_ENABLE_SEARCH_EXTENSIONS,
    const ENABLE_SEARCH_STATISTICS: bool = DEFAULT_ENABLE_SEARCH_STATISTICS,
    Orderer: ActionOrderer = TableActionOrderer,
    Eval: Evaluator = StaticEvaluator,
> {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// The transposition table for storing previously searched positions.
    transposition_table: Arc<TranspositionTable>,
    orderer: PhantomData<Orderer>,
    evaluator: PhantomData<Eval>,
}

pub type DefaultPVSPlayer<Orderer = TableActionOrderer, Eval = StaticEvaluator> = PVSPlayer<
    DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    DEFAULT_SOFT_FAILING_STRATEGY,
    DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS,
    DEFAULT_ENABLE_LATE_MOVE_PRUNING,
    DEFAULT_ENABLE_ASPIRATION_WINDOWS,
    DEFAULT_ENABLE_SEARCH_EXTENSIONS,
    DEFAULT_ENABLE_SEARCH_STATISTICS,
    Orderer,
    Eval,
>;

impl<
        const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char,
        const SOFT_FAILING_STRATEGY: bool,
        const ENABLE_LATE_MOVE_REDUCTIONS: bool,
        const ENABLE_LATE_MOVE_PRUNING: bool,
        const ENABLE_ASPIRATION_WINDOWS: bool,
        const ENABLE_SEARCH_EXTENSIONS: bool,
        const ENABLE_SEARCH_STATISTICS: bool,
        Orderer: ActionOrderer,
        Eval: Evaluator,
    > Player
    for PVSPlayer<
        TRANSPOSITION_TABLE_SYMMETRY_TYPE,
        SOFT_FAILING_STRATEGY,
        ENABLE_LATE_MOVE_REDUCTIONS,
        ENABLE_LATE_MOVE_PRUNING,
        ENABLE_ASPIRATION_WINDOWS,
        ENABLE_SEARCH_EXTENSIONS,
        ENABLE_SEARCH_STATISTICS,
        Orderer,
        Eval,
    >
{
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        std::thread::scope(|scope| {
            let search_canceled = Arc::new(AtomicBool::new(false));
            let mut handles = vec![];
            let time_limit = self.options.time_limit;

            // Timer thread
            let timer_search_canceled = Arc::clone(&search_canceled);
            handles.push(scope.spawn(move || {
                let start_time = std::time::Instant::now();

                // Periodic check if the search was already canceled by itself
                while start_time.elapsed() < time_limit
                    && !timer_search_canceled.load(std::sync::atomic::Ordering::Acquire)
                {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                // Stop search after time limit
                timer_search_canceled.store(true, std::sync::atomic::Ordering::Release);

                Ok(None)
            }));

            // Lazy-SMP Threads
            match self.options.features.lazy_smp {
                LazySMPFeature::No => {}
                LazySMPFeature::Yes(parallelization) => {
                    for _ in 0..(parallelization.get() - 1) {
                        let worker = self.start_worker(scope, game.clone(), Arc::clone(&search_canceled));

                        handles.push(worker);
                    }
                }
            }

            // Start Main Thread
            let main_worker_result = self.start_main_worker(game.clone(), &search_canceled);

            let mut results = vec![main_worker_result];
            for handle in handles {
                results.push(handle.join().unwrap());
            }

            let action = self.extract_best_action(game, &results);

            let _ = self.write_log(format!("Best action: {action:?}").as_str()); // ignore errors

            Ok(action)
        })
    }
}

impl<
        const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char,
        const SOFT_FAILING_STRATEGY: bool,
        const ENABLE_LATE_MOVE_REDUCTIONS: bool,
        const ENABLE_LATE_MOVE_PRUNING: bool,
        const ENABLE_ASPIRATION_WINDOWS: bool,
        const ENABLE_SEARCH_EXTENSIONS: bool,
        const ENABLE_SEARCH_STATISTICS: bool,
        Orderer: ActionOrderer,
        Eval: Evaluator,
    >
    PVSPlayer<
        TRANSPOSITION_TABLE_SYMMETRY_TYPE,
        SOFT_FAILING_STRATEGY,
        ENABLE_LATE_MOVE_REDUCTIONS,
        ENABLE_LATE_MOVE_PRUNING,
        ENABLE_ASPIRATION_WINDOWS,
        ENABLE_SEARCH_EXTENSIONS,
        ENABLE_SEARCH_STATISTICS,
        Orderer,
        Eval,
    >
{
    fn start_worker<'scope>(
        &mut self,
        scope: &'scope Scope<'scope, 'static>,
        game: Patchwork,
        search_canceled: Arc<AtomicBool>,
    ) -> ScopedJoinHandle<'scope, PlayerResult<Option<(ActionId, i32)>>> {
        let transposition_table = Arc::clone(&self.transposition_table);
        scope.spawn(move || {
            let mut worker = DefaultPVSWorker::<
                false,
                TRANSPOSITION_TABLE_SYMMETRY_TYPE,
                SOFT_FAILING_STRATEGY,
                ENABLE_LATE_MOVE_REDUCTIONS,
                ENABLE_LATE_MOVE_PRUNING,
                ENABLE_ASPIRATION_WINDOWS,
                ENABLE_SEARCH_EXTENSIONS,
                false,
            >::new(Arc::clone(&search_canceled), transposition_table);

            let result = worker.search(game);

            search_canceled.store(true, Ordering::Release);

            result
        })
    }

    fn start_main_worker(
        &mut self,
        game: Patchwork,
        search_canceled: &Arc<AtomicBool>,
    ) -> PlayerResult<Option<(ActionId, i32)>> {
        let mut worker = DefaultPVSWorker::<
            true,
            TRANSPOSITION_TABLE_SYMMETRY_TYPE,
            SOFT_FAILING_STRATEGY,
            ENABLE_LATE_MOVE_REDUCTIONS,
            ENABLE_LATE_MOVE_PRUNING,
            ENABLE_ASPIRATION_WINDOWS,
            ENABLE_SEARCH_EXTENSIONS,
            ENABLE_SEARCH_STATISTICS,
        >::new(Arc::clone(search_canceled), Arc::clone(&self.transposition_table));

        if ENABLE_SEARCH_STATISTICS {
            worker.set_logging(&mut self.options.logging);
        }

        let result = worker.search(game);

        search_canceled.store(true, Ordering::Release);

        result
    }

    fn extract_best_action(&mut self, game: &Patchwork, results: &[PlayerResult<Option<(ActionId, i32)>>]) -> ActionId {
        let mut best_action = None;
        let mut best_evaluation = i32::MIN;

        for result in results {
            match result {
                Ok(None) => {}
                Ok(Some((action, evaluation))) => {
                    if *evaluation > best_evaluation {
                        best_evaluation = *evaluation;
                        best_action = Some(action);
                    }
                }
                Err(error) => {
                    let previous_backtrace = env::var("RUST_BACKTRACE");
                    env::set_var("RUST_BACKTRACE", "1");
                    let _ = self.write_log(format!("Worker Thread returned error: {error:?}").as_str());
                    env::set_var("RUST_BACKTRACE", previous_backtrace.unwrap_or_default());
                }
            }
        }

        if let Some(action) = best_action {
            return *action;
        }

        let _ = self.write_log("No Worker returned an Action. Using Transposition Table");

        if let Some(action) = self.transposition_table.probe_pv_move(game) {
            match game.clone().do_action(action, false) {
                Ok(()) => return action,
                Err(_) => {
                    let _ = self.write_log(format!("Invalid Action Found in Transposition Table: {action:?}").as_str());
                    // ignore errors
                }
            }
        }

        let _ = self.write_log("No best action found. Returning random valid action. This only happends when no full search iteration could be done."); // ignore errors

        game.get_random_action()
    }

    /// Writes a single str to the logging writer.
    ///
    /// # Arguments
    ///
    /// * `logging` - The logging configuration.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the writing.
    #[inline]
    fn write_log(&mut self, logging: &str) -> Result<(), std::io::Error> {
        let writer = match self.options.logging {
            Logging::Disabled | Logging::VerboseOnly { .. } => return Ok(()),
            Logging::Enabled {
                progress_writer: ref mut writer,
            }
            | Logging::Verbose {
                progress_writer: ref mut writer,
                ..
            } => writer.as_mut(),
        };

        writeln!(writer, "{logging}")
    }
}

impl<
        const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char,
        const SOFT_FAILING_STRATEGY: bool,
        const ENABLE_LATE_MOVE_REDUCTIONS: bool,
        const ENABLE_LATE_MOVE_PRUNING: bool,
        const ENABLE_ASPIRATION_WINDOWS: bool,
        const ENABLE_SEARCH_EXTENSIONS: bool,
        const ENABLE_SEARCH_STATISTICS: bool,
        Orderer: ActionOrderer + Default,
        Eval: Evaluator + Default,
    > Default
    for PVSPlayer<
        TRANSPOSITION_TABLE_SYMMETRY_TYPE,
        SOFT_FAILING_STRATEGY,
        ENABLE_LATE_MOVE_REDUCTIONS,
        ENABLE_LATE_MOVE_PRUNING,
        ENABLE_ASPIRATION_WINDOWS,
        ENABLE_SEARCH_EXTENSIONS,
        ENABLE_SEARCH_STATISTICS,
        Orderer,
        Eval,
    >
{
    fn default() -> Self {
        let options = PVSOptions {
            features: PVSFeatures {
                failing_strategy: if SOFT_FAILING_STRATEGY {
                    FailingStrategy::FailSoft
                } else {
                    FailingStrategy::FailHard
                },
                aspiration_window: ENABLE_ASPIRATION_WINDOWS,
                late_move_reductions: ENABLE_LATE_MOVE_PRUNING,
                late_move_pruning: ENABLE_LATE_MOVE_PRUNING,
                search_extensions: ENABLE_SEARCH_EXTENSIONS,
                transposition_table: match TRANSPOSITION_TABLE_SYMMETRY_TYPE {
                    'd' | 'D' => TranspositionTableFeature::Disabled,
                    'e' | 'E' => TranspositionTableFeature::Enabled {
                        size: TranspositionTableFeature::DEFAULT_SIZE,
                        strategy: TranspositionTableFeature::DEFAULT_STRATEGY,
                    },
                    's' | 'S' => TranspositionTableFeature::SymmetryEnabled {
                        size: TranspositionTableFeature::DEFAULT_SIZE,
                        strategy: TranspositionTableFeature::DEFAULT_STRATEGY,
                    },
                    _ => unreachable!(
                        "[PVSPlayer::default] Transposition table symmetry type is invalid: {} (valid 'd','e' and 's')",
                        TRANSPOSITION_TABLE_SYMMETRY_TYPE
                    ),
                },
                ..PVSFeatures::default()
            },
            ..PVSOptions::default()
        };
        let transposition_table = Arc::new(match options.features.transposition_table {
            TranspositionTableFeature::Disabled => TranspositionTable::empty(),
            TranspositionTableFeature::Enabled { size, strategy }
            | TranspositionTableFeature::SymmetryEnabled { size, strategy } => {
                TranspositionTable::new(size, strategy == FailingStrategy::FailSoft)
            }
        });

        Self {
            name: "Principal Variation Search Player".to_string(),
            options,
            transposition_table,
            evaluator: PhantomData,
            orderer: PhantomData,
        }
    }
}

impl<Orderer: ActionOrderer + Default, Eval: Evaluator + Default>
    PVSPlayer<
        DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
        DEFAULT_SOFT_FAILING_STRATEGY,
        DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS,
        DEFAULT_ENABLE_LATE_MOVE_PRUNING,
        DEFAULT_ENABLE_ASPIRATION_WINDOWS,
        DEFAULT_ENABLE_SEARCH_EXTENSIONS,
        DEFAULT_ENABLE_SEARCH_STATISTICS,
        Orderer,
        Eval,
    >
{
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name and options.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the player.
    /// * `options` - The options for the Principal Variation Search (PVS) algorithm.
    ///
    /// # Returns
    ///
    /// A new [`PrincipalVariationSearchPlayer`] with the given name and options.
    #[must_use]
    #[rustfmt::skip]
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::too_many_lines)]
    pub fn new(name: impl Into<String>, options: Option<PVSOptions>) -> Box<dyn Player> {
        let options = options.unwrap_or_default();
        let name = name.into();
        let transposition_table = Arc::new(match options.features.transposition_table {
            TranspositionTableFeature::Disabled => TranspositionTable::empty(),
            TranspositionTableFeature::Enabled { size, strategy }
            | TranspositionTableFeature::SymmetryEnabled { size, strategy } => {
                TranspositionTable::new(size, strategy == FailingStrategy::FailSoft)
            }
        });

        match (
            options.features.transposition_table,
            options.features.failing_strategy,
            options.features.late_move_reductions,
            options.features.late_move_pruning,
            options.features.aspiration_window,
            options.features.search_extensions,
            options.logging.is_enabled()
        ) {
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, true, true, true) => Box::new(PVSPlayer::<'d', true, true, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, true, true, false) => Box::new(PVSPlayer::<'d', true, true, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, true, false, true) => Box::new(PVSPlayer::<'d', true, true, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, true, false, false) => Box::new(PVSPlayer::<'d', true, true, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, false, true, true) => Box::new(PVSPlayer::<'d', true, true, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, false, true, false) => Box::new(PVSPlayer::<'d', true, true, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, false, false, true) => Box::new(PVSPlayer::<'d', true, true, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, true, false, false, false) => Box::new(PVSPlayer::<'d', true, true, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, true, true, true) => Box::new(PVSPlayer::<'d', true, true, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, true, true, false) => Box::new(PVSPlayer::<'d', true, true, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, true, false, true) => Box::new(PVSPlayer::<'d', true, true, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, true, false, false) => Box::new(PVSPlayer::<'d', true, true, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, false, true, true) => Box::new(PVSPlayer::<'d', true, true, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, false, true, false) => Box::new(PVSPlayer::<'d', true, true, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, false, false, true) => Box::new(PVSPlayer::<'d', true, true, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, true, false, false, false, false) => Box::new(PVSPlayer::<'d', true, true, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, true, true, true) => Box::new(PVSPlayer::<'d', true, false, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, true, true, false) => Box::new(PVSPlayer::<'d', true, false, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, true, false, true) => Box::new(PVSPlayer::<'d', true, false, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, true, false, false) => Box::new(PVSPlayer::<'d', true, false, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, false, true, true) => Box::new(PVSPlayer::<'d', true, false, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, false, true, false) => Box::new(PVSPlayer::<'d', true, false, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, false, false, true) => Box::new(PVSPlayer::<'d', true, false, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, true, false, false, false) => Box::new(PVSPlayer::<'d', true, false, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, true, true, true) => Box::new(PVSPlayer::<'d', true, false, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, true, true, false) => Box::new(PVSPlayer::<'d', true, false, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, true, false, true) => Box::new(PVSPlayer::<'d', true, false, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, true, false, false) => Box::new(PVSPlayer::<'d', true, false, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, false, true, true) => Box::new(PVSPlayer::<'d', true, false, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, false, true, false) => Box::new(PVSPlayer::<'d', true, false, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, false, false, true) => Box::new(PVSPlayer::<'d', true, false, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailSoft, false, false, false, false, false) => Box::new(PVSPlayer::<'d', true, false, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, true, true, true) => Box::new(PVSPlayer::<'d', false, true, true, true, true, true> { name, options, transposition_table, ..Default::default()}),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, true, true, false) => Box::new(PVSPlayer::<'d', false, true, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, true, false, true) => Box::new(PVSPlayer::<'d', false, true, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, true, false, false) => Box::new(PVSPlayer::<'d', false, true, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, false, true, true) => Box::new(PVSPlayer::<'d', false, true, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, false, true, false) => Box::new(PVSPlayer::<'d', false, true, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, false, false, true) => Box::new(PVSPlayer::<'d', false, true, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, true, false, false, false) => Box::new(PVSPlayer::<'d', false, true, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, true, true, true) => Box::new(PVSPlayer::<'d', false, true, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, true, true, false) => Box::new(PVSPlayer::<'d', false, true, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, true, false, true) => Box::new(PVSPlayer::<'d', false, true, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, true, false, false) => Box::new(PVSPlayer::<'d', false, true, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, false, true, true) => Box::new(PVSPlayer::<'d', false, true, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, false, true, false) => Box::new(PVSPlayer::<'d', false, true, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, false, false, true) => Box::new(PVSPlayer::<'d', false, true, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, true, false, false, false, false) => Box::new(PVSPlayer::<'d', false, true, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, true, true, true) => Box::new(PVSPlayer::<'d', false, false, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, true, true, false) => Box::new(PVSPlayer::<'d', false, false, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, true, false, true) => Box::new(PVSPlayer::<'d', false, false, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, true, false, false) => Box::new(PVSPlayer::<'d', false, false, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, false, true, true) => Box::new(PVSPlayer::<'d', false, false, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, false, true, false) => Box::new(PVSPlayer::<'d', false, false, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, false, false, true) => Box::new(PVSPlayer::<'d', false, false, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, true, false, false, false) => Box::new(PVSPlayer::<'d', false, false, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, true, true, true) => Box::new(PVSPlayer::<'d', false, false, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, true, true, false) => Box::new(PVSPlayer::<'d', false, false, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, true, false, true) => Box::new(PVSPlayer::<'d', false, false, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, true, false, false) => Box::new(PVSPlayer::<'d', false, false, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, false, true, true) => Box::new(PVSPlayer::<'d', false, false, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, false, true, false) => Box::new(PVSPlayer::<'d', false, false, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, false, false, true) => Box::new(PVSPlayer::<'d', false, false, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Disabled, FailingStrategy::FailHard, false, false, false, false, false) => Box::new(PVSPlayer::<'d', false, false, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, true, true, true) => Box::new(PVSPlayer::<'e', true, true, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, true, true, false) => Box::new(PVSPlayer::<'e', true, true, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, true, false, true) => Box::new(PVSPlayer::<'e', true, true, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, true, false, false) => Box::new(PVSPlayer::<'e', true, true, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, false, true, true) => Box::new(PVSPlayer::<'e', true, true, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, false, true, false) => Box::new(PVSPlayer::<'e', true, true, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, false, false, true) => Box::new(PVSPlayer::<'e', true, true, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, true, false, false, false) => Box::new(PVSPlayer::<'e', true, true, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, true, true, true) => Box::new(PVSPlayer::<'e', true, true, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, true, true, false) => Box::new(PVSPlayer::<'e', true, true, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, true, false, true) => Box::new(PVSPlayer::<'e', true, true, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, true, false, false) => Box::new(PVSPlayer::<'e', true, true, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, false, true, true) => Box::new(PVSPlayer::<'e', true, true, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, false, true, false) => Box::new(PVSPlayer::<'e', true, true, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, false, false, true) => Box::new(PVSPlayer::<'e', true, true, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, true, false, false, false, false) => Box::new(PVSPlayer::<'e', true, true, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, true, true, true) => Box::new(PVSPlayer::<'e', true, false, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, true, true, false) => Box::new(PVSPlayer::<'e', true, false, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, true, false, true) => Box::new(PVSPlayer::<'e', true, false, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, true, false, false) => Box::new(PVSPlayer::<'e', true, false, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, false, true, true) => Box::new(PVSPlayer::<'e', true, false, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, false, true, false) => Box::new(PVSPlayer::<'e', true, false, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, false, false, true) => Box::new(PVSPlayer::<'e', true, false, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, true, false, false, false) => Box::new(PVSPlayer::<'e', true, false, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, true, true, true) => Box::new(PVSPlayer::<'e', true, false, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, true, true, false) => Box::new(PVSPlayer::<'e', true, false, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, true, false, true) => Box::new(PVSPlayer::<'e', true, false, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, true, false, false) => Box::new(PVSPlayer::<'e', true, false, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, false, true, true) => Box::new(PVSPlayer::<'e', true, false, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, false, true, false) => Box::new(PVSPlayer::<'e', true, false, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, false, false, true) => Box::new(PVSPlayer::<'e', true, false, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailSoft, false, false, false, false, false) => Box::new(PVSPlayer::<'e', true, false, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, true, true, true) => Box::new(PVSPlayer::<'e', false, true, true, true, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, true, true, false) => Box::new(PVSPlayer::<'e', false, true, true, true, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, true, false, true) => Box::new(PVSPlayer::<'e', false, true, true, true, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, true, false, false) => Box::new(PVSPlayer::<'e', false, true, true, true, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, false, true, true) => Box::new(PVSPlayer::<'e', false, true, true, false, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, false, true, false) => Box::new(PVSPlayer::<'e', false, true, true, false, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, false, false, true) => Box::new(PVSPlayer::<'e', false, true, true, false, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, true, false, false, false) => Box::new(PVSPlayer::<'e', false, true, true, false, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, true, true, true) => Box::new(PVSPlayer::<'e', false, true, false, true, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, true, true, false) => Box::new(PVSPlayer::<'e', false, true, false, true, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, true, false, true) => Box::new(PVSPlayer::<'e', false, true, false, true, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, true, false, false) => Box::new(PVSPlayer::<'e', false, true, false, true, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, false, true, true) => Box::new(PVSPlayer::<'e', false, true, false, false, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, false, true, false) => Box::new(PVSPlayer::<'e', false, true, false, false, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, false, false, true) => Box::new(PVSPlayer::<'e', false, true, false, false, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, true, false, false, false, false) => Box::new(PVSPlayer::<'e', false, true, false, false, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, true, true, true) => Box::new(PVSPlayer::<'e', false, false, true, true, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, true, true, false) => Box::new(PVSPlayer::<'e', false, false, true, true, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, true, false, true) => Box::new(PVSPlayer::<'e', false, false, true, true, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, true, false, false) => Box::new(PVSPlayer::<'e', false, false, true, true, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, false, true, true) => Box::new(PVSPlayer::<'e', false, false, true, false, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, false, true, false) => Box::new(PVSPlayer::<'e', false, false, true, false, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, false, false, true) => Box::new(PVSPlayer::<'e', false, false, true, false, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, true, false, false, false) => Box::new(PVSPlayer::<'e', false, false, true, false, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, true, true, true) => Box::new(PVSPlayer::<'e', false, false, false, true, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, true, true, false) => Box::new(PVSPlayer::<'e', false, false, false, true, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, true, false, true) => Box::new(PVSPlayer::<'e', false, false, false, true, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, true, false, false) => Box::new(PVSPlayer::<'e', false, false, false, true, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, false, true, true) => Box::new(PVSPlayer::<'e', false, false, false, false, true, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, false, true, false) => Box::new(PVSPlayer::<'e', false, false, false, false, true, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, false, false, true) => Box::new(PVSPlayer::<'e', false, false, false, false, false, true> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::Enabled { .. }, FailingStrategy::FailHard, false, false, false, false, false) => Box::new(PVSPlayer::<'e', false, false, false, false, false, false> { name, options, transposition_table, ..Default::default() } ),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, true, true, true) => Box::new(PVSPlayer::<'s', true, true, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, true, true, false) => Box::new(PVSPlayer::<'s', true, true, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, true, false, true) => Box::new(PVSPlayer::<'s', true, true, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, true, false, false) => Box::new(PVSPlayer::<'s', true, true, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, false, true, true) => Box::new(PVSPlayer::<'s', true, true, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, false, true, false) => Box::new(PVSPlayer::<'s', true, true, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, false, false, true) => Box::new(PVSPlayer::<'s', true, true, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, true, false, false, false) => Box::new(PVSPlayer::<'s', true, true, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, true, true, true) => Box::new(PVSPlayer::<'s', true, true, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, true, true, false) => Box::new(PVSPlayer::<'s', true, true, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, true, false, true) => Box::new(PVSPlayer::<'s', true, true, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, true, false, false) => Box::new(PVSPlayer::<'s', true, true, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, false, true, true) => Box::new(PVSPlayer::<'s', true, true, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, false, true, false) => Box::new(PVSPlayer::<'s', true, true, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, false, false, true) => Box::new(PVSPlayer::<'s', true, true, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, true, false, false, false, false) => Box::new(PVSPlayer::<'s', true, true, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, true, true, true) => Box::new(PVSPlayer::<'s', true, false, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, true, true, false) => Box::new(PVSPlayer::<'s', true, false, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, true, false, true) => Box::new(PVSPlayer::<'s', true, false, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, true, false, false) => Box::new(PVSPlayer::<'s', true, false, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, false, true, true) => Box::new(PVSPlayer::<'s', true, false, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, false, true, false) => Box::new(PVSPlayer::<'s', true, false, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, false, false, true) => Box::new(PVSPlayer::<'s', true, false, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, true, false, false, false) => Box::new(PVSPlayer::<'s', true, false, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, true, true, true) => Box::new(PVSPlayer::<'s', true, false, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, true, true, false) => Box::new(PVSPlayer::<'s', true, false, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, true, false, true) => Box::new(PVSPlayer::<'s', true, false, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, true, false, false) => Box::new(PVSPlayer::<'s', true, false, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, false, true, true) => Box::new(PVSPlayer::<'s', true, false, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, false, true, false) => Box::new(PVSPlayer::<'s', true, false, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, false, false, true) => Box::new(PVSPlayer::<'s', true, false, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailSoft, false, false, false, false, false) => Box::new(PVSPlayer::<'s', true, false, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, true, true, true) => Box::new(PVSPlayer::<'s', false, true, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, true, true, false) => Box::new(PVSPlayer::<'s', false, true, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, true, false, true) => Box::new(PVSPlayer::<'s', false, true, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, true, false, false) => Box::new(PVSPlayer::<'s', false, true, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, false, true, true) => Box::new(PVSPlayer::<'s', false, true, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, false, true, false) => Box::new(PVSPlayer::<'s', false, true, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, false, false, true) => Box::new(PVSPlayer::<'s', false, true, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, true, false, false, false) => Box::new(PVSPlayer::<'s', false, true, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, true, true, true) => Box::new(PVSPlayer::<'s', false, true, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, true, true, false) => Box::new(PVSPlayer::<'s', false, true, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, true, false, true) => Box::new(PVSPlayer::<'s', false, true, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, true, false, false) => Box::new(PVSPlayer::<'s', false, true, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, false, true, true) => Box::new(PVSPlayer::<'s', false, true, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, false, true, false) => Box::new(PVSPlayer::<'s', false, true, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, false, false, true) => Box::new(PVSPlayer::<'s', false, true, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, true, false, false, false, false) => Box::new(PVSPlayer::<'s', false, true, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, true, true, true) => Box::new(PVSPlayer::<'s', false, false, true, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, true, true, false) => Box::new(PVSPlayer::<'s', false, false, true, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, true, false, true) => Box::new(PVSPlayer::<'s', false, false, true, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, true, false, false) => Box::new(PVSPlayer::<'s', false, false, true, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, false, true, true) => Box::new(PVSPlayer::<'s', false, false, true, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, false, true, false) => Box::new(PVSPlayer::<'s', false, false, true, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, false, false, true) => Box::new(PVSPlayer::<'s', false, false, true, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, true, false, false, false) => Box::new(PVSPlayer::<'s', false, false, true, false, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, true, true, true) => Box::new(PVSPlayer::<'s', false, false, false, true, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, true, true, false) => Box::new(PVSPlayer::<'s', false, false, false, true, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, true, false, true) => Box::new(PVSPlayer::<'s', false, false, false, true, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, true, false, false) => Box::new(PVSPlayer::<'s', false, false, false, true, false, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, false, true, true) => Box::new(PVSPlayer::<'s', false, false, false, false, true, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, false, true, false) => Box::new(PVSPlayer::<'s', false, false, false, false, true, false> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, false, false, true) => Box::new(PVSPlayer::<'s', false, false, false, false, false, true> { name, options, transposition_table, ..Default::default() }),
            (TranspositionTableFeature::SymmetryEnabled { .. }, FailingStrategy::FailHard, false, false, false, false, false) => Box::new(PVSPlayer::<'s', false, false, false, false, false, false> { name, options, transposition_table, ..Default::default() }),
        }
    }
}
