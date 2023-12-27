use std::sync::{atomic::AtomicBool, Arc};

use itertools::Itertools;
use patchwork_core::{evaluator_constants, Action, Notation, Patchwork, Player, PlayerResult, TurnType};

use crate::{
    search_diagnostics::SearchDiagnostics,
    transposition_table::{Entry, EvaluationType},
    PVSOptions, TranspositionTable,
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
/// - [Search Extension](https://www.chessprogramming.org/Extensions) - Win-seeking search extensions for special patch placements
/// - [Move Ordering](https://www.chessprogramming.org/Move_Ordering)
///     - PV Actions sorted first
///
/// TODO:
/// # Features that still need to be implemented
///
///     - Something like MMV-LVA but for patchwork (e.g. ending score)
///     - Actions that are inside the transposition table
///     - Killer Moves (TODO)
///     - Thread escape move check
///     - History Heuristic
/// - [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#:~:text=Late%20Move%20Pruning%20%E2%80%8B,by%20the%20move%20ordering%20algorithm.) Remove late moves in move ordering
/// - [Internal Iterative Deepening (IID)](https://www.chessprogramming.org/Internal_Iterative_Deepening)
/// - [Null Move Pruning](https://www.chessprogramming.org/Null_Move_Pruning) if it brings something
/// - [Lazy SMP](https://www.chessprogramming.org/Lazy_SMP) - spawn multiple threads in iterative deepening, share transposition table, take whichever finishes first
///
/// # Features that could maybe be implemented (look into it what it is)
///    -   (Reverse) Futility Pruning
///    -   Delta Pruning
pub struct PVSPlayer {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// search diagnostics
    pub diagnostics: SearchDiagnostics,
    /// The transposition table.
    transposition_table: TranspositionTable,
    /// The best action found so far.
    best_action: Option<Action>,
    /// The best evaluation found so far.
    best_evaluation: Option<isize>,
    /// Whether the search has been canceled.
    search_canceled: Arc<AtomicBool>,
}

impl PVSPlayer {
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<PVSOptions>) -> Self {
        let options = options.unwrap_or_default();
        let transposition_table_size = options.transposition_table_size;
        PVSPlayer {
            name: name.into(),
            options,
            diagnostics: Default::default(),
            transposition_table: TranspositionTable::new(transposition_table_size), // TODO: size of transposition table as parameter instead of const generics
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

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<Action> {
        std::thread::scope(|s| {
            let search_canceled = Arc::clone(&self.search_canceled);
            let time_limit = self.options.time_limit;

            // reset the parameters for the search
            self.search_canceled.store(false, std::sync::atomic::Ordering::SeqCst);
            self.best_evaluation = None;
            self.best_action = None;

            // reset all diagnostics
            self.diagnostics = Default::default();
            self.transposition_table.reset_diagnostics();
            self.transposition_table.increment_age();

            // thread to stop search after time limit
            s.spawn(move || {
                let start_time = std::time::Instant::now();
                while start_time.elapsed() < time_limit && !search_canceled.load(std::sync::atomic::Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
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
    pub const LMR_REDUCED_BY_DEPTH: usize = 1;
    pub const LMR_FULL_DEPTH_ACTIONS: usize = 4;
    pub const LMR_REDUCTION_LIMIT: usize = 3;
    pub const MAX_SEARCH_EXTENSIONS: usize = 16; // can never be reached as we only have a search extension for special patches that cannot activate another special patch placement

    /// The maximum depth to search.
    /// This is an upper bound that can never be reached as no game can go on
    /// longer than 54*2 turns with phantom moves 108*2=216
    pub const MAX_DEPTH: usize = 256;

    // TODO: better estimations for delta, alpha and beta
    /// Starting value for alpha (lower bound)
    pub const STARTING_ALPHA: isize = -5;
    /// Starting value for beta (upper bound)
    pub const STARTING_BETA: isize = 5;
    /// Minimum delta for aspiration windows
    pub const MINIMUM_DELTA: isize = 1;

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
    fn search(&mut self, game: &mut Patchwork) -> PlayerResult<Action> {
        let mut delta = Self::MINIMUM_DELTA;
        let mut alpha = Self::STARTING_ALPHA;
        let mut beta = Self::STARTING_BETA;
        let mut depth = 1;

        // [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening) loop
        while depth < Self::MAX_DEPTH {
            self.diagnostics.reset_iterative_deepening_iteration();

            let evaluation = self.principal_variation_search(game, 0, depth, alpha, beta, 0)?;

            if self.search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            // [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows) with exponential backoff
            if evaluation <= alpha {
                // Evaluation is below aspiration window [Fail-Low](https://www.chessprogramming.org/Fail-Low#Root_with_Aspiration)
                // The best found evaluation is less than or equal to the lower bound (alpha), so we need to research at the same depth

                beta = (alpha + beta) / 2; // adjust beta towards alpha
                alpha = (evaluation - delta).min(evaluator_constants::NEGATIVE_INFINITY);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY); // use same exponential backoff as in [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L429C17-L429C36)
                self.diagnostics.increment_aspiration_window_fail_low();
                continue;
            } else if evaluation >= beta {
                // Evaluation is above aspiration window [Fail-High](https://www.chessprogramming.org/Fail-High#Root_with_Aspiration)
                // The best found evaluation is greater or equal to the upper bound (beta), so we need to research at the same depth

                beta = (evaluation + delta).min(evaluator_constants::POSITIVE_INFINITY);
                delta = (delta + delta / 3).min(evaluator_constants::POSITIVE_INFINITY);
                self.diagnostics.increment_aspiration_window_fail_high();
                continue;
            }

            let _ = self.write_diagnostics(game, depth, true); // ignore errors

            if self.best_evaluation.is_some() && self.best_evaluation.unwrap() == evaluator_constants::POSITIVE_INFINITY
            {
                // We found a winning game or only have one option available, so we can stop searching
                break;
            }

            // Evaluation is within the aspiration window,
            // so we can move on to the next depth with a window set around the eval

            delta = Self::MINIMUM_DELTA; // TODO: use avg of root node scores like in Stockfish `delta = Value(9) + int(avg) * avg / 14847;`
            alpha = evaluation - delta;
            beta = evaluation + delta;
            depth += 1;
        }

        let best_action = self.best_action.take().unwrap_or_else(|| {
            let _ = self.write_single_diagnostic("No best action found. Returning random valid action. This only happends when no full search iteration could be done."); // ignore errors
            game.get_random_action()
        });

        let _ = self.write_single_diagnostic(format!("Best action: {:?}", best_action).as_str()); // ignore errors
        let _ = self.write_diagnostics(game, depth, true); // ignore errors

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
        alpha: isize,
        beta: isize,
        num_extensions: usize,
    ) -> PlayerResult<isize> {
        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(0);
        }

        // TODO: lookup mates (inspired by Searcher.cs)

        if let Some((table_action, table_evaluation)) =
            self.transposition_table.probe_hash_entry(game, alpha, beta, depth)
        {
            if ply_from_root == 0 {
                // TODO: split phantom and null moves
                self.best_action = Some(table_action);
                self.best_evaluation = Some(table_evaluation);
            }
            return Ok(table_evaluation);
        }

        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        self.diagnostics.increment_nodes_searched();

        let mut actions = game.get_valid_actions();

        // shortcut for only one available action
        if actions.len() == 1 && ply_from_root == 0 {
            self.search_canceled.store(true, std::sync::atomic::Ordering::Release);
            self.best_action = Some(actions[0].clone());
            self.best_evaluation = Some(evaluator_constants::POSITIVE_INFINITY);
            return Ok(evaluator_constants::POSITIVE_INFINITY);
        }

        // TODO: is this useful for patchwork?
        // NULL MOVE PRUNING
        //       // Null move search:
        //       if(ok_to_do_null_move_at_this_node()) {
        //         make_null_move();
        //         value = -search(-beta, -beta, -(beta-1), depth-4);
        //         unmake_null_move();
        //         if(value >= beta) return value;
        //       }

        let pv_action = if ply_from_root == 0 {
            self.best_action
                .clone()
                .map_or_else(|| self.transposition_table.probe_pv_move(game), Some)
        } else {
            self.transposition_table.probe_pv_move(game)
        };

        // TODO: move sorter, move pvNode first (with https://www.chessprogramming.org/Triangular_PV-Table or transposition table)
        self.options
            .action_sorter
            .sort_actions(&mut actions, pv_action.as_ref());

        // PV-Node should always be sorted first
        #[cfg(debug_assertions)]
        if pv_action.is_some() {
            let pv_action = pv_action.unwrap();
            if actions[0] != pv_action {
                println!("PV-Node action is not sorted first!");
                println!("PLY_FROM_ROOT {:?}", ply_from_root);
                println!("BEST_ACTION: {:?}", self.best_action);
                println!("PROBE PV: {:?}", self.transposition_table.probe_pv_move(game));
            }

            debug_assert_eq!(actions[0], pv_action);
        }

        let mut is_pv_node = true;
        let mut best_action = None;
        let mut alpha: isize = alpha;
        let mut evaluation_bound = EvaluationType::UpperBound;

        // TODO: late move pruning (LMP) (remove last actions in list while some conditions are not met, e.g. in check, depth, ...)
        for i in 0..actions.len() {
            let action = &actions[i];

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
                // Apply [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions) if we're not in the early moves (and this is not a PV node)
                // Reduce the depth of the search for later actions as these are less likely to be good (assuming the action ordering is good)
                // Code adapted from https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html
                // Search Extensions are not applied to the reduced depth search
                let mut needs_full_search = true;
                if extension == 0 && i >= PVSPlayer::LMR_FULL_DEPTH_ACTIONS && depth >= PVSPlayer::LMR_REDUCTION_LIMIT {
                    // Search this move with reduced depth
                    evaluation = -self.zero_window_search(game, depth - 1 - PVSPlayer::LMR_REDUCED_BY_DEPTH, -alpha)?;
                    needs_full_search = evaluation > alpha;
                }

                if needs_full_search {
                    // Null-window search
                    evaluation = -self.zero_window_search(
                        game,
                        depth - 1, // do not apply search extensions in zws
                        -alpha,
                    )?;

                    if evaluation > alpha && evaluation < beta {
                        self.diagnostics.increment_zero_window_search_fail();
                        // Re-search with full window
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

                self.transposition_table.store_evaluation_with_symmetries(
                    game,
                    depth,
                    beta,
                    EvaluationType::LowerBound,
                    action,
                );
                return Ok(beta); // fail-hard beta-cutoff
            }

            if evaluation > alpha {
                evaluation_bound = EvaluationType::Exact;
                alpha = evaluation; // alpha acts like max in MiniMax
                best_action = Some(action.clone());
            }

            is_pv_node = false;
        }

        // store null action in transposition table if it is a EvaluationType::UpperBound
        let null_action = Action::null();
        let transposition_table_action = best_action.as_ref().unwrap_or(&null_action);

        self.transposition_table.store_evaluation_with_symmetries(
            game,
            depth,
            alpha,
            evaluation_bound,
            transposition_table_action,
        );

        if ply_from_root == 0 {
            self.best_action = best_action;
            self.best_evaluation = Some(alpha);
        }

        Ok(alpha)
    }

    /// Does a Zero/Scout Window Search (ZWS) with the given parameters.
    ///
    /// fail-hard zero window search, returns either `beta-1` or `beta`
    /// only takes the beta parameter because `alpha == beta - 1`
    fn zero_window_search(&mut self, game: &mut Patchwork, depth: usize, beta: isize) -> PlayerResult<isize> {
        if self.search_canceled.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(0);
        }

        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        self.diagnostics.increment_nodes_searched();

        let actions = game.get_valid_actions();
        for action in actions {
            game.do_action(&action, true)?;

            let evaluation = -self.zero_window_search(
                game,
                depth - 1, // do not apply search extensions in zws
                1 - beta,
            )?;

            game.undo_action(&action, true)?;

            if evaluation >= beta {
                return Ok(beta); // fail-hard beta-cutoff
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
        if num_extensions >= Self::MAX_SEARCH_EXTENSIONS {
            return 0;
        }

        let mut extension = 0;

        // Extend the depth of search for special patch placements
        if matches!(
            game.turn_type,
            TurnType::SpecialPatchPlacement(_) | TurnType::SpecialPhantom(_)
        ) {
            self.diagnostics.increment_special_patch_extensions();
            // TODO: this will double extend with special phantom then special patch placement
            // we could not extend special phantom but that would be wrong as we could have a special phantom and already have depth 0
            // maybe change evaluation to go further
            extension = 1;
        }

        extension
    }

    // TODO: rename method and do something like quiescence search to mitigate the horizon effect (is this even needed in patchwork?)
    fn evaluation(&mut self, game: &mut Patchwork) -> PlayerResult<isize> {
        self.diagnostics.increment_nodes_searched();

        let mut needs_undo_action = false;
        if matches!(game.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom(_)) {
            // force a turn for phantom moves
            game.do_action(&Action::null(), true)?;
            needs_undo_action = true;
        }

        let color = game.get_current_player() as isize;

        // TODO: mate scores
        // TODO: do we want to keep the color as we can query whose turn it is via game?
        let evaluation = color * self.options.evaluator.evaluate_node(game);

        if needs_undo_action {
            // Reset to phantom action
            game.undo_action(&Action::null(), true)?;
        }

        Ok(evaluation)
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
        if let Some(ref mut writer) = self.options.diagnostics {
            writeln!(writer.as_mut(), "{}", diagnostic)?;
        }
        Ok(())
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
        include_transposition_table: bool,
    ) -> Result<(), std::io::Error> {
        if let Some(ref mut writer) = self.options.diagnostics {
            let writer = writer.as_mut();

            let time_diff = std::time::Instant::now().duration_since(self.diagnostics.start_time);
            let pv_actions = self
                .transposition_table
                .get_pv_line(game, depth)
                .iter()
                .map(|action| match action.save_to_notation() {
                    Ok(notation) => notation,
                    Err(_) => "######".to_string(),
                })
                .join(" → ");
            let best_evaluation = self
                .best_evaluation
                .map(|eval| format!("{}", eval))
                .unwrap_or("NONE".to_string());
            let best_action = self
                .best_action
                .as_ref()
                .map(|action| match action.save_to_notation() {
                    Ok(notation) => notation,
                    Err(_) => "######".to_string(),
                })
                .unwrap_or("NONE".to_string());
            let move_ordering = (self.diagnostics.fail_high_first as f64) / (self.diagnostics.fail_high as f64);

            writeln!(writer, "───────────── Principal Variation Search Player ─────────────")?;
            writeln!(writer, "Depth:               {:?}", depth)?;
            writeln!(writer, "Time:                {:?}", time_diff)?;
            writeln!(writer, "Nodes searched:      {:?}", self.diagnostics.nodes_searched)?;
            writeln!(writer, "Best Evaluation:     {}", best_evaluation)?;
            writeln!(writer, "Best Action:         {}", best_action)?;
            writeln!(writer, "Move Ordering:       {:?}", move_ordering)?;
            writeln!(writer, "Aspiration window:   {:?} low {:?}, high", self.diagnostics.aspiration_window_fail_low, self.diagnostics.aspiration_window_fail_high)?;
            writeln!(writer, "Zero window search:  {:?} fail", self.diagnostics.zero_window_search_fail)?;
            writeln!(writer, "Special patch ext.:  {:?}", self.diagnostics.special_patch_extensions)?;
            writeln!(writer, "Principal Variation: {}", pv_actions)?;

            if include_transposition_table {
                self.transposition_table.diagnostics.write_diagnostics(writer)?;
            }
            writeln!(writer, "─────────────────────────────────────────────────────────────")?;

            return Ok(());


            // TODO: remove
            // print out all entries of the transposition table
            writeln!(writer, "┌────────┬───────────────────── Transposition Table Entries ────────────┬─────────────────┐")?;
            writeln!(writer, "│ Index  │         Key          │ Depth │ Age │    Type    │ Evaluation │    Action       │")?;
            writeln!(writer, "├────────┼──────────────────────┼───────┼─────┼────────────┼────────────┼─────────────────┤")?;
            let mut written_entries = 0;
            for (index, entry) in self.transposition_table.entries.iter().enumerate() {
                if entry.key == 0 {
                    continue;
                }
                let unpacked_data = Entry::unpack_data(entry.data, entry.extra_data);
                if unpacked_data.is_none() {
                    continue;
                }

                if written_entries > 100 {
                    writeln!(writer, "│  ...   │         ...          │  ...  │ ... │     ...    │     ...    │       ...       │")?;
                    break;
                }

                let (table_depth, table_evaluation, table_evaluation_type, table_action) = unpacked_data.unwrap();

                written_entries += 1;
                writeln!(
                    writer,
                    "│{: >7?} │ {: >20?} | {: >5?} | {: >3?} | {: >10} | {: >10?} |{: >16} │",
                    index,
                    entry.key,
                    table_depth,
                    entry.age,
                    format!("{:?}", table_evaluation_type),
                    table_evaluation,
                    table_action.save_to_notation().unwrap_or("######".to_string())
                )?;
            }
            writeln!(writer, "└────────┴──────────────────────┴───────┴─────┴────────────┴────────────┴─────────────────┘")?;
        }

        Ok(())
    }
}
