use std::sync::{atomic::AtomicBool, Arc};

use itertools::Itertools;

use patchwork_core::{evaluator_constants, ActionId, Notation, Patchwork, Player, PlayerResult, TurnType};
use pv_table::PVTable;
use transposition_table::{EvaluationType, TranspositionTable};

use crate::{
    pvs_options::FailingStrategy, DiagnosticsFeature, PVSOptions, SearchDiagnostics, TranspositionTableFeature,
};

/// A computer player that uses the Principal Variation Search (PVS) algorithm to choose an action.
///
/// # Features
/// - [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening)
/// - [Alpha-Beta Pruning](https://www.chessprogramming.org/Alpha-Beta)
/// - [NegaMax](https://www.chessprogramming.org/Negamax)
/// - [Principal Variation Search (PVS)](https://www.chessprogramming.org/Principal_Variation_Search)
/// - [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows)
/// - [Transposition Table](https://www.chessprogramming.org/Transposition_Table)
/// - [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions)
/// - [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning)
/// - [Search Extension](https://www.chessprogramming.org/Extensions) - Win-seeking search extensions for special patch placements
/// - [Move Ordering](https://www.chessprogramming.org/Move_Ordering)
///     - PV Actions sorted first
///
/// TODO:
/// - https://www.chessprogramming.org/Legal_Move#Legality_Test for transposition table entries from key collisions
/// # Features that still need to be implemented
/// - PV Table
/// - Move Ordering
///     - Something like MMV-LVA but for patchwork (e.g. ending score)
///     - Actions that are inside the transposition table
///     - Killer Moves (TODO)
///     - Thread escape move check
///     - History Heuristic
///     - Move Ordering via Machine Learning (something like [Neural MoveMap Heuristic](https://www.chessprogramming.org/Neural_MoveMap_Heuristic))
///  Remove late moves in move ordering
/// - [Internal Iterative Deepening (IID)](https://www.chessprogramming.org/Internal_Iterative_Deepening)
/// - [Null Move Pruning](https://www.chessprogramming.org/Null_Move_Pruning) if it brings something
/// - [Lazy SMP](https://www.chessprogramming.org/Lazy_SMP) - spawn multiple threads in iterative deepening, share transposition table, take whichever finishes first
/// - [Automated Tuning](https://www.chessprogramming.org/Automated_Tuning) via regression, reinforcement learning or supervised learning for evaluation
///   - [Texel's Tuning Method](https://www.chessprogramming.org/Texel%27s_Tuning_Method)
/// - [ƎUИИ Efficiently Updatable Neural Networks](https://www.chessprogramming.org/NNUE) implementation in Rust [Carp Engine](https://github.com/dede1751/carp/tree/main/chess/src/nnue)
///
/// # Features that could maybe be implemented (look into it what it is)
///    -   (Reverse) Futility Pruning --> pretty sure
///    -   Delta Pruning
///    -   https://www.chessprogramming.org/PV_Extensions
pub struct PVSPlayer {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// search diagnostics
    pub diagnostics: SearchDiagnostics,
    /// The principal variation table.
    #[allow(unused)] // FEATURE:PV_TABLE
    pv_table: PVTable,
    /// The transposition table.
    transposition_table: Option<TranspositionTable>,
    /// The best action found so far.
    best_action: Option<ActionId>,
    /// The best evaluation found so far.
    best_evaluation: Option<i32>,
    /// Whether the search has been canceled.
    search_canceled: Arc<AtomicBool>,
}

