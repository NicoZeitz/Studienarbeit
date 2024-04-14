use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use action_orderer::{ActionList, ActionOrderer, TableActionOrderer};
use evaluator::StaticEvaluator;
use itertools::Itertools;
use patchwork_core::{evaluator_constants, ActionId, Evaluator, Logging, Notation, Patchwork, PlayerResult, TurnType};
use transposition_table::{EvaluationType, TranspositionTable};

use crate::{
    constants::{
        DEFAULT_ASPIRATION_WINDOWS_MINIMUM_DELTA, DEFAULT_ASPIRATION_WINDOWS_STARTING_ALPHA,
        DEFAULT_ASPIRATION_WINDOWS_STARTING_BETA, DEFAULT_ENABLE_ASPIRATION_WINDOWS, DEFAULT_ENABLE_LATE_MOVE_PRUNING,
        DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS, DEFAULT_ENABLE_SEARCH_EXTENSIONS, DEFAULT_LMP_AMOUNT_NON_PRUNED_ACTIONS,
        DEFAULT_LMP_AMOUNT_OF_ACTIONS_PER_PATCH, DEFAULT_LMP_APPLY_AFTER_PLYS, DEFAULT_LMR_AMOUNT_FULL_DEPTH_ACTIONS,
        DEFAULT_LMR_APPLY_AFTER_PLYS, DEFAULT_MAX_SEARCH_EXTENSIONS, DEFAULT_SOFT_FAILING_STRATEGY,
        DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    },
    lmp_flags::LMPFlags,
    search_recorder::SearchRecorder,
    SearchStatistics,
};

pub type DefaultPVSWorker<
    'worker,
    const IS_MAIN_WORKER: bool,
    const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char,
    const SOFT_FAILING_STRATEGY: bool,
    const ENABLE_LATE_MOVE_REDUCTIONS: bool,
    const ENABLE_LATE_MOVE_PRUNING: bool,
    const ENABLE_ASPIRATION_WINDOWS: bool,
    const ENABLE_SEARCH_EXTENSIONS: bool,
    const ENABLE_SEARCH_STATISTICS: bool,
> = PVSWorker<
    'worker,
    IS_MAIN_WORKER,
    TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    SOFT_FAILING_STRATEGY,
    ENABLE_LATE_MOVE_REDUCTIONS,
    ENABLE_LATE_MOVE_PRUNING,
    ENABLE_ASPIRATION_WINDOWS,
    ENABLE_SEARCH_EXTENSIONS,
    ENABLE_SEARCH_STATISTICS,
>;

pub struct PVSWorker<
    'worker,
    const IS_MAIN_WORKER: bool,
    const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char = DEFAULT_TRANSPOSITION_TABLE_SYMMETRY_TYPE,
    const SOFT_FAILING_STRATEGY: bool = DEFAULT_SOFT_FAILING_STRATEGY,
    const ENABLE_LATE_MOVE_REDUCTIONS: bool = DEFAULT_ENABLE_LATE_MOVE_REDUCTIONS,
    const ENABLE_LATE_MOVE_PRUNING: bool = DEFAULT_ENABLE_LATE_MOVE_PRUNING,
    const ENABLE_ASPIRATION_WINDOWS: bool = DEFAULT_ENABLE_ASPIRATION_WINDOWS,
    const ENABLE_SEARCH_EXTENSIONS: bool = DEFAULT_ENABLE_SEARCH_EXTENSIONS,
    const ENABLE_SEARCH_STATISTICS: bool = IS_MAIN_WORKER,
    const ENABLE_SEARCH_RECORDER: bool = false,
    const LMR_AMOUNT_FULL_DEPTH_ACTIONS: usize = DEFAULT_LMR_AMOUNT_FULL_DEPTH_ACTIONS,
    const LMR_APPLY_AFTER_PLYS: usize = DEFAULT_LMR_APPLY_AFTER_PLYS,
    const LMP_AMOUNT_NON_PRUNED_ACTIONS: usize = DEFAULT_LMP_AMOUNT_NON_PRUNED_ACTIONS,
    const LMP_APPLY_AFTER_PLYS: usize = DEFAULT_LMP_APPLY_AFTER_PLYS,
    const LMP_AMOUNT_OF_ACTIONS_PER_PATCH: usize = DEFAULT_LMP_AMOUNT_OF_ACTIONS_PER_PATCH,
    const MAX_SEARCH_EXTENSIONS: usize = DEFAULT_MAX_SEARCH_EXTENSIONS,
    const ASPIRATION_WINDOWS_STARTING_ALPHA: i32 = DEFAULT_ASPIRATION_WINDOWS_STARTING_ALPHA,
    const ASPIRATION_WINDOWS_STARTING_BETA: i32 = DEFAULT_ASPIRATION_WINDOWS_STARTING_BETA,
    const ASPIRATION_WINDOWS_MINIMUM_DELTA: i32 = DEFAULT_ASPIRATION_WINDOWS_MINIMUM_DELTA,
    Orderer: ActionOrderer = TableActionOrderer,
    Eval: Evaluator = StaticEvaluator,
> {
    /// Whether the search has been canceled.
    search_canceled: Arc<AtomicBool>,
    /// search statistics
    pub statistics: SearchStatistics<ENABLE_SEARCH_STATISTICS>,
    /// The evaluator to evaluate the game state.
    pub evaluator: Eval,
    /// The action sorter to sort the actions.
    pub action_orderer: Orderer,
    /// The transposition table.
    pub transposition_table: Arc<TranspositionTable>,
    /// The best action found so far.
    ///
    /// The best action for the root is kept in a separate variable so that
    /// it can be returned even if the transposition table is disabled or the
    /// pv action is overwritten by the transposition table.
    best_action: Option<ActionId>,
    /// The best evaluation found so far.
    best_evaluation: Option<i32>,
    /// The logging to use.
    logging: Option<&'worker mut Logging>,
    // The search recorder used to record the search tree
    search_recorder: SearchRecorder<ENABLE_SEARCH_RECORDER>,
}

