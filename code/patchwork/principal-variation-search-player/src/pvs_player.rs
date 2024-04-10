use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc},
};

use action_orderer::{ActionList, ActionOrderer, TableActionOrderer};
use evaluator::StaticEvaluator;
use itertools::Itertools;

use patchwork_core::{
    evaluator_constants, ActionId, Evaluator, Logging, Notation, Patchwork, Player, PlayerResult, TurnType,
};
use transposition_table::{EvaluationType, TranspositionTable};

use crate::{pvs_options::FailingStrategy, PVSOptions, SearchStatistics, TranspositionTableFeature};

pub struct SearchRecorderNode {
    pub state: Patchwork,
    pub value: Option<i32>,
    pub alpha: Option<i32>,
    pub beta: Option<i32>,
    pub description: Option<String>,
    pub children: Vec<Rc<RefCell<SearchRecorderNode>>>,
}

pub struct SearchRecorder {
    pub index: usize,
    pub root: Option<Rc<RefCell<SearchRecorderNode>>>,
    pub current_nodes: Vec<Rc<RefCell<SearchRecorderNode>>>,
}

impl SearchRecorder {
    pub fn new() -> Self {
        Self {
            index: 1,
            root: None,
            current_nodes: vec![],
        }
    }

    pub fn push_state(&mut self, state: Patchwork) {
        return;

        let node = Rc::new(RefCell::new(SearchRecorderNode {
            state,
            value: None,
            alpha: None,
            beta: None,
            description: None,
            children: vec![],
        }));

        if self.current_nodes.is_empty() {
            self.root = Some(Rc::clone(&node));
        } else {
            let last_node = self.current_nodes.last().unwrap();
            RefCell::borrow_mut(last_node).children.push(Rc::clone(&node));
        }
        self.current_nodes.push(node);
    }

    pub fn pop_state_with_value(&mut self, value: i32, alpha: i32, beta: i32, description: &str) {
        return;

        let last_node = self.current_nodes.pop().unwrap();
        RefCell::borrow_mut(&last_node).value = Some(value);
        RefCell::borrow_mut(&last_node).alpha = Some(alpha);
        RefCell::borrow_mut(&last_node).beta = Some(beta);
        RefCell::borrow_mut(&last_node).description = Some(description.to_string());
    }

    pub fn print_to_file(&mut self) {
        return;

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("search_tree_{:04}.txt", self.index))
            .unwrap();

        self.index += 1;

        let mut writer = std::io::BufWriter::new(file);
        self.print_to_file_recursive(&self.root, 0, &mut writer, "");
    }

    #[allow(clippy::only_used_in_recursion)]
    fn print_to_file_recursive(
        &self,
        node: &Option<Rc<RefCell<SearchRecorderNode>>>,
        depth: usize,
        writer: &mut std::io::BufWriter<std::fs::File>,
        padding: &str,
    ) {
        if let Some(node) = node {
            let node = RefCell::borrow(node);

            writeln!(
                writer,
                "{padding}Value: {:4?}, Text: {:?}, Alpha: {:?}, Beta: {:?}, Depth: {depth:?}, Player: {:?}", // , State: {:?}",
                node.value.unwrap(),
                node.description.as_ref().unwrap(),
                node.alpha.unwrap(),
                node.beta.unwrap(),
                node.state.get_current_player(),
                // node.state
            )
            .unwrap();

            for i in 0..node.children.len() {
                let child = &node.children[i];

                if i < node.children.len() - 1 {
                    let mut hasher = DefaultHasher::new();
                    RefCell::borrow(child).state.hash(&mut hasher);
                    let hash1 = hasher.finish();

                    let next_child = &node.children[i + 1];

                    let mut hasher = DefaultHasher::new();
                    RefCell::borrow(next_child).state.hash(&mut hasher);
                    let hash2 = hasher.finish();

                    if hash1 == hash2 {
                        self.print_to_file_recursive(
                            &Some(Rc::clone(child)),
                            depth + 1,
                            writer,
                            format!("{padding}    ↓   ").as_str(),
                        );
                        continue;
                    }
                }

                self.print_to_file_recursive(
                    &Some(Rc::clone(child)),
                    depth + 1,
                    writer,
                    format!("{padding}    ").as_str(),
                );
            }
        }
    }
}