impl PVSPlayer {
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<PVSOptions>) -> Self {
        let options = options.unwrap_or_default();
        let transposition_table = match options.features.transposition_table {
            TranspositionTableFeature::Disabled => None,
            TranspositionTableFeature::Enabled { size, strategy }
            | TranspositionTableFeature::SymmetryEnabled { size, strategy } => {
                Some(TranspositionTable::new(size, strategy == FailingStrategy::FailSoft))
            }
        };

        PVSPlayer {
            name: name.into(),
            options,
            diagnostics: Default::default(),
            pv_table: Default::default(),
            transposition_table,
            best_action: None,
            best_evaluation: None,
            search_canceled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for PVSPlayer {
    fn default() -> Self {
        Self::new(
            "Principal Variation Search (PVS) Player".to_string(),
            Default::default(),
        )
    }
}

impl Player for PVSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        // PERF: shortcut for only one available action
        let valid_action = game.get_valid_actions();
        if valid_action.len() == 1 {
            return Ok(valid_action[0]);
        }

        std::thread::scope(|s| {
            let search_canceled = Arc::clone(&self.search_canceled);
            let time_limit = self.options.time_limit;

            // reset the parameters for the search
            self.search_canceled.store(false, std::sync::atomic::Ordering::SeqCst);
            self.best_evaluation = None;
            self.best_action = None;

            // reset all diagnostics
            self.diagnostics.reset();
            if let Some(ref mut transposition_table) = self.transposition_table {
                transposition_table.reset_diagnostics();
                transposition_table.increment_age();
            }

            // TODO: record a max depth reached inside pvs search, if there are no more actions available, we can stop searching (endgame)
            s.spawn(move || {
                // Periodic check if the search was already canceled by itself
                let start_time = std::time::Instant::now();
                while start_time.elapsed() < time_limit && !search_canceled.load(std::sync::atomic::Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                // Stop search after time limit
                search_canceled.store(true, std::sync::atomic::Ordering::Release);
            });

            let mut game = game.clone();

            // do the search
            let result = self.search(&mut game);

            // force stop the timer thread if the search canceled itself
            self.search_canceled.store(true, std::sync::atomic::Ordering::Release);

            result
        })
    }
}

impl PVSPlayer {
    /// The amount of actions where the following search is for sure not reduced
    /// by LMR. After these amount of actions the LMR is applied.
    pub const LMR_AMOUNT_FULL_DEPTH_ACTIONS: usize = 3;
    /// The maximum depth at which LMR will be applied.
    pub const LMR_DEPTH_LIMIT: usize = 2;

    /// The amount of actions that are for sure not pruned by LMP. After these
    /// amount of actions the LMP is applied.
    pub const LMP_AMOUNT_NON_PRUNED_ACTIONS: usize = 7;
    /// The maximum depth at which LMP will be applied.
    pub const LMP_DEPTH_LIMIT: usize = 2;

    pub const MAX_SEARCH_EXTENSIONS: usize = 16; // can never be reached as we only have a search extension for special patches that cannot activate another special patch placement

    /// The maximum depth to search.
    /// This is an upper bound that can never be reached as no game can go on
    /// longer than 54*2 turns with phantom moves 108*2=216
    ///
    /// This is equal to the maximum amount of ply's that is searched (including phantom actions)
    pub const MAX_DEPTH: usize = 256;

    // TODO: better estimations for delta, alpha and beta
    /// Starting value for alpha (lower bound)
    pub const STARTING_ALPHA: i32 = -60;
    /// Starting value for beta (upper bound)
    pub const STARTING_BETA: i32 = 60;
    /// Minimum delta for aspiration windows
    pub const MINIMUM_DELTA: i32 = 1;

    /// The minimum bound for alpha. Ensures that the minimum alpha value is
    /// less than the minimum evaluation to avoid a fail-low with the maximum
    /// window size.
    pub const MIN_ALPHA_BOUND: i32 = evaluator_constants::NEGATIVE_INFINITY - 1;
    /// The maximum bound for beta. Ensures that the maximum beta value is
    /// greater than the maximum evaluation to avoid a fail-high with the
    /// maximum window size.
    pub const MAX_BETA_BOUND: i32 = evaluator_constants::POSITIVE_INFINITY + 1;

    /// Does a Iterative Deepening Principal Variation Search (PVS) with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `color` - The color of the player to search for. (`+1` for player 1 and `-1` for player 2)
    ///
    /// # Returns
    ///
    /// The best action found by the search or an error if some error occurred.
    fn search(&mut self, game: &mut Patchwork) -> PlayerResult<ActionId> {
        let mut delta = Self::MINIMUM_DELTA;
        let mut alpha = Self::MIN_ALPHA_BOUND;
        let mut beta = Self::MAX_BETA_BOUND;
        let mut depth = 1;

        if self.options.features.aspiration_window {
            alpha = Self::STARTING_ALPHA;
            beta = Self::STARTING_BETA;
        }

        self.diagnostics.reset_iterative_deepening_iteration();

        // [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening) loop
        while depth < Self::MAX_DEPTH {
            let evaluation = self.principal_variation_search(game, 0, depth, alpha, beta, 0)?;

            if self.search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            // TODO: we fail too often
            // [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows) with exponential backoff
            //
            // Due to search instability issues we cannot assume that the search can be done with the non-failing bound
            // adjusted right under/over the other bound
            // [Aspiration Windows](https://web.archive.org/web/20071031095918/http://www.brucemo.com/compchess/programming/aspiration.htm)
            if evaluation <= alpha {
                // Evaluation is below aspiration window [Fail-Low](https://www.chessprogramming.org/Fail-Low#Root_with_Aspiration)
                // The best found evaluation is less than or equal to the lower bound (alpha), so we need to research at the same depth

                debug_assert!(
                    self.options.features.aspiration_window, // this should never happen if aspiration windows are off
                    "[PVSPlayer::search] Assert evaluation({}) <= alpha({}) should imply aspiration window but was not.",
                    evaluation,
                    alpha
                );

                beta = (alpha + beta) / 2; // adjust beta towards alpha
                alpha = (evaluation - delta).min(Self::MIN_ALPHA_BOUND);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY); // use same exponential backoff as in [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L429C17-L429C36)
                self.diagnostics.increment_aspiration_window_fail_low();
                continue;
            } else if evaluation >= beta {
                // Evaluation is above aspiration window [Fail-High](https://www.chessprogramming.org/Fail-High#Root_with_Aspiration)
                // The best found evaluation is greater or equal to the upper bound (beta), so we need to research at the same depth

                debug_assert!(
                    self.options.features.aspiration_window, // this should never happen if aspiration windows are off
                    "[PVSPlayer::search] Assert evaluation({}) >= beta({}) should imply aspiration window but was not.",
                    evaluation,
                    beta
                );

                beta = (evaluation + delta).min(Self::MAX_BETA_BOUND);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY);
                self.diagnostics.increment_aspiration_window_fail_high();
                continue;
            }

            let _ = self.write_diagnostics(game, depth); // ignore errors

            if let Some(evaluator_constants::POSITIVE_INFINITY) = self.best_evaluation {
                // We found a winning game, so we can stop searching
                break;
            }

            if self.options.features.aspiration_window {
                // Evaluation is within the aspiration window,
                // so we can move on to the next depth with a window set around the eval

                delta = Self::MINIMUM_DELTA; // TODO: use avg of root node scores like in Stockfish `delta = Value(9) + int(avg) * avg / 14847;`
                alpha = evaluation - delta;
                beta = evaluation + delta;
            }

            depth += 1;
            self.diagnostics.reset_iterative_deepening_iteration();
        }

        let best_action = self.best_action.take().unwrap_or_else(|| {
            let _ = self.write_single_diagnostic("No best action found. Returning random valid action. This only happends when no full search iteration could be done."); // ignore errors
            game.get_random_action()
        });

        let _ = self.write_single_diagnostic(format!("Best action: {:?}", best_action).as_str()); // ignore errors
        let _ = self.write_diagnostics(game, depth); // ignore errors

        Ok(best_action)
    }

    /// Does a Principal Variation Search (PVS) with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to search in.
    /// * `ply_from_root` - The ply from the root node.
    /// * `depth` - The depth to search.
    /// * `alpha` - The alpha value.
    /// * `beta` - The beta value.
    /// * `color` - The color of the player to search for. (`+1` for player 1 and `-1` for player 2)
    #[allow(clippy::needless_range_loop)]
    fn principal_variation_search(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        alpha: i32,
        beta: i32,
        num_extensions: usize,
    ) -> PlayerResult<i32> {
        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(0);
        }

        // TODO: lookup mates (inspired by Searcher.cs)

        if let Some(ref mut transposition_table) = self.transposition_table {
            if let Some((table_action, table_evaluation)) =
                transposition_table.probe_hash_entry(game, alpha, beta, depth)
            {
                if ply_from_root == 0 {
                    // TODO: split phantom and null moves
                    self.best_action = Some(table_action);
                    self.best_evaluation = Some(table_evaluation);
                }
                return Ok(table_evaluation);
            }
        }

        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        self.diagnostics.increment_nodes_searched();

        let mut actions = game.get_valid_actions();

        // TODO: is this useful for patchwork?
        // NULL MOVE PRUNING
        //       // Null move search:
        //       if(ok_to_do_null_move_at_this_node()) {
        //         make_null_move();
        //         value = -search(-beta, -beta, -(beta-1), depth-4);
        //         unmake_null_move();
        //         if(value >= beta) return value;
        //       }

        let pv_action = self.get_pv_action(game, ply_from_root);

        // TODO: move sorter, move pvNode first (with https://www.chessprogramming.org/Triangular_PV-Table or transposition table)
        self.options.action_sorter.sort_actions(&mut actions, pv_action);

        // TODO: ensure when code is but free
        // // PV-Node should always be sorted first
        // #[cfg(debug_assertions)]
        // if pv_action.is_some() {
        //     let pv_action = pv_action.unwrap();
        //     if actions[0] != pv_action {
        //         println!("PV-Node action is not sorted first!");
        //         println!("PLY_FROM_ROOT {:?}", ply_from_root);
        //         println!("BEST_ACTION: {:?}", self.best_action);
        //         println!("PROBE PV: {:?}", self.transposition_table.probe_pv_move(game));
        //     }

        //     debug_assert_eq!(actions[0], pv_action);
        //

        let mut is_pv_node = true;
        let mut best_action = ActionId::null();
        let mut alpha = alpha;
        let mut evaluation_bound = EvaluationType::UpperBound;

        for i in 0..actions.len() {
            let action = actions[i];

            // TODO: late move pruning (LMP) (remove last actions in list while some conditions are not met, e.g. in check, depth, ...)
            // [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning)
            if i >= Self::LMP_AMOUNT_NON_PRUNED_ACTIONS && self.options.features.late_move_pruning {
                self.diagnostics.increment_late_move_pruning();
                // TODO: check some things here to allow
                // -> make it more like futility pruning
                continue;
            }

            game.do_action(action, true)?;

            // Extend the depth of the search in certain interesting cases (special patch placement)
            let extension = self.get_search_extension(game, num_extensions);

            let mut evaluation = 0;
            if is_pv_node {
                // Full window search for pv node
                evaluation = -self.principal_variation_search(
                    game,
                    ply_from_root + 1,
                    depth - 1 + extension,
                    -beta,
                    -alpha,
                    num_extensions + extension,
                )?
            } else {
                self.diagnostics.increment_zero_window_search();
                // Apply [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions) if we're not in the early moves (and this is not a PV node)
                // Reduce the depth of the search for later actions as these are less likely to be good (assuming the action ordering is good)
                // Code adapted from https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html
                // Search Extensions are not applied to the reduced depth search
                let mut needs_full_search = true;
                if extension == 0
                    && i >= PVSPlayer::LMR_AMOUNT_FULL_DEPTH_ACTIONS
                    && depth >= PVSPlayer::LMR_DEPTH_LIMIT
                {
                    self.diagnostics.increment_late_move_reductions();

                    let lmr_depth_reduction = if depth >= 6 { depth / 3 } else { 1 };
                    // Search this move with reduced depth
                    evaluation = -self.zero_window_search(game, depth - 1 - lmr_depth_reduction, -alpha)?;
                    needs_full_search = evaluation > alpha;
                }

                if needs_full_search {
                    // [Null Window](https://www.chessprogramming.org/Null_Window) search
                    evaluation = -self.zero_window_search(
                        game,
                        depth - 1, // do not apply search extensions in zws
                        -alpha,
                    )?;

                    if evaluation > alpha && evaluation < beta {
                        // Re-search with full window
                        self.diagnostics.increment_zero_window_search_fail();
                        evaluation = -self.principal_variation_search(
                            game,
                            ply_from_root + 1,
                            depth - 1 + extension,
                            -beta,
                            -alpha,
                            num_extensions + extension,
                        )?;
                    }
                }
            }

            game.undo_action(action, true)?;

            if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
                return Ok(0);
            }

            if evaluation >= beta {
                self.diagnostics.increment_fail_high(is_pv_node);

                self.store_transposition_table(game, depth, beta, EvaluationType::LowerBound, action);

                return Ok(if self.options.features.failing_strategy == FailingStrategy::FailSoft {
                    evaluation // Fail-soft beta-cutoff
                } else {
                    beta // Fail-hard beta-cutoff
                });
            }

            if evaluation > alpha {
                evaluation_bound = EvaluationType::Exact;
                alpha = evaluation; // alpha acts like max in MiniMax
                best_action = action;
            }

            is_pv_node = false;
        }
        // If we are in the first ply then the evaluation_bound has to be exact
        // or iff aspiration windows are used can also be an upper bound
        debug_assert!(
            ply_from_root != 0 || evaluation_bound == EvaluationType::Exact || self.options.features.aspiration_window,
            "[PVSPlayer::principal_variation_search] Assert about ply_from_root: {}, evaluation_bound: {:?}, aspiration_windows: {}",
            ply_from_root,
            evaluation_bound,
            self.options.features.aspiration_window
        );

        // In case of a UpperBound we store a null action, as the true best
        // action is unknown
        self.store_transposition_table(game, depth, alpha, evaluation_bound, best_action);

        // TODO: remove these
        if ply_from_root == 0 {
            self.best_action = Some(best_action);
            self.best_evaluation = Some(alpha);
        }

        Ok(alpha)
    }

    /// Does a Zero/Scout Window Search (ZWS) with the given parameters.
    ///
    /// fail-hard zero window search, returns either `beta-1` or `beta`
    /// only takes the beta parameter because `alpha == beta - 1`
    fn zero_window_search(&mut self, game: &mut Patchwork, depth: usize, beta: i32) -> PlayerResult<i32> {
        // Return if the search has been canceled
        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(0);
        }

        // Return evaluation if the game is over or we reached the maximum depth
        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        // Collect diagnostics
        self.diagnostics.increment_nodes_searched();

        let actions = game.get_valid_actions();
        for action in actions {
            game.do_action(action, true)?;

            let evaluation = -self.zero_window_search(
                game,
                depth - 1, // do not apply search extensions in zws
                1 - beta,
            )?;

            game.undo_action(action, true)?;

            if evaluation >= beta {
                return Ok(if self.options.features.failing_strategy == FailingStrategy::FailSoft {
                    evaluation // Fail-soft beta-cutoff
                } else {
                    beta // Fail-hard beta-cutoff
                });
            }
        }
        Ok(beta - 1) // fail-hard, return alpha
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
    fn get_search_extension(&mut self, game: &Patchwork, num_extensions: usize) -> usize {
        if !self.options.features.search_extensions {
            return 0;
        }

        if num_extensions >= Self::MAX_SEARCH_EXTENSIONS {
            return 0;
        }

        let mut extension = 0;

        // Extend the depth of search for special patch placements
        if matches!(
            game.turn_type,
            TurnType::SpecialPatchPlacement | TurnType::SpecialPhantom
        ) {
            self.diagnostics.increment_special_patch_extensions();
            // TODO: this will double extend with special phantom then special patch placement
            // we could not extend special phantom but that would be wrong as we could have a special phantom and already have depth 0
            // maybe change evaluation to go further
            extension = 1;
        }

        extension
    }

    /// Evaluates the given game state.
    ///
    /// # Possible Improvement
    ///
    /// Implement another function "quiescence search" that searches for a stable evaluation before calling this evaluation function
    /// This would mitigate the horizon effect
    /// The question is if this is even needed in patchwork
    fn evaluation(&mut self, game: &mut Patchwork) -> PlayerResult<i32> {
        self.diagnostics.increment_nodes_searched();
        self.diagnostics.increment_leaf_nodes_searched();

        // Force a turn for phantom moves
        let mut needs_undo_action = false;
        if matches!(game.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom) {
            game.do_action(ActionId::phantom(), true)?;
            needs_undo_action = true;
        }

        // TODO: mate scores

        let color = game.get_current_player() as i32;
        let evaluation = color * self.options.evaluator.evaluate_node(game);

        // self.store_transposition_table(game, 0, evaluation, EvaluationType::Exact, ActionId::null());

        // Reset to phantom action
        if needs_undo_action {
            game.undo_action(ActionId::phantom(), true)?;
            // self.store_transposition_table(game, 0, evaluation, EvaluationType::Exact, ActionId::null());
        }

        Ok(evaluation)
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
        game: &mut Patchwork,
        depth: usize,
        evaluation: i32,
        evaluation_type: EvaluationType,
        action: ActionId,
    ) {
        let Some(ref mut transposition_table) = self.transposition_table else {
            return;
        };

        match self.options.features.transposition_table {
            TranspositionTableFeature::Disabled => unreachable!("[PVSPlayer::store_transposition_table] Transposition table exists but the feature is actually disabled."),
            TranspositionTableFeature::Enabled { .. } => transposition_table.store_evaluation(game, depth, evaluation, evaluation_type, action),
            TranspositionTableFeature::SymmetryEnabled { .. } => transposition_table.store_evaluation_with_symmetries(game, depth, evaluation, evaluation_type, action),
        }
    }

    /// Gets the principal variation action for the given game state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to get the principal variation action for.
    /// * `ply_from_root` - The ply from the root node.
    ///
    /// # Returns
    ///
    /// The principal variation action for the given game state.
    fn get_pv_action(&self, game: &mut Patchwork, ply_from_root: usize) -> Option<ActionId> {
        // FEATURE:PV_TABLE: use pv table here
        if ply_from_root == 0 {
            if let Some(pv_action) = self.best_action {
                return Some(pv_action);
            }
        }

        if let Some(ref transposition_table) = self.transposition_table {
            return transposition_table.probe_pv_move(game);
        }

        None
    }

    /// Writes a single diagnostic to the diagnostics writer.
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - The diagnostic to write.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the writing.
    #[inline]
    fn write_single_diagnostic(&mut self, diagnostic: &str) -> Result<(), std::io::Error> {
        let writer = match self.options.features.diagnostics {
            DiagnosticsFeature::Disabled => return Ok(()),
            DiagnosticsFeature::Enabled { ref mut writer } => writer.as_mut(),
            DiagnosticsFeature::Verbose { ref mut writer } => writer.as_mut(),
        };

        writeln!(writer, "{}", diagnostic)
    }

    /// Writes the diagnostics to the diagnostics writer.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether to write the full diagnostics or only the most important ones.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the writing.
    #[inline]
    #[rustfmt::skip]
    fn write_diagnostics(
        &mut self,
        game: &Patchwork,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        let is_verbose = matches!(self.options.features.diagnostics, crate::DiagnosticsFeature::Verbose { .. });
        let writer = match self.options.features.diagnostics {
            DiagnosticsFeature::Disabled => return Ok(()),
            DiagnosticsFeature::Enabled { ref mut writer } => writer.as_mut(),
            DiagnosticsFeature::Verbose { ref mut writer } => writer.as_mut(),
        };

        // FEATURE:PV_TABLE: use pv table here
        let pv_actions = if let Some(ref mut transposition_table) = self.transposition_table {
           transposition_table.get_pv_line(game, depth).iter()
           .map(|action| match action.save_to_notation() {
               Ok(notation) => notation,
               Err(_) => "######".to_string(),
           })
           .join(" → ")
        } else if let Some(pv_action) = self.best_action {
            match pv_action.save_to_notation() {
                Ok(notation) => format!("{} → ...", notation),
                Err(_) => "###### → ...".to_string(),
            }
        } else {
            "NONE".to_string()
        };

        let best_evaluation = self.best_evaluation.map(|eval| format!("{}", eval)).unwrap_or("NONE".to_string());
        let best_action = self.best_action.as_ref().map(|action| match action.save_to_notation() {
            Ok(notation) => notation,
            Err(_) => "######".to_string(),
        }).unwrap_or("NONE".to_string());

        let mut features = vec![];
        if self.options.features.aspiration_window {
            features.push("AW");
        }
        if matches!(self.options.features.transposition_table, TranspositionTableFeature::Enabled { .. }) {
            features.push("TT");
        } else if matches!(self.options.features.transposition_table, TranspositionTableFeature::SymmetryEnabled { .. }) {
            features.push("TT(S)");
        }
        if self.options.features.late_move_reductions {
            features.push("LMR");
        }
        if self.options.features.late_move_pruning {
            features.push("LMP");
        }
        if self.options.features.search_extensions {
            features.push("SE");
        }
        let features = features.join(", ");

        // [Branching Factor](https://www.chessprogramming.org/Branching_Factor)
        let average_branching_factor = (self.diagnostics.leaf_nodes_searched as f64).powf(1.0 / depth as f64);
        let effective_branching_factor = self.diagnostics.nodes_searched as f64 / self.diagnostics.nodes_searched_previous_iteration as f64;
        let mean_branching_factor = self.diagnostics.nodes_searched as f64 / (self.diagnostics.nodes_searched - self.diagnostics.leaf_nodes_searched) as f64;

        writeln!(writer, "───────────── Principal Variation Search Player ─────────────")?;
        writeln!(writer, "Features:            [{}]", features)?;
        writeln!(writer, "Depth:               {:?}", depth)?;
        writeln!(writer, "Time:                {:?}", std::time::Instant::now().duration_since(self.diagnostics.start_time))?;
        writeln!(writer, "Nodes searched:      {:?}", self.diagnostics.nodes_searched)?;
        writeln!(writer, "Branching factor:    {:.2} AVG / {:.2} EFF / MEAN {:.2}", average_branching_factor, effective_branching_factor, mean_branching_factor)?;
        writeln!(writer, "Best Action:         {} ({} pts)", best_action, best_evaluation)?;
        writeln!(writer, "Move Ordering:       {:?} ({} high pv / {} high)", (self.diagnostics.fail_high_first as f64) / (self.diagnostics.fail_high as f64), self.diagnostics.fail_high_first, self.diagnostics.fail_high)?;
        writeln!(writer, "Aspiration window:   {:?} low / {:?} high", self.diagnostics.aspiration_window_fail_low, self.diagnostics.aspiration_window_fail_high)?;
        writeln!(writer, "Zero window search:  {:?} fails ({:.2}%)", self.diagnostics.zero_window_search_fail, self.diagnostics.zero_window_search_fail_rate() * 100.0)?;
        writeln!(writer, "Special patch ext.:  {:?} ({})", self.diagnostics.special_patch_extensions, if self.options.features.search_extensions { "enabled" } else { "disabled" })?;
        writeln!(writer, "LMR / LMP:           {:?} / {:?}", self.diagnostics.late_move_reductions, self.diagnostics.late_move_pruning)?;
        writeln!(writer, "Principal Variation: {}", pv_actions)?;
        if let Some(ref mut transposition_table) = self.transposition_table {
            transposition_table.diagnostics.write_diagnostics(writer)?;
            if is_verbose {
                transposition_table.diagnostics.write_transposition_table(writer, transposition_table, Some(100))?;
            }
        }
        writeln!(writer, "─────────────────────────────────────────────────────────────")?;

        Ok(())
    }
}