impl<
        'worker,
        const IS_MAIN_WORKER: bool,
        const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char,
        const SOFT_FAILING_STRATEGY: bool,
        const ENABLE_LATE_MOVE_REDUCTIONS: bool,
        const ENABLE_LATE_MOVE_PRUNING: bool,
        const ENABLE_ASPIRATION_WINDOWS: bool,
        const ENABLE_SEARCH_EXTENSIONS: bool,
        const ENABLE_SEARCH_STATISTICS: bool,
        const ENABLE_SEARCH_RECORDER: bool,
        const LMR_AMOUNT_FULL_DEPTH_ACTIONS: usize,
        const LMR_APPLY_AFTER_PLYS: usize,
        const LMP_AMOUNT_NON_PRUNED_ACTIONS: usize,
        const LMP_APPLY_AFTER_PLYS: usize,
        const LMP_AMOUNT_OF_ACTIONS_PER_PATCH: usize,
        const MAX_SEARCH_EXTENSIONS: usize,
        const ASPIRATION_WINDOWS_STARTING_ALPHA: i32,
        const ASPIRATION_WINDOWS_STARTING_BETA: i32,
        const ASPIRATION_WINDOWS_MINIMUM_DELTA: i32,
        Orderer: ActionOrderer + Default,
        Eval: Evaluator + Default,
    >
    PVSWorker<
        'worker,
        IS_MAIN_WORKER,
        TRANSPOSITION_TABLE_SYMMETRY_TYPE,
        SOFT_FAILING_STRATEGY,
        ENABLE_LATE_MOVE_REDUCTIONS,
        ENABLE_LATE_MOVE_PRUNING,
        ENABLE_ASPIRATION_WINDOWS,
        ENABLE_SEARCH_EXTENSIONS,
        ENABLE_SEARCH_STATISTICS,
        ENABLE_SEARCH_RECORDER,
        LMR_AMOUNT_FULL_DEPTH_ACTIONS,
        LMR_APPLY_AFTER_PLYS,
        LMP_AMOUNT_NON_PRUNED_ACTIONS,
        LMP_APPLY_AFTER_PLYS,
        LMP_AMOUNT_OF_ACTIONS_PER_PATCH,
        MAX_SEARCH_EXTENSIONS,
        ASPIRATION_WINDOWS_STARTING_ALPHA,
        ASPIRATION_WINDOWS_STARTING_BETA,
        ASPIRATION_WINDOWS_MINIMUM_DELTA,
        Orderer,
        Eval,
    >
{
    // ───────────────────────────────────────── CONSTANTS ─────────────────────────────────────────

    /// The maximum depth to search.
    /// This is an upper bound that can probably never will be reached
    /// no game can go longer than 63·2 plys
    ///
    /// Phantom moves do not count towards the ply count and depth count
    ///
    /// This is equal to the maximum amount of ply's that is searched (including phantom actions)
    pub const MAX_DEPTH: usize = 126;

    /// The minimum bound for alpha. Ensures that the minimum alpha value is
    /// less than the minimum evaluation to avoid a fail-low with the maximum
    /// window size.
    pub const MIN_ALPHA_BOUND: i32 = evaluator_constants::NEGATIVE_INFINITY - 1;
    /// The maximum bound for beta. Ensures that the maximum beta value is
    /// greater than the maximum evaluation to avoid a fail-high with the
    /// maximum window size.
    pub const MAX_BETA_BOUND: i32 = evaluator_constants::POSITIVE_INFINITY + 1;

    pub const IS_MAIN_WORKER: bool = IS_MAIN_WORKER;
    pub const SOFT_FAILING_STRATEGY: bool = SOFT_FAILING_STRATEGY;
    pub const ENABLE_LATE_MOVE_REDUCTIONS: bool = ENABLE_LATE_MOVE_REDUCTIONS;
    pub const ENABLE_LATE_MOVE_PRUNING: bool = ENABLE_LATE_MOVE_PRUNING;
    pub const ENABLE_ASPIRATION_WINDOWS: bool = ENABLE_ASPIRATION_WINDOWS;
    pub const ENABLE_SEARCH_EXTENSIONS: bool = ENABLE_SEARCH_EXTENSIONS;
    pub const TRANSPOSITION_TABLE_SYMMETRY_TYPE: char = TRANSPOSITION_TABLE_SYMMETRY_TYPE;
    pub const ENABLE_TRANSPOSITION_TABLE: bool =
        TRANSPOSITION_TABLE_SYMMETRY_TYPE != Self::TRANSPOSITION_TABLE_DISABLED;
    pub const ENABLE_SEARCH_STATISTICS: bool = ENABLE_SEARCH_STATISTICS;
    pub const LMR_AMOUNT_FULL_DEPTH_ACTIONS: usize = LMR_AMOUNT_FULL_DEPTH_ACTIONS;
    pub const LMR_APPLY_AFTER_PLYS: usize = LMR_APPLY_AFTER_PLYS;
    pub const LMP_AMOUNT_NON_PRUNED_ACTIONS: usize = LMP_AMOUNT_NON_PRUNED_ACTIONS;
    pub const LMP_APPLY_AFTER_PLYS: usize = LMP_APPLY_AFTER_PLYS;
    pub const MAX_SEARCH_EXTENSIONS: usize = MAX_SEARCH_EXTENSIONS;
    pub const ASPIRATION_WINDOWS_STARTING_ALPHA: i32 = ASPIRATION_WINDOWS_STARTING_ALPHA;
    pub const ASPIRATION_WINDOWS_STARTING_BETA: i32 = ASPIRATION_WINDOWS_STARTING_BETA;
    pub const ASPIRATION_WINDOWS_MIN_DELTA: i32 = ASPIRATION_WINDOWS_MINIMUM_DELTA;

    pub const TRANSPOSITION_TABLE_DISABLED: char = 'd';
    pub const TRANSPOSITION_TABLE_ENABLED: char = 'e';
    pub const TRANSPOSITION_TABLE_SYMMETRY_ENABLED: char = 's';

    // ───────────────────────────────────── FACTORY FUNCTIONS ─────────────────────────────────────

    /// Creates a new PVS worker with the given transposition table.
    ///
    /// # Arguments
    ///
    /// * `search_canceled` - The flag to check if the search has been canceled.
    /// * `transposition_table` - The transposition table
    ///
    /// # Returns
    ///
    /// The new PVS worker.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn new(search_canceled: Arc<AtomicBool>, transposition_table: Arc<TranspositionTable>) -> Self {
        assert!(
            TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_DISABLED
                || TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_ENABLED
                || TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_SYMMETRY_ENABLED,
            "[PVSWorker::new] Transposition table symmetry type is invalid: {} (valid 'd','e' and 's')",
            Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE
        );

        Self {
            statistics: SearchStatistics::default(),
            evaluator: Eval::default(),
            action_orderer: Orderer::default(),
            search_canceled,
            transposition_table,
            best_action: None,
            best_evaluation: None,
            logging: None,
            search_recorder: SearchRecorder::<ENABLE_SEARCH_RECORDER>::new(),
        }
    }

    // ────────────────────────────────────────── SETTERS ──────────────────────────────────────────

    /// Sets the logging to use.
    ///
    /// # Arguments
    ///
    /// * `logging` - The logging to use.
    pub fn set_logging(&mut self, logging: &'worker mut Logging) {
        self.logging = Some(logging);
    }

    // ──────────────────────── ITERATIVE DEEPENING AND ASPIRATION WINDOWS  ────────────────────────

    /// Does a Iterative Deepening Principal Variation Search (PVS) with the
    /// given parameters.
    ///
    /// Stops the search when the `search_canceled` flag is set to `true`.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    pub fn search(&mut self, mut game: Patchwork) -> PlayerResult<Option<(ActionId, i32)>> {
        let mut delta = Self::ASPIRATION_WINDOWS_MIN_DELTA;
        let mut alpha = Self::MIN_ALPHA_BOUND;
        let mut beta = Self::MAX_BETA_BOUND;
        let mut depth = 1;

        if Self::ENABLE_ASPIRATION_WINDOWS {
            alpha = Self::ASPIRATION_WINDOWS_STARTING_ALPHA;
            beta = Self::ASPIRATION_WINDOWS_STARTING_BETA;
        }

        self.statistics.reset_iterative_deepening_iteration(); /* STATISTICS */

        // [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening) loop
        while depth < Self::MAX_DEPTH {
            let best_action = self.best_action;
            let best_evaluation = self.best_evaluation;

            let evaluation = self.principal_variation_search::<false>(&mut game, 0, depth, alpha, beta, 0)?;

            self.search_recorder.print_to_file(); /* SEARCH RECORDER */

            if self.search_canceled.load(Ordering::Acquire) {
                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                break;
            }

            // [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows)
            if Self::ENABLE_ASPIRATION_WINDOWS && evaluation <= alpha {
                // The best found evaluation is less then or equal to the lower bound (alpha),
                // so we need to research at the same depth.
                (alpha, beta, delta) = self.update_aspiration_window_lower_bound(alpha, beta, evaluation, delta);

                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                self.statistics.increment_aspiration_window_fail_low(); /* STATISTICS */
                continue;
            } else if Self::ENABLE_ASPIRATION_WINDOWS && evaluation >= beta {
                (alpha, beta, delta) = self.update_aspiration_window_upper_bound(alpha, beta, evaluation, delta);

                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                self.statistics.increment_aspiration_window_fail_low(); /* STATISTICS */
                continue;
            } else if !Self::ENABLE_ASPIRATION_WINDOWS {
                debug_assert!(evaluation > alpha, "[PVSWorker::update_aspiration_window_lower_bound] Assert evaluation({evaluation}) <= alpha({alpha}) should imply aspiration window but was not.");
                debug_assert!(evaluation < beta,  "[PVSWorker::update_aspiration_window_upper_bound] Assert evaluation({evaluation}) >= beta({beta}) should imply aspiration window but was not.");
            }

            if self.best_evaluation == Some(evaluator_constants::POSITIVE_INFINITY) {
                // We found a winning game, so we can stop searching
                break;
            }

            /* STATISTICS */
            if Self::IS_MAIN_WORKER && Self::ENABLE_SEARCH_STATISTICS {
                let _ = self.write_statistics(&game, depth); // ignore errors
            }

            if Self::ENABLE_ASPIRATION_WINDOWS {
                // Evaluation is within the aspiration window,
                // so we can move on to the next depth with a window set around the evaluation
                (alpha, beta, delta) = self.reset_aspiration_window(alpha, beta, evaluation, delta);
            }

            depth += 1;
            self.statistics.reset_iterative_deepening_iteration(); /* STATISTICS */
        }

        /* STATISTICS */
        if Self::IS_MAIN_WORKER && Self::ENABLE_SEARCH_STATISTICS {
            let _ = self.write_statistics(&game, depth); // ignore errors
        }

        if let Some(action) = self.best_action.take() {
            return Ok(Some((action, self.best_evaluation.take().unwrap())));
        }

        Ok(None)
    }

    /// Updates the lower bound of the aspiration window after it failed with a
    /// [Fail-Low](https://www.chessprogramming.org/Fail-Low#Root_with_Aspiration)
    /// This means the correct evaluation is below the aspiration window.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The current lower bound.
    /// * `beta` - The current upper bound.
    /// * `evaluation` - The evaluation that failed the aspiration window.
    /// * `delta` - The current delta.
    ///
    /// # Returns
    ///
    /// The new lower bound, upper bound and delta.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    fn update_aspiration_window_lower_bound(
        &mut self,
        mut alpha: i32,
        mut beta: i32,
        evaluation: i32,
        mut delta: i32,
    ) -> (i32, i32, i32) {
        // Alpha is adjusted delta away from the evaluation that was returned
        // Beta is also adjusted into the middle right between the old alpha and beta
        // Due to search instability issues we cannot assume that the search can be done with
        // new beta = old alpha (the non-failing bound) adjusted right towards the failing bound
        // Delta is adjusted with the same exponential backoff as in
        // [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L365)

        beta = (alpha + beta) / 2;
        alpha = (evaluation - delta).min(Self::MIN_ALPHA_BOUND);
        delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY);

        self.statistics.increment_aspiration_window_fail_low();

        (alpha, beta, delta)
    }

    /// Updates the upper bound of the aspiration window after it failed with a
    /// [Fail-High](https://www.chessprogramming.org/Fail-High#Root_with_Aspiration)
    ///
    /// This means the correct evaluation is above the aspiration window.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The current lower bound.
    /// * `beta` - The current upper bound.
    /// * `evaluation` - The evaluation that failed the aspiration window.
    /// * `delta` - The current delta.
    ///
    /// # Returns
    ///
    /// The new lower bound, upper bound and delta.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    fn update_aspiration_window_upper_bound(
        &mut self,
        alpha: i32,
        mut beta: i32,
        evaluation: i32,
        mut delta: i32,
    ) -> (i32, i32, i32) {
        // this should never happen if aspiration windows are off
        debug_assert!(
            ENABLE_ASPIRATION_WINDOWS,
            "[PVSWorker::update_aspiration_window_upper_bound] Assert evaluation({evaluation}) >= beta({beta}) should imply aspiration window but was not."
        );

        // Beta is adjusted delta away from the evaluation that was returned
        // Alpha is not adjusted just like in [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L359-L360)

        beta = (evaluation + delta).min(Self::MAX_BETA_BOUND);
        delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY);

        self.statistics.increment_aspiration_window_fail_high();

        (alpha, beta, delta)
    }

    /// Resets the aspiration window around the given evaluation after a
    /// successful search.
    ///
    /// # Arguments
    ///
    /// * `alpha` - The current lower bound.
    /// * `beta` - The current upper bound.
    /// * `evaluation` - The evaluation that was returned.
    /// * `delta` - The current delta.
    ///
    /// # Returns
    ///
    /// The new lower bound, upper bound and delta.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[allow(unused_assignments)]
    #[allow(clippy::unused_self)]
    fn reset_aspiration_window(
        &mut self,
        mut alpha: i32,
        mut beta: i32,
        evaluation: i32,
        mut delta: i32,
    ) -> (i32, i32, i32) {
        delta =
            (Self::ASPIRATION_WINDOWS_MIN_DELTA + evaluation.abs() / 10).min(evaluator_constants::POSITIVE_INFINITY);

        alpha = evaluation - delta;
        beta = evaluation + delta;

        (alpha, beta, delta)
    }

    // ──────────────────────────────── PRINCIPAL VARIATION SEARCH  ────────────────────────────────

    /// Does a Principal Variation Search (PVS) with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `ply_from_root` - The ply from the root node.
    /// * `depth` - The depth to search.
    /// * `alpha` - The lower bound.
    /// * `beta` - The upper bound.
    /// * `num_extensions` - The number of search extensions that have been
    ///    applied so far.
    ///
    /// # Returns
    ///
    /// The evaluation of the game state.
    ///
    /// # Complexity
    ///
    /// Worst-case: `𝒪(𝑏ᵈ)` where `𝑏` is the branching factor and `𝑑` is the
    ///          depth to search.
    /// Best-case: `𝒪(√𝑏ᵈ)` where `𝑏` is the branching factor and `𝑑` is the
    ///         depth to search.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::useless_let_if_seq)]
    #[rustfmt::skip]
    fn principal_variation_search<const ZERO_WINDOW_SEARCH: bool>(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        alpha: i32,
        beta: i32,
        num_extensions: usize,
    ) -> PlayerResult<i32> {
        self.statistics.increment_nodes_searched(); /* STATISTICS */
        self.search_recorder.push_state(game.clone()); /* SEARCH RECORDER */

        // search canceled, return as fast as possible
        if self.search_canceled.load(Ordering::Relaxed) {
            self.search_recorder.pop_state_with_value(0, alpha, beta, format!("Search Canceled ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
            return Ok(0);
        }

        // skip phantom moves
        if matches!(game.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom) {
            let evaluation = self.phantom_skip::<ZERO_WINDOW_SEARCH>(game, ply_from_root, depth, alpha, beta, num_extensions)?;
            self.search_recorder.pop_state_with_value(evaluation, alpha, beta, format!("Phantom Action ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
            return Ok(evaluation);
        }

        // Transposition table lookup
        if Self::ENABLE_TRANSPOSITION_TABLE {
            if let Some((table_action, table_evaluation)) =
                self.transposition_table.probe_hash_entry(game, alpha, beta, depth)
            {
                // cannot happen in Zero window search anyways since alpha = beta - 1
                if ply_from_root == 0 && !ZERO_WINDOW_SEARCH {
                    self.best_action = Some(table_action);
                    self.best_evaluation = Some(table_evaluation);
                }
                self.search_recorder.pop_state_with_value(table_evaluation, alpha, beta, format!("TT-Hit ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
                return Ok(table_evaluation);
            }
        }

        if depth == 0 || game.is_terminated() {
            let evaluation = self.evaluation(game);
            self.search_recorder.pop_state_with_value(evaluation, alpha, beta, format!("Evaluation ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
            return Ok(evaluation);
        }

        let mut actions = game.get_valid_actions();
        let mut scores = vec![0.0; actions.len()];
        let mut action_list = self.get_action_list(game, ply_from_root, &mut actions, &mut scores);
        let mut is_pv_node = true;
        let mut best_action = ActionId::null();
        let mut alpha = alpha;
        let mut evaluation_bound = EvaluationType::UpperBound;
        let mut lmp_flags = self.get_late_move_pruning_flags(&action_list);

        for i in 0..action_list.len() {
            let Some(action) = self.get_next_action(ply_from_root, i, &mut action_list, &mut lmp_flags) else {
                break;
            };

            // Save previous state characteristics that are needed later
            let previous_special_tile_condition_reached = game.is_special_tile_condition_reached();

            game.do_action(action, true)?;

            let (next_depth, extension) = self.get_next_depth::<ZERO_WINDOW_SEARCH>(
                game,
                depth,
                !is_pv_node,
                num_extensions,
                previous_special_tile_condition_reached,
            );

            let mut evaluation = 0;
            if is_pv_node {
                // Full window search for pv node in non-zero window search
                evaluation = -self.principal_variation_search::<ZERO_WINDOW_SEARCH>(
                    game,
                    ply_from_root + 1,
                    next_depth,
                    -beta,
                    -alpha,
                    num_extensions + extension,
                )?;
            } else {
                self.statistics.increment_zero_window_search(); /* STATISTICS */
                let mut needs_full_search = true;

                // Apply [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions)
                // Code adapted from [An Introduction to Late Move Reductions](https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html)
                if self.should_late_move_reduce(game, i, ply_from_root, extension) {
                    // Search this move with reduced depth
                    let next_depth = self.get_late_move_reduced_depth(next_depth);
                    evaluation = -self.zero_window_search(
                        game,
                        ply_from_root + 1,
                        next_depth,
                        -alpha,
                        num_extensions + extension,
                    )?;
                    needs_full_search = evaluation > alpha;
                    if needs_full_search {
                        self.statistics.increment_late_move_reduction_fails(); /* STATISTICS */
                    }
                }

                if needs_full_search {
                    // [Null Window](https://www.chessprogramming.org/Null_Window) at full search depth
                    evaluation = -self.zero_window_search(
                        game,
                        ply_from_root + 1,
                        next_depth, // do not apply search extensions in zws
                        -alpha,
                        num_extensions + extension,
                    )?;

                    if evaluation > alpha && evaluation < beta && !ZERO_WINDOW_SEARCH {
                        // Cannot happen in Zero window search anyways since alpha = beta - 1 < evaluation < beta

                        // Zero-Window-Search failed, re-search with full window
                        let (next_depth, extension) = self.get_next_depth::<ZERO_WINDOW_SEARCH>(
                            game,
                            depth,
                            false,
                            num_extensions,
                            previous_special_tile_condition_reached,
                        );

                        self.statistics.increment_zero_window_search_fail(); /* STATISTICS */
                        evaluation = -self.principal_variation_search::<ZERO_WINDOW_SEARCH>(
                            game,
                            ply_from_root + 1,
                            next_depth + extension,
                            -beta,
                            -alpha,
                            num_extensions + extension,
                        )?;
                    }
                }
            }

            game.undo_action(action, true)?;

            if self.search_canceled.load(Ordering::Relaxed) {
                self.search_recorder.pop_state_with_value(0, alpha, beta, format!("Search Canceled ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
                return Ok(0);
            }

            if evaluation >= beta {
                self.statistics.increment_fail_high(is_pv_node); /* STATISTICS */

                self.store_transposition_table(game, depth, beta, EvaluationType::LowerBound, action);

                return Ok(if Self::SOFT_FAILING_STRATEGY {
                    self.search_recorder.pop_state_with_value(evaluation, alpha, beta, format!("Fail-Soft Beta-Cutoff ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
                    evaluation // Fail-soft beta-cutoff
                } else {
                    self.search_recorder.pop_state_with_value(beta, alpha, beta, format!("Fail-Hard Beta-Cutoff ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */
                    beta // Fail-hard beta-cutoff
                });
            }

            // Cannot happen in Zero window search anyways since alpha = beta - 1
            if evaluation > alpha && !ZERO_WINDOW_SEARCH {
                evaluation_bound = EvaluationType::Exact;
                alpha = evaluation; // alpha acts like max in minimax
                best_action = action;
            }

            is_pv_node = false;
        }

        // In case of a UpperBound we store a null action, as the true best
        // action is unknown
        self.store_transposition_table(game, depth, alpha, evaluation_bound, best_action);

        // Cannot happen in Zero window search anyways since ply is 0
        if ply_from_root == 0 && !ZERO_WINDOW_SEARCH {
            self.best_action = Some(best_action);
            self.best_evaluation = Some(alpha);
        }

        // Check assumptions
        // If we are in the first ply then the evaluation_bound has to be exact
        // or iff aspiration windows are used can also be an upper bound
        debug_assert!(
            ply_from_root != 0 || evaluation_bound == EvaluationType::Exact || Self::ENABLE_ASPIRATION_WINDOWS,
            "[PVSWorker::principal_variation_search] Assert about ply_from_root: {}, evaluation_bound: {:?}, aspiration_windows: {}, alpha: {:?}.",
            ply_from_root,
            evaluation_bound,
            Self::ENABLE_ASPIRATION_WINDOWS,
            alpha
        );

        self.search_recorder.pop_state_with_value(alpha, alpha, beta, format!("Full Search ({ZERO_WINDOW_SEARCH})").as_str()); /* SEARCH RECORDER */

        Ok(alpha)
    }

    /// Does a Zero/Scout Window Search (ZWS) with the given parameters.
    ///
    /// fail-hard zero window search, returns either `beta-1` or `beta`
    /// only takes the beta parameter because `alpha == beta - 1`
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `ply_from_root` - The ply from the root node.
    /// * `depth` - The depth to search.
    /// * `beta` - The upper bound.
    /// * `num_extensions` - The number of search extensions that have been
    ///     applied so far.
    ///
    /// # Returns
    ///
    /// The evaluation of the game state.
    ///
    /// # Complexity
    ///
    /// Same as `principal_variation_search`
    #[inline]
    fn zero_window_search(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        beta: i32,
        num_extensions: usize,
    ) -> PlayerResult<i32> {
        let alpha = beta - 1;

        self.principal_variation_search::<true>(game, ply_from_root, depth, alpha, beta, num_extensions)
    }

    /// Skips the current search if it is a phantom move.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `ply_from_root` - The ply from the root node.
    /// * `depth` - The depth to search.
    /// * `alpha` - The lower bound.
    /// * `beta` - The upper bound.
    /// * `num_extensions` - The number of search extensions that have been
    ///    applied so far.
    ///
    /// # Returns
    ///
    /// The evaluation of the game state.
    ///
    /// # Complexity
    ///
    /// Same as `principal_variation_search`
    #[inline]
    #[rustfmt::skip]
    fn phantom_skip<const ZERO_WINDOW_SEARCH: bool>(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        alpha: i32,
        beta: i32,
        num_extensions: usize,
    ) -> PlayerResult<i32> {
        game.do_action(ActionId::phantom(), true)?;

        let evaluation = -self.principal_variation_search::<ZERO_WINDOW_SEARCH>(
            game,
            ply_from_root,
            depth,
            alpha,
            beta,
            num_extensions,
        )?;

        game.undo_action(ActionId::phantom(), true)?;

        Ok(evaluation)
    }

    // ──────────────────────────────────────── EVALUATION  ────────────────────────────────────────

    /// Evaluates the given game state.
    ///
    /// # Possible Improvement
    ///
    /// Implement another function "quiescence search" that searches for a stable evaluation before
    /// calling this evaluation function. This would mitigate the horizon effect.
    /// The question is if this is even needed in patchwork
    ///
    /// # Arguments
    ///
    /// * `game` - The game to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the game state.
    ///
    /// # Complexity
    ///
    /// The same complexity as the evaluator used.
    fn evaluation(&mut self, game: &Patchwork) -> i32 {
        // State cannot be in a phantom state here
        debug_assert!(matches!(
            game.turn_type,
            TurnType::Normal | TurnType::SpecialPatchPlacement
        ));

        self.statistics.increment_leaf_nodes_searched(); /* STATISTICS */

        let color = if game.is_player_1() { 1 } else { -1 };
        let evaluation = color * self.evaluator.evaluate_node(game);

        // self.store_transposition_table(game, 0, evaluation, EvaluationType::Exact, ActionId::null());

        debug_assert!(
            (evaluator_constants::NEGATIVE_INFINITY..=evaluator_constants::POSITIVE_INFINITY).contains(&evaluation),
            "[PVSWorker::evaluation] Assert evaluation({}) is not in range of [{}, {}].",
            evaluation,
            evaluator_constants::NEGATIVE_INFINITY,
            evaluator_constants::POSITIVE_INFINITY
        );

        evaluation
    }

    // ───────────────────────────────────── LATE MOVE PRUNING ─────────────────────────────────────

    /// Gets the late move pruning flags for the given action list.
    ///
    /// # Arguments
    ///
    /// * `action_list` - The action list to get the flags for.
    ///
    /// # Returns
    ///
    /// The late move pruning flags.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[allow(clippy::unused_self)]
    fn get_late_move_pruning_flags(&self, action_list: &ActionList<'_>) -> LMPFlags<LMP_AMOUNT_OF_ACTIONS_PER_PATCH> {
        if ENABLE_LATE_MOVE_PRUNING {
            LMPFlags::<LMP_AMOUNT_OF_ACTIONS_PER_PATCH>::initialize_from(action_list)
        } else {
            LMPFlags::fake()
        }
    }

    // ────────────────────────────────────── ACTION ORDERING ──────────────────────────────────────

    /// Gets the action list for the given game state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to get the action list for.
    /// * `ply_from_root` - The ply from the root node.
    ///
    /// # Returns
    ///
    /// The action list for the given game state as well as the vectors with actions and scores.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `𝑛` is the amount of actions and `𝒪(𝑚)` is the complexity of the `score_action` function which is usually `𝒪(𝟣)`.
    fn get_action_list<'a>(
        &self,
        game: &Patchwork,
        ply_from_root: usize,
        actions: &'a mut [ActionId],
        scores: &'a mut [f64],
    ) -> ActionList<'a> {
        let mut action_list = ActionList::new(actions, scores);

        let pv_action = self.get_pv_action(game, ply_from_root);

        self.action_orderer.score_actions(game, &mut action_list, pv_action, ply_from_root);

        action_list
    }

    /// Gets the next action to search from the action list. Optionally None if late move pruning should be applied
    ///
    /// # Arguments
    ///
    /// * `ply_from_root` - The ply from the root node.
    /// * `action_index` - The index of the current action.
    /// * `action_list` - The list of actions to search.
    /// * `lmp_flags` - The flags for late move pruning.
    ///
    /// # Returns
    ///
    /// The next action to search or None if late move pruning should be applied.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the number of available actions to choose from
    fn get_next_action(
        &mut self,
        ply_from_root: usize,
        action_index: usize,
        action_list: &mut ActionList<'_>,
        lmp_flags: &mut LMPFlags<LMP_AMOUNT_OF_ACTIONS_PER_PATCH>,
    ) -> Option<ActionId> {
        if Self::ENABLE_LATE_MOVE_PRUNING
            && ply_from_root >= Self::LMP_APPLY_AFTER_PLYS
            && action_index >= Self::LMP_AMOUNT_NON_PRUNED_ACTIONS
        {
            // [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning)
            // Remove late moves in move ordering, only prune after trying at least one move for each possible patch
            let action = lmp_flags.get_next_missing();

            if action.is_none() {
                self.statistics.increment_late_move_pruning(); /* STATISTICS */
            }

            return action;
        }

        // Move Ordering
        // [How move ordering works](https://rustic-chess.org/search/ordering/how.html)
        // > Sidenote: Why do we assign sort scores to the moves, and then use pick_move() to swap one move to the
        // > current index of the move list while alpha/beta iterates over it? Couldn't we just physically sort the
        // > list before the move loop starts, and be done with it?
        // > It's possible, but we don't do that because of how alpha/beta works. If alpha/beta encounters a move
        // > that is so good that searching further down the move list is no longer required, then it will exit and
        // > return the evaluation score of that move. (This is a so-called beta-cutoff.) If you physically sorted
        // > all the moves before the move loop starts, you may have sorted lots of moves that may never be examined
        // > by alpha/beta. This costs time, and thus it makes the engine slower.
        // > You can hear and read "move ordering" and "move sorting" interchangeably. The difference is that "move
        // > ordering" does the "score and pick" approach, and "move sorting" physically sorts the entire move list.
        // > The result is the same, but move ordering is the faster approach, as described above.
        Some(self.action_orderer.pick_action(action_list, action_index))
    }

    // ───────────────────────────────────── SEARCH EXTENSIONS ─────────────────────────────────────

    /// Gets the next depth that should be used.
    /// This depth is the depth reduced by the usual amount together with the
    /// applied search extensions. The number of extensions is returned as well.
    ///
    /// # Returns
    ///
    /// The next depth and the number of search extensions that were applied.
    fn get_next_depth<const ZERO_WINDOW_SEARCH: bool>(
        &mut self,
        game: &Patchwork,
        depth: usize,
        will_zero_window_search: bool,
        num_extensions: usize,
        previous_special_tile_condition_reached: bool,
    ) -> (usize, usize) {
        let depth = depth - 1; // usual depth reduction

        if will_zero_window_search {
            return (depth, 0);
        }

        let extension = self.get_search_extension::<ZERO_WINDOW_SEARCH>(
            game,
            num_extensions,
            previous_special_tile_condition_reached,
        );

        (depth + extension, extension)
    }

    /// Extends the depth of the search in certain interesting cases
    ///
    /// Currently only extends the depth of the search for special patch placements
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `num_extensions` - The number of search extensions already applied.
    ///
    /// # Returns
    ///
    /// The number of search extensions (depth) to apply.
    fn get_search_extension<const ZERO_WINDOW_SEARCH: bool>(
        &mut self,
        game: &Patchwork,
        num_extensions: usize,
        previous_special_tile_condition_reached: bool,
    ) -> usize {
        if ZERO_WINDOW_SEARCH {
            return 0;
        }

        if !Self::ENABLE_SEARCH_EXTENSIONS {
            return 0;
        }

        if num_extensions >= Self::MAX_SEARCH_EXTENSIONS {
            return 0;
        }

        let mut extension = 0;

        // Extend the depth of search for special patch placements
        if matches!(game.turn_type, TurnType::SpecialPatchPlacement) {
            self.statistics.increment_special_patch_extensions();
            extension += 1;
        }

        // Extend the depth of search if the 7×7 special tile was given
        if !previous_special_tile_condition_reached && game.is_special_tile_condition_reached() {
            self.statistics.increment_special_tile_extensions();
            extension += 1;
        }

        extension
    }

    // ─────────────────────────────────── LATE MOVE REDUCTIONS  ───────────────────────────────────

    /// Determines if Late Move Reductions (LMR) should be applied.
    ///
    /// This means that we're not in the early moves (and this is not a PV node)
    /// Reduce the depth of the search for later actions as these are
    /// less likely to be good (assuming the action ordering is good)
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `action_index` - The index of the current action.
    /// * `ply_from_root` - The ply from the root node.
    /// * `search_extensions` - The number of search extensions that should be applied to the search.
    ///
    /// # Returns
    ///
    /// If Late Move Reductions (LMR) should be applied.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[allow(clippy::unused_self)]
    fn should_late_move_reduce(
        &mut self,
        game: &Patchwork,
        action_index: usize,
        ply_from_root: usize,
        search_extensions: usize,
    ) -> bool {
        if action_index < Self::LMR_AMOUNT_FULL_DEPTH_ACTIONS {
            return false;
        }

        if ply_from_root < Self::LMR_APPLY_AFTER_PLYS {
            return false;
        }

        if search_extensions == 0 {
            self.statistics.increment_late_move_reductions();
            return true;
        }

        if search_extensions == 1 && matches!(game.turn_type, TurnType::SpecialPatchPlacement) {
            // special patch search extension is not relevant for late move reduction
            self.statistics.increment_late_move_reductions();
            return true;
        }

        false
    }

    /// Gets the reduced depth for Late Move Reductions (LMR).
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth of the search.
    ///
    /// # Returns
    ///
    /// The reduced depth.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[allow(clippy::unused_self)]
    const fn get_late_move_reduced_depth(&self, depth: usize) -> usize {
        if depth >= 6 {
            return depth.saturating_sub(depth / 3);
        }

        depth.saturating_sub(1)
    }

    // ──────────────────────────────────── TRANSPOSITION TABLE ────────────────────────────────────

    /// Gets the principal variation action for the given game state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to get the principal variation action for.
    /// * `ply_from_root` - The ply from the root node.
    ///
    /// # Returns
    ///
    /// The principal variation action for the given game state or `None` if no
    /// principal variation action could be found.
    fn get_pv_action(&self, game: &Patchwork, ply_from_root: usize) -> Option<ActionId> {
        if ply_from_root == 0 {
            if let Some(pv_action) = self.best_action {
                return Some(pv_action);
            }
        }

        // Uses Transposition Table for PV-Action, for more information see
        // [TT-move ordering: Sidenote](https://rustic-chess.org/search/ordering/tt_move.html)
        // > Sidenote: what about ordering on the PV-move? There is a technique called PV-move ordering, which orders
        // > the best move from the previous iteration in the first spot. Ordering the move works the same was is with
        // > the TT-move; the only difference is that you pass the PV-move to the score_move() function instead of the
        // > TT-move. This is easier to implement, because you don't need a TT for it. As the TT stores the PV-moves
        // > (and the cut-moves), PV-move ordering is inherently built into TT-move ordering. If you have a TT, PV-move
        // > ordering becomes superfluous.
        // >
        // > There is a chance your the TT-entry holding the PV-move for the position you are in was overwritten with a
        // > different entry so you have no PV-move to order on. As far as I know, many engines take this risk for
        // > granted and don't implement PV-move ordering is a fallback. It's probably not worth it with regard to Elo
        // > gain. Rustic does not implement PV-move ordering.
        if Self::ENABLE_TRANSPOSITION_TABLE {
            return self.transposition_table.probe_pv_move(game);
        }

        None
    }

    /// Stores the given search position inside the transposition table if the
    /// transposition table is enabled.
    ///
    /// Also stores the symmetries of the given action if the transposition
    /// table with symmetries is enabled.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to store the position for.
    /// * `depth` - The depth at which the evaluation was calculated
    /// * `evaluation` - The evaluation of the position
    /// * `lower_bound` - The lower bound of the evaluation
    /// * `action` - The best action to take in this position
    fn store_transposition_table(
        &mut self,
        game: &Patchwork,
        depth: usize,
        evaluation: i32,
        evaluation_type: EvaluationType,
        action: ActionId,
    ) {
        if !Self::ENABLE_TRANSPOSITION_TABLE {
            return;
        }

        if Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_ENABLED {
            self.transposition_table.store_evaluation(game, depth, evaluation, evaluation_type, action);
        } else if Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_SYMMETRY_ENABLED {
            self.transposition_table
                .store_evaluation_with_symmetries(game, depth, evaluation, evaluation_type, action);
        } else {
            unreachable!(
                "[PVSWorker::store_transposition_table] Transposition table symmetry type is invalid: {} (valid 'd','e' and 's')",
                Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE
            );
        }
    }

    /// Gets the principal variation actions for the given game state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to get the principal variation actions for.
    /// * `depth` - The depth to get the principal variation actions for.
    ///
    /// # Returns
    ///
    /// The principal variation actions for the given game state or an empty
    /// vector if no principal variation actions could be found.
    ///
    /// # Notes
    ///
    /// If the depth is 0 the method assumes that the game is in the root node.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the depth of the PV line.
    fn get_pv_action_line(&self, game: &Patchwork, depth: usize) -> String {
        if Self::ENABLE_TRANSPOSITION_TABLE {
            return self
                .transposition_table
                .get_pv_line(game, depth)
                .iter()
                .map(|action| action.save_to_notation().map_or_else(|_| "######".to_string(), |notation| notation))
                .join(" → ");
        }

        if depth == 0 {
            if let Some(pv_action) = self.best_action {
                return pv_action
                    .save_to_notation()
                    .map_or_else(|_| "###### → ...".to_string(), |notation| format!("{notation} → ..."));
            }
        }

        "NONE".to_string()
    }

    // ───────────────────────────────────── SEARCH STATISTICS ─────────────────────────────────────

    /// Writes the statistics to the logging writer.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to write the statistics for.
    /// * `depth` - The depth of the search.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the writing.
    #[inline]
    #[rustfmt::skip]
    fn write_statistics(
        &mut self,
        game: &Patchwork,
        depth: usize,
    ) -> Result<(), std::io::Error> {

        let pv_actions = self.get_pv_action_line(game, depth);
        let best_evaluation = self.best_evaluation.map_or("NONE".to_string(), |eval| format!("{eval}"));
        let best_action = self.best_action.as_ref().map_or("NONE".to_string(), |action| action.save_to_notation().map_or_else(|_| "######".to_string(), |notation| notation));

        let Some(logging) = self.logging.as_mut() else {
            return Ok(());
        };

        let mut debug_writer = None;
        let writer = match logging {
            Logging::Disabled => return Ok(()),
            Logging::Enabled { progress_writer: ref mut writer } => writer.as_mut(),
            Logging::Verbose { progress_writer: ref mut writer, debug_writer: ref mut d_writer } => {
                debug_writer = Some(d_writer);
                writer.as_mut()
            },
            Logging::VerboseOnly { ref mut debug_writer } => {
                if Self::ENABLE_TRANSPOSITION_TABLE {
                    self.transposition_table.statistics.write_transposition_table(debug_writer, self.transposition_table.as_ref(), None)?;
                }
                return Ok(())
            }
        };

        let mut features = vec![];
        if Self::ENABLE_ASPIRATION_WINDOWS {
            features.push("AW");
        }
        if Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_ENABLED {
            features.push("TT");
        } else if Self::TRANSPOSITION_TABLE_SYMMETRY_TYPE == Self::TRANSPOSITION_TABLE_SYMMETRY_ENABLED {
            features.push("TT(S)");
        }
        if Self::ENABLE_LATE_MOVE_REDUCTIONS {
            features.push("LMR");
        }
        if Self::ENABLE_LATE_MOVE_PRUNING {
            features.push("LMP");
        }
        if Self::ENABLE_SEARCH_EXTENSIONS {
            features.push("SE");
        }
        let features = features.join(", ");

        // [Branching Factor](https://www.chessprogramming.org/Branching_Factor)
        let average_branching_factor = (self.statistics.leaf_nodes_searched as f64).powf(1.0 / depth as f64);
        let effective_branching_factor = self.statistics.nodes_searched as f64 / self.statistics.nodes_searched_previous_iteration as f64;
        let mean_branching_factor = self.statistics.nodes_searched as f64 / (self.statistics.nodes_searched - self.statistics.leaf_nodes_searched) as f64;
        let player_1_pos = game.player_1.get_position();
        let player_2_pos = game.player_2.get_position();

        writeln!(writer, "───────────── Principal Variation Search Player ─────────────")?;
        writeln!(writer, "Features:            [{features}]")?;
        writeln!(writer, "Depth:               {:?} started from (1: {}, 2: {}, type: {:?})", depth, player_1_pos, player_2_pos, game.turn_type)?;
        writeln!(writer, "Time:                {:?}", std::time::Instant::now().duration_since(self.statistics.start_time))?;
        writeln!(writer, "Nodes searched:      {:?}", self.statistics.nodes_searched)?;
        writeln!(writer, "Branching factor:    {average_branching_factor:.2} AVG / {effective_branching_factor:.2} EFF / {mean_branching_factor:.2} MEAN")?;
        writeln!(writer, "Best Action:         {best_action} ({best_evaluation} pts)")?;
        writeln!(writer, "Move Ordering:       {:.2?}% ({} high pv / {} high)", (self.statistics.fail_high_first as f64) / (self.statistics.fail_high as f64) * 100.0, self.statistics.fail_high_first, self.statistics.fail_high)?;
        writeln!(writer, "Aspiration window:   {:?} low / {:?} high", self.statistics.aspiration_window_fail_low, self.statistics.aspiration_window_fail_high)?;
        writeln!(writer, "Zero window search:  {:?} fails ({:.2}%)", self.statistics.zero_window_search_fail, self.statistics.zero_window_search_fail_rate() * 100.0)?;
        writeln!(writer, "Search Extensions:   {:?} SP, {:?} ST ({})", self.statistics.special_patch_extensions, self.statistics.special_tile_extensions, if Self::ENABLE_SEARCH_EXTENSIONS { "enabled" } else { "disabled" })?;
        writeln!(writer, "LMR (Fail/All):      {:?}/{:?} ({:.2}%)", self.statistics.late_move_reduction_fails, self.statistics.late_move_reductions, self.statistics.late_move_reduction_fail_rate() * 100.0)?;
        writeln!(writer, "LMP:                 {:?}", self.statistics.late_move_pruning)?;
        writeln!(writer, "Principal Variation: {pv_actions}")?;
        if Self::ENABLE_TRANSPOSITION_TABLE {
            self.transposition_table.statistics.write_statistics(writer)?;
            if let Some(debug_writer) = debug_writer {
                self.transposition_table.statistics.write_transposition_table(debug_writer, self.transposition_table.as_ref(), None)?;
            }
        }
        writeln!(writer, "─────────────────────────────────────────────────────────────")?;

        Ok(())
    }
}