impl Default for SearchRecorder {
    fn default() -> Self {
        Self::new()
    }
}

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
///     - With PV-Action via Transposition Table
pub struct PVSPlayer<Orderer: ActionOrderer = TableActionOrderer, Eval: Evaluator = StaticEvaluator> {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// search statistics
    pub statistics: SearchStatistics,
    /// The evaluator to evaluate the game state.
    pub evaluator: Eval,
    /// The action sorter to sort the actions.
    pub action_orderer: Orderer,
    /// The transposition table.
    transposition_table: Option<TranspositionTable>,
    /// The best action found so far.
    ///
    /// The best action for the root is kept in a separate variable so that
    /// it can be returned even if the transposition table is disabled or the
    /// pv action is overwritten by the transposition table.
    best_action: Option<ActionId>,
    /// The best evaluation found so far.
    best_evaluation: Option<i32>,
    /// Whether the search has been canceled.
    search_canceled: Arc<AtomicBool>,
    search_recorder: SearchRecorder,
}

impl<Orderer: ActionOrderer + Default, Eval: Evaluator + Default> PVSPlayer<Orderer, Eval> {
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

        Self {
            name: name.into(),
            options,
            statistics: SearchStatistics::default(),
            evaluator: Default::default(),
            action_orderer: Default::default(),
            transposition_table,
            best_action: None,
            best_evaluation: None,
            search_canceled: Arc::new(AtomicBool::new(false)),
            search_recorder: SearchRecorder::default(),
        }
    }
}

impl<Orderer: ActionOrderer + Default, Eval: Evaluator + Default> Default for PVSPlayer<Orderer, Eval> {
    fn default() -> Self {
        Self::new("Principal Variation Search Player".to_string(), None)
    }
}

impl<Orderer: ActionOrderer, Eval: Evaluator> Player for PVSPlayer<Orderer, Eval> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        std::thread::scope(|s| {
            let search_canceled = Arc::clone(&self.search_canceled);
            let time_limit = self.options.time_limit;

            // reset the parameters for the search
            self.search_canceled.store(false, std::sync::atomic::Ordering::SeqCst);
            self.best_evaluation = None;
            self.best_action = None;

            // reset all statistics
            self.statistics.reset();
            if let Some(ref mut transposition_table) = self.transposition_table {
                transposition_table.reset_statistics();
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

            // force stop the timer thread if the search cancelled itself
            self.search_canceled.store(true, std::sync::atomic::Ordering::Release);

            result
        })
    }
}

impl<Orderer: ActionOrderer, Eval: Evaluator> PVSPlayer<Orderer, Eval> {
    /// The amount of actions where the following search is for sure not reduced
    /// by LMR. After these amount of actions the LMR is applied.
    pub const LMR_AMOUNT_FULL_DEPTH_ACTIONS: usize = 3;
    /// The maximum depth at which LMR will be applied.
    pub const LMR_DEPTH_LIMIT: usize = 1;

    /// The amount of actions that are for sure not pruned by LMP. After these
    /// amount of actions the LMP is applied.
    pub const LMP_AMOUNT_NON_PRUNED_ACTIONS: usize = 5;
    /// The maximum depth at which LMP will be applied.
    pub const LMP_DEPTH_LIMIT: usize = 1; // BUG: PVS PLAYER plays worse if this is enabled. Why?
    /// The maximum amount of search extensions that can be applied.
    pub const MAX_SEARCH_EXTENSIONS: usize = 16; // Can probably never be reached

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
    pub const MINIMUM_DELTA: i32 = 3;

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

        self.statistics.reset_iterative_deepening_iteration();

        // [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening) loop
        while depth < Self::MAX_DEPTH {
            let best_action = self.best_action;
            let best_evaluation = self.best_evaluation;

            let evaluation = self.principal_variation_search(game, 0, depth, alpha, beta, 0)?;
            self.search_recorder.print_to_file();

            if self.search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                break;
            }

