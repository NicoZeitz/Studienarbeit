use std::sync::{atomic::AtomicBool, Arc};

use patchwork_core::{Action, Evaluator as EvaluatorTrait, Patchwork, Player, PlayerResult};
use patchwork_evaluator::StaticEvaluator as Evaluator;

use crate::PVSOptions;

/// The diagnostics of a search.
/// TODO: Implement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SearchDiagnostics {
    pub nodes: usize,
}

/// A computer player that uses the Principal Variation Search (PVS) algorithm to choose an action.
///
/// # Features
/// - [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening)
/// - [Alpha-Beta Pruning](https://www.chessprogramming.org/Alpha-Beta)
/// - [Principal Variation Search (PVS)](https://www.chessprogramming.org/Principal_Variation_Search)
/// - [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows)
/// - [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions)
#[derive(Debug, Clone)]
pub struct PVSPlayer {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// The evaluator to evaluate the game state.
    pub evaluator: Evaluator,
    /// search diagnostics
    pub diagnostics: SearchDiagnostics,
    /// The best action found so far.
    best_action: Option<Action>,
    /// Whether the search has been canceled.
    search_canceled: Arc<AtomicBool>,
}

impl PVSPlayer {
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<PVSOptions>) -> Self {
        let options = options.unwrap_or_default();
        PVSPlayer {
            name: name.into(),
            options,
            evaluator: Default::default(),
            diagnostics: Default::default(),
            best_action: None,
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

            // stop search after time limit
            s.spawn(move || {
                std::thread::sleep(time_limit);
                search_canceled.store(true, std::sync::atomic::Ordering::SeqCst);
            });

            let mut game = game.clone();

            // do the search
            let action = self.search(&mut game)?;

            // reset the parameters for the next search
            self.diagnostics = Default::default();
            self.search_canceled.store(false, std::sync::atomic::Ordering::SeqCst);

            Ok(action)
        })
    }
}

impl PVSPlayer {
    const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;
    const POSITIVE_INFINITY: f64 = f64::INFINITY;
    const LMR_REDUCED_BY_DEPTH: usize = 1;
    const LMR_FULL_DEPTH_ACTIONS: usize = 4;
    const LMR_REDUCTION_LIMIT: usize = 3;

    fn search(&mut self, game: &mut Patchwork) -> PlayerResult<Action> {
        let mut delta = 100.0; // TODO: test which value should be used
        let mut alpha = PVSPlayer::NEGATIVE_INFINITY; // TODO: initialize to score_estimate - delta
        let mut beta = PVSPlayer::POSITIVE_INFINITY; // TODO: initialize to score_estimate + delta
        let mut depth = 1;

        // [Iterative Deepening](https://www.chessprogramming.org/Iterative_Deepening) loop
        while depth < usize::MAX {
            let evaluation = self.principal_variation_search(game, depth, alpha, beta)?;

            if self.search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            // [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows) with exponential backoff
            if evaluation <= alpha {
                // Evaluation is below aspiration window [Fail-Low](https://www.chessprogramming.org/Fail-Low#Root_with_Aspiration)
                // The best found evaluation is less than or equal to the lower bound (alpha), so we need to research at the same depth
                beta = (alpha + beta) / 2.0; // adjust beta towards alpha
                alpha = evaluation - delta;
                delta += delta / 3.0; // use same exponential backoff as in [Stockfish](https://github.com/official-stockfish/Stockfish/blob/master/src/search.cpp#L429C17-L429C36)
                continue;
            } else if evaluation >= beta {
                // Evaluation is above aspiration window [Fail-High](https://www.chessprogramming.org/Fail-High#Root_with_Aspiration)
                // The best found evaluation is greater or equal to the upper bound (beta), so we need to research at the same depth
                beta = evaluation + delta;
                delta += delta / 3.0;
                continue;
            }

            // Evaluation is within the aspiration window,
            // so we can move on to the next depth with a window set around the eval

            delta = 100.0; // TODO: use avg of root node scores like in Stockfish `delta = Value(9) + int(avg) * avg / 14847;`
            alpha = evaluation - delta;
            beta = evaluation + delta;
            depth += 1;
        }

        Ok(self.best_action.take().unwrap_or_else(|| {
            println!("No best action found. Returning random valid action. (Walking) THIS SHOULD NOT HAPPEN!");
            game.get_random_action()
        }))
    }