            let _ = self.write_statistics(game, depth); // ignore errors

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
                    "[PVSPlayer::search] Assert evaluation({evaluation}) <= alpha({alpha}) should imply aspiration window but was not."
                );

                beta = (alpha + beta) / 2; // adjust beta towards alpha
                alpha = (evaluation - delta).min(Self::MIN_ALPHA_BOUND);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY); // use same exponential backoff as in [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L429C17-L429C36)
                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                self.statistics.increment_aspiration_window_fail_low();
                continue;
            } else if evaluation >= beta {
                // Evaluation is above aspiration window [Fail-High](https://www.chessprogramming.org/Fail-High#Root_with_Aspiration)
                // The best found evaluation is greater or equal to the upper bound (beta), so we need to research at the same depth

                debug_assert!(
                    self.options.features.aspiration_window, // this should never happen if aspiration windows are off
                    "[PVSPlayer::search] Assert evaluation({evaluation}) >= beta({beta}) should imply aspiration window but was not."
                );

                beta = (evaluation + delta).min(Self::MAX_BETA_BOUND);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY);
                self.best_action = best_action;
                self.best_evaluation = best_evaluation;
                self.statistics.increment_aspiration_window_fail_high();
                continue;
            }

            // let _ = self.write_statistics(game, depth); // ignore errors

            if self.best_evaluation == Some(evaluator_constants::POSITIVE_INFINITY) {
                // We found a winning game, so we can stop searching
                // TODO: it would be possible here to get the plys to win by offset
                break;
            }

            if self.options.features.aspiration_window {
                // Evaluation is within the aspiration window,
                // so we can move on to the next depth with a window set around the eval
                if let Some(eval) = self.best_evaluation {
                    delta = (Self::MINIMUM_DELTA + eval.abs() / 10).min(evaluator_constants::POSITIVE_INFINITY);
                // TODO: use avg of root node scores like in Stockfish `delta = Value(9) + int(avg) * avg / 14847;`
                } else {
                    delta = Self::MINIMUM_DELTA;
                }

                alpha = evaluation - delta;
                beta = evaluation + delta;
            }

            depth += 1;
            self.statistics.reset_iterative_deepening_iteration();
        }

        let best_action = self.best_action.take().unwrap_or_else(|| {
            let _ = self.write_log("No best action found. Returning random valid action. This only happends when no full search iteration could be done."); // ignore errors
            game.get_random_action()
        });

        let _ = self.write_log(format!("Best action: {best_action:?}").as_str()); // ignore errors
        let _ = self.write_statistics(game, depth); // ignore errors

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
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::useless_let_if_seq)]
    fn principal_variation_search(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        alpha: i32,
        beta: i32,
        num_extensions: usize,
    ) -> PlayerResult<i32> {
        self.search_recorder.push_state(game.clone());

        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            self.search_recorder
                .pop_state_with_value(0, alpha, beta, "Search Cancelled");
            return Ok(0);
        }

        // TODO: lookup mates (inspired by Searcher.cs)

        if let Some(ref mut transposition_table) = self.transposition_table {
            if let Some((table_action, table_evaluation)) =
                transposition_table.probe_hash_entry(game, alpha, beta, depth)
            {
                if ply_from_root == 0 {
                    self.best_action = Some(table_action);
                    self.best_evaluation = Some(table_evaluation);
                }
                self.search_recorder
                    .pop_state_with_value(table_evaluation, alpha, beta, "Transposition Table Hit");
                return Ok(table_evaluation);
            }
        }

        if depth == 0 || game.is_terminated() {
            let evaluation = self.evaluation(game)?;
            self.search_recorder
                .pop_state_with_value(evaluation, alpha, beta, "Evaluation");
            return Ok(evaluation);
        }

        self.statistics.increment_nodes_searched();

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

        let mut scores = vec![0.0; actions.len()];
        let mut action_list = ActionList::new(&mut actions, &mut scores);
        let mut lmp_flags = if self.options.features.late_move_pruning {
            LMPFlags::initialize_from(&action_list)
        } else {
            LMPFlags::fake()
        };

        self.action_orderer
            .score_actions(&mut action_list, pv_action, ply_from_root);

        let mut is_pv_node = true;
        let mut best_action = ActionId::null();
        let mut alpha = alpha;
        let mut evaluation_bound = EvaluationType::UpperBound;
        let should_late_move_prune = ply_from_root >= Self::LMP_DEPTH_LIMIT && self.options.features.late_move_pruning;

        for i in 0..action_list.len() {
            let action = if should_late_move_prune && i >= Self::LMP_AMOUNT_NON_PRUNED_ACTIONS {
                // [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning)
                // Remove late moves in move ordering, only prune after trying at least one move for each possible patch
                let Some(action) = lmp_flags.get_next_missing() else {
                    self.statistics.increment_late_move_pruning();
                    break;
                };
                action
            } else {
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
                self.action_orderer.pick_action(&mut action_list, i)
            };

            lmp_flags.set_action_type_done(action);

            // Save previous state characteristics that are needed later
            let previous_special_tile_condition_reached = game.is_special_tile_condition_reached();

            game.do_action(action, true)?;

            // Extend the depth of the search in certain interesting cases (special patch placement)
            let extension = self.get_search_extension(game, num_extensions, previous_special_tile_condition_reached);

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
                )?;
            } else {
                self.statistics.increment_zero_window_search();
                // Apply [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions) if we're not in the early moves (and this is not a PV node)
                // Reduce the depth of the search for later actions as these are less likely to be good (assuming the action ordering is good)
                // Code adapted from https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html
                // Search Extensions are not applied to the reduced depth search
                let mut needs_full_search = true;
                if extension == 0 && i >= Self::LMR_AMOUNT_FULL_DEPTH_ACTIONS && ply_from_root >= Self::LMR_DEPTH_LIMIT
                {
                    self.statistics.increment_late_move_reductions();

                    let lmr_depth_reduction = if depth >= 6 { depth / 3 } else { 1 };
                    // Search this move with reduced depth
                    evaluation = -self.zero_window_search(
                        game,
                        ply_from_root + 1,
                        (depth - 1).saturating_sub(lmr_depth_reduction),
                        -alpha,
                    )?;
                    needs_full_search = evaluation > alpha;
                }

                if needs_full_search {
                    // [Null Window](https://www.chessprogramming.org/Null_Window) search
                    evaluation = -self.zero_window_search(
                        game,
                        ply_from_root + 1,
                        depth - 1, // do not apply search extensions in zws
                        -alpha,
                    )?;

                    if evaluation > alpha && evaluation < beta {
                        // Re-search with full window
                        self.statistics.increment_zero_window_search_fail();
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
                self.search_recorder
                    .pop_state_with_value(0, alpha, beta, "Search Cancelled");
                return Ok(0);
            }

            if evaluation >= beta {
                self.statistics.increment_fail_high(is_pv_node);

                self.store_transposition_table(game, depth, beta, EvaluationType::LowerBound, action);

                return Ok(if self.options.features.failing_strategy == FailingStrategy::FailSoft {
                    self.search_recorder
                        .pop_state_with_value(evaluation, alpha, beta, "FailSoft Evaluation");
                    evaluation // Fail-soft beta-cutoff
                } else {
                    self.search_recorder
                        .pop_state_with_value(beta, alpha, beta, "Beta Cutoff");
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
            "[PVSPlayer::principal_variation_search] Assert about ply_from_root: {}, evaluation_bound: {:?}, aspiration_windows: {}, alpha: {:?}.",
            ply_from_root,
            evaluation_bound,
            self.options.features.aspiration_window,
            alpha
        );

        // In case of a UpperBound we store a null action, as the true best
        // action is unknown
        self.store_transposition_table(game, depth, alpha, evaluation_bound, best_action);

        if ply_from_root == 0 {
            self.best_action = Some(best_action);
            self.best_evaluation = Some(alpha);
        }

        self.search_recorder
            .pop_state_with_value(alpha, alpha, beta, "Normal Alpha");
        Ok(alpha)
    }

    /// Does a Zero/Scout Window Search (ZWS) with the given parameters.
    ///
    /// fail-hard zero window search, returns either `beta-1` or `beta`
    /// only takes the beta parameter because `alpha == beta - 1`
    fn zero_window_search(
        &mut self,
        game: &mut Patchwork,
        ply_from_root: usize,
        depth: usize,
        beta: i32,
    ) -> PlayerResult<i32> {
        let alpha = beta - 1;

        self.search_recorder.push_state(game.clone());

        // Return if the search has been canceled
        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            self.search_recorder
                .pop_state_with_value(0, alpha, beta, "Search Cancelled (ZWS)");
            return Ok(0);
        }

        if let Some(ref mut transposition_table) = self.transposition_table {
            if let Some((_table_action, table_evaluation)) =
                transposition_table.probe_hash_entry(game, alpha, beta, depth)
            {
                self.search_recorder.pop_state_with_value(
                    table_evaluation,
                    alpha,
                    beta,
                    "Transposition Table Hit (ZWS)",
                );
                return Ok(table_evaluation);
            }
        }

        // Return evaluation if the game is over or we reached the maximum depth
        if depth == 0 || game.is_terminated() {
            let evaluation = self.evaluation(game)?;
            self.search_recorder
                .pop_state_with_value(evaluation, alpha, beta, "Evaluation (ZWS)");
            return Ok(evaluation);
        }

        // Collect statistics
        self.statistics.increment_nodes_searched();

        let mut actions = game.get_valid_actions();

        let pv_action = self.get_pv_action(game, ply_from_root);

        let mut scores = vec![0.0; actions.len()];
        let mut action_list = ActionList::new(&mut actions, &mut scores);
        let mut lmp_flags = if self.options.features.late_move_pruning {
            LMPFlags::initialize_from(&action_list)
        } else {
            LMPFlags::fake()
        };

        self.action_orderer
            .score_actions(&mut action_list, pv_action, ply_from_root);

        let mut is_pv_node = true;
        let should_late_move_prune = ply_from_root >= Self::LMP_DEPTH_LIMIT && self.options.features.late_move_pruning;

        for i in 0..action_list.len() {
            let action = if should_late_move_prune && i >= Self::LMP_AMOUNT_NON_PRUNED_ACTIONS {
                let Some(action) = lmp_flags.get_next_missing() else {
                    self.statistics.increment_late_move_pruning();
                    break;
                };
                action
            } else {
                self.action_orderer.pick_action(&mut action_list, i)
            };

            lmp_flags.set_action_type_done(action);

            game.do_action(action, true)?;

            let evaluation = -self.zero_window_search(
                game,
                ply_from_root + 1,
                depth - 1, // do not apply search extensions in zws
                1 - beta,
            )?;

            game.undo_action(action, true)?;

            if evaluation >= beta {
                self.statistics.increment_fail_high(is_pv_node);

                return Ok(if self.options.features.failing_strategy == FailingStrategy::FailSoft {
                    self.search_recorder
                        .pop_state_with_value(evaluation, alpha, beta, "FailSoft Evaluation (ZWS)");
                    evaluation // Fail-soft beta-cutoff
                } else {
                    self.search_recorder
                        .pop_state_with_value(beta, alpha, beta, "Beta Cutoff (ZWS)");
                    beta // Fail-hard beta-cutoff
                });
            }

            is_pv_node = false;
        }
        self.search_recorder
            .pop_state_with_value(alpha, alpha, beta, "Normal Alpha/Beta (ZWS)");
        Ok(alpha) // fail-hard, return alpha
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
    fn get_search_extension(
        &mut self,
        game: &Patchwork,
        num_extensions: usize,
        previous_special_tile_condition_reached: bool,
    ) -> usize {
        if !self.options.features.search_extensions {
            return 0;
        }

        if num_extensions >= Self::MAX_SEARCH_EXTENSIONS {
            return 0;
        }

        let mut extension = 0;

        // Extend the depth of search for special patch placements
        if matches!(game.turn_type, TurnType::SpecialPhantom) {
            self.statistics.increment_special_patch_extensions();
            extension += 1;
        }

        // Extend the depth of search if the 7x7 special tile was given
        if !previous_special_tile_condition_reached && game.is_special_tile_condition_reached() {
            self.statistics.increment_special_tile_extensions();
            extension += 1;
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
    #[allow(clippy::useless_let_if_seq)]
    fn evaluation(&mut self, game: &mut Patchwork) -> PlayerResult<i32> {
        self.statistics.increment_nodes_searched();
        self.statistics.increment_leaf_nodes_searched();

        let color = if game.is_player_1() { 1 } else { -1 };

        // Force a turn for phantom moves
        let mut needs_undo_action = false;
        if matches!(game.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom) {
            game.do_action(ActionId::phantom(), true)?;
            needs_undo_action = true;
        }

        // TODO: mate scores
        // if (moves.Length == 0)
        // {
        //     if (moveGenerator.InCheck())
        //     {
        //         int mateScore = immediateMateScore - plyFromRoot;
        //         return -mateScore;
        //     }
        //     else
        //     {
        //         return 0;
        //     }
        // }

        let evaluation = color * self.evaluator.evaluate_node(game);

        // self.store_transposition_table(game, 0, evaluation, EvaluationType::Exact, ActionId::null());

        // Reset to phantom action
        if needs_undo_action {
            game.undo_action(ActionId::phantom(), true)?;
            // self.store_transposition_table(game, 0, evaluation, EvaluationType::Exact, ActionId::null());
        }

        debug_assert!(
            (evaluator_constants::NEGATIVE_INFINITY..=evaluator_constants::POSITIVE_INFINITY).contains(&evaluation),
            "[PVSPlayer::evaluation] Assert evaluation({}) is not in range of [{}, {}].",
            evaluation,
            evaluator_constants::NEGATIVE_INFINITY,
            evaluator_constants::POSITIVE_INFINITY
        );

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
        game: &Patchwork,
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
        if let Some(ref transposition_table) = self.transposition_table {
            return transposition_table.probe_pv_move(game);
        }

        None
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
        if let Some(ref transposition_table) = self.transposition_table {
            return transposition_table
                .get_pv_line(game, depth)
                .iter()
                .map(|action| {
                    action
                        .save_to_notation()
                        .map_or_else(|_| "######".to_string(), |notation| notation)
                })
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

    /// Writes the statistics to the logging writer.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether to write the full statistics or only the most important ones.
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

        let mut debug_writer = None;
        let writer = match self.options.logging {
            Logging::Disabled => return Ok(()),
            Logging::Enabled { progress_writer: ref mut writer } => writer.as_mut(),
            Logging::Verbose { progress_writer: ref mut writer, debug_writer: ref mut d_writer } => {
                debug_writer = Some(d_writer);
                writer.as_mut()
            },
            Logging::VerboseOnly { ref mut debug_writer } =>{
                if let Some(ref mut transposition_table) = self.transposition_table {
                    transposition_table.statistics.write_transposition_table(debug_writer, transposition_table, None)?;
                }
                return Ok(())
            }
        };

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
        writeln!(writer, "Move Ordering:       {:?} ({} high pv / {} high)", (self.statistics.fail_high_first as f64) / (self.statistics.fail_high as f64), self.statistics.fail_high_first, self.statistics.fail_high)?;
        writeln!(writer, "Aspiration window:   {:?} low / {:?} high", self.statistics.aspiration_window_fail_low, self.statistics.aspiration_window_fail_high)?;
        writeln!(writer, "Zero window search:  {:?} fails ({:.2}%)", self.statistics.zero_window_search_fail, self.statistics.zero_window_search_fail_rate() * 100.0)?;
        writeln!(writer, "Search Extensions:   {:?} SP, {:?} ST ({})", self.statistics.special_patch_extensions, self.statistics.special_tile_extensions, if self.options.features.search_extensions { "enabled" } else { "disabled" })?;
        writeln!(writer, "LMR / LMP:           {:?} / {:?}", self.statistics.late_move_reductions, self.statistics.late_move_pruning)?;
        writeln!(writer, "Principal Variation: {pv_actions}")?;
        if let Some(ref mut transposition_table) = self.transposition_table {
            transposition_table.statistics.write_statistics(writer)?;
            if let Some(debug_writer) = debug_writer {
                transposition_table.statistics.write_transposition_table(debug_writer, transposition_table, None)?;
            }
        }
        writeln!(writer, "─────────────────────────────────────────────────────────────")?;

        Ok(())
    }
}

/// Different Flags and Actions for Late Move Pruning that are used to ensure
/// that every action type is tried at least once before pruning all other
/// actions.
struct LMPFlags {
    walking: Option<ActionId>,
    patch1: Option<ActionId>,
    patch2: Option<ActionId>,
    patch3: Option<ActionId>,
}

impl LMPFlags {
    /// Creates a fake LMP flags struct in constant time.
    ///
    /// # Returns
    ///
    /// The fake LMP flags struct.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub const fn fake() -> Self {
        Self {
            walking: None,
            patch1: None,
            patch2: None,
            patch3: None,
        }
    }

    /// Initializes the LMP flags from the given action list.
    ///
    /// # Arguments
    ///
    /// * `action_list` - The action list to initialize the flags from.
    ///
    /// # Returns
    ///
    /// The initialized LMP flags.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the length of the action list.
    #[allow(clippy::collapsible_if)]
    #[inline]
    pub fn initialize_from(action_list: &ActionList<'_>) -> Self {
        let mut flags = Self {
            walking: None,
            patch1: None,
            patch2: None,
            patch3: None,
        };
        let mut highest_patch_1_score = f64::NEG_INFINITY;
        let mut highest_patch_2_score = f64::NEG_INFINITY;
        let mut highest_patch_3_score = f64::NEG_INFINITY;

        // TODO: we do not want to go thought the whole list
        for i in 0..action_list.len() {
            let action = action_list.get_action(i);
            let score = action_list.get_score(i);

            if action.is_walking() {
                flags.walking = Some(action);
            }

            if action.is_first_patch_taken() || action.is_special_patch_placement() {
                if score > highest_patch_1_score {
                    highest_patch_1_score = score;
                    flags.patch1 = Some(action);
                }
            } else if action.is_second_patch_taken() {
                if score > highest_patch_2_score {
                    highest_patch_2_score = score;
                    flags.patch2 = Some(action);
                }
            } else if action.is_third_patch_taken() {
                if score > highest_patch_3_score {
                    highest_patch_3_score = score;
                    flags.patch3 = Some(action);
                }
            }
        }

        flags
    }

    /// Gets an action as parameter and sets the flag for that action type to
    /// `None` so that the action type is done and does not prevent lmp.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to set the flag for.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub fn set_action_type_done(&mut self, action: ActionId) {
        if action.is_walking() {
            self.walking = None;
        } else if action.is_first_patch_taken() || action.is_special_patch_placement() {
            self.patch1 = None;
        } else if action.is_second_patch_taken() {
            self.patch2 = None;
        } else if action.is_third_patch_taken() {
            self.patch3 = None;
        }
    }

    /// Gets the next action for the action type that is not done yet.
    ///
    /// # Returns
    ///
    /// The next action for the action type that is not done yet or `None` if
    /// all action types are done.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub fn get_next_missing(&mut self) -> Option<ActionId> {
        if let Some(action) = self.walking.take() {
            return Some(action);
        }

        if let Some(action) = self.patch1.take() {
            return Some(action);
        }

        if let Some(action) = self.patch2.take() {
            return Some(action);
        }

        if let Some(action) = self.patch3.take() {
            return Some(action);
        }

        None
    }
}