    /// Does a Principal Variation Search (PVS) with the given parameters.
    #[allow(clippy::needless_range_loop)]
    fn principal_variation_search(
        &mut self,
        game: &mut Patchwork,
        depth: usize,
        alpha: f64,
        beta: f64,
    ) -> PlayerResult<f64> {
        // TODO: fast cutoff if search time is up

        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        let actions = game.get_valid_actions();

        // TODO: Save Principal Variation (PV)

        // TODO: is this useful for patchwork?
        // NULL MOVE PRUNING
        //       // Null move search:
        //       if(ok_to_do_nullmove_at_this_node()) {
        //         make_nullmove();
        //         value = -search(-beta, -beta, -(beta-1), depth-4);
        //         unmake_nullmove();
        //         if(value >= beta) return value;
        //       }

        // TODO: move sorter, move pvNode first
        // TODO: late move pruning (remove last actions in list while some conditions are not met, e.g. in check, depth, ...)

        let mut is_pv_node = true;
        let mut best_action = Some(actions[0].clone());
        let mut alpha = alpha;

        for i in 0..actions.len() {
            let action = &actions[i];

            game.do_action(action, true)?;

            let mut evaluation = f64::NAN;
            if is_pv_node {
                // Full window search for pv node
                evaluation = -self.principal_variation_search(game, depth - 1, -beta, -alpha)?
            } else {
                // Apply [Late Move Reductions (LMR)](https://www.chessprogramming.org/Late_Move_Reductions) if we're not in the early moves (and this is not a PV node)
                // Reduce the depth of the search for later actions as these are less likely to be good (assuming the action ordering is good)
                // Code adapted from https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html
                // TODO: add search extensions (e.g. special patch placement) and ignore these here (By LMR)
                let mut needs_full_search = true;
                if i >= PVSPlayer::LMR_FULL_DEPTH_ACTIONS && depth >= PVSPlayer::LMR_REDUCTION_LIMIT
                /* && ok_to_reduce() */
                {
                    // Search this move with reduced depth
                    evaluation = -self.zero_window_search(game, depth - 1 - PVSPlayer::LMR_REDUCED_BY_DEPTH, -alpha)?;
                    needs_full_search = evaluation > alpha;
                }

                if needs_full_search {
                    // Null-window search
                    evaluation = -self.zero_window_search(game, depth - 1, -alpha)?;

                    if evaluation > alpha && evaluation < beta {
                        // Re-search with full window
                        evaluation = -self.principal_variation_search(game, depth - 1, -beta, -alpha)?;
                    }
                }

                debug_assert!(!f64::is_nan(evaluation));
            }

            game.undo_action(action, true)?;

            if evaluation >= beta {
                return Ok(beta); // fail-hard beta-cutoff
            }
            if evaluation > alpha {
                alpha = evaluation; // alpha acts like max in MiniMax
                best_action = Some(action.clone());
            }
            is_pv_node = false;
        }

        self.best_action = best_action;

        // TODO: Transposition Table with Symmetry Reduction

        Ok(alpha)
    }

    /// Does a Zero/Scout Window Search (ZWS) with the given parameters.
    ///
    /// fail-hard zero window search, returns either `beta-1` or `beta`
    /// only takes the beta parameter because `alpha == beta - 1`
    fn zero_window_search(&mut self, game: &mut Patchwork, depth: usize, beta: f64) -> PlayerResult<f64> {
        // TODO: fast cutoff if search time is up

        if depth == 0 || game.is_terminated() {
            return self.evaluation(game);
        }

        let actions = game.get_valid_actions();
        for action in actions {
            game.do_action(&action, true)?;

            let evaluation = -self.zero_window_search(game, depth - 1, 1.0 - beta)?;

            game.undo_action(&action, true)?;

            if evaluation >= beta {
                return Ok(beta); // fail-hard beta-cutoff
            }
        }
        Ok(beta - 1.0) // fail-hard, return alpha
    }

    // TODO: rename method and do search extensions and so on
    fn evaluation(&mut self, game: &Patchwork) -> PlayerResult<f64> {
        Ok(self.evaluator.evaluate_node(game))
    }
}
