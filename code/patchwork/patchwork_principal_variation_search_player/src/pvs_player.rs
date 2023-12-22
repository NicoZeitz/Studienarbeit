use patchwork_core::{Action, Patchwork, Player, PlayerResult};
use patchwork_evaluator::StaticEvaluator as Evaluator;

use crate::PVSOptions;

/// A computer player that uses the Principal Variation Search (PVS) algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PVSPlayer {
    /// The name of the player.
    pub name: String,
    /// The options for the Principal Variation Search (PVS) algorithm.
    pub options: PVSOptions,
    /// The evaluator to evaluate the game state.
    pub evaluator: Evaluator,
    /// The best action found so far.
    best_action: Option<Action>,
}

impl PVSPlayer {
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<PVSOptions>) -> Self {
        let options = options.unwrap_or_default();
        PVSPlayer {
            name: name.into(),
            options,
            evaluator: Default::default(),
            best_action: None,
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

    fn get_action(&mut self, _game: &Patchwork) -> PlayerResult<Action> {
        todo!()
        // self.search(game)
    }
}

impl PVSPlayer {
    // fn search(&mut self, game: &Patchwork) -> Action {
    //     let search_canceled = AtomicBool::new(false);

    //     std::thread::scope(|s| {
    //         s.spawn(|| {
    //             std::thread::sleep(self.options.time_limit);
    //             search_canceled.store(true, std::sync::atomic::Ordering::SeqCst);
    //         });

    //         let chosen_action = None;
    //         let alpha = isize::MIN;
    //         let beta = isize::MAX;

    //         // iterative deepening loop
    //         for depth in 1..usize::MAX {
    //             let (action, evaluation) = self.principal_variation_search(game, depth, alpha, beta);

    //             if search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
    //                 return action;
    //             }
    //         }

    //         // should not happen
    //         chosen_action.unwrap_or_else(|| game.get_valid_actions().into_iter().collect::<Vec<_>>()[0].clone())
    //     })
    // }

    // fn principal_variation_search(&mut self, game: Game, depth: usize, alpha: isize, beta: isize) {
    //     if depth == 0 || game.is_terminated() {
    //         return self.evaluator.evaluate_node(game);
    //     }

    //     let actions = game.get_valid_actions();

    //     for action in actions {}

    //     let mut bSearchPv = true;
    // }
    // // int pvSearch( int alpha, int beta, int depth ) {
    // //     if( depth == 0 ) return quiesce(alpha, beta);
    // //     bool bSearchPv = true;
    // //     for ( all moves)  {
    // //        make
    // //        if ( bSearchPv ) {
    // //           score = -pvSearch(-beta, -alpha, depth - 1);
    // //        } else {
    // //           score = -zwSearch(-alpha, depth - 1);
    // //           if ( score > alpha ) // in fail-soft ... && score < beta ) is common
    // //              score = -pvSearch(-beta, -alpha, depth - 1); // re-search
    // //        }
    // //        unmake
    // //        if( score >= beta )
    // //           return beta;   // fail-hard beta-cutoff
    // //        if( score > alpha ) {
    // //           alpha = score; // alpha acts like max in MiniMax
    // //           bSearchPv = false;   // *1)
    // //        }
    // //     }
    // //     return alpha;
    // //  }

    // //  // fail-hard zero window search, returns either beta-1 or beta
    // //  int zwSearch(int beta, int depth ) {
    // //     // alpha == beta - 1
    // //     // this is either a cut- or all-node
    // //     if( depth == 0 ) return quiesce(beta-1, beta);
    // //     for ( all moves)  {
    // //       make
    // //       score = -zwSearch(1-beta, depth - 1);
    // //       unmake
    // //       if( score >= beta )
    // //          return beta;   // fail-hard beta-cutoff
    // //     }
    // //     return beta-1; // fail-hard, return alpha
    // //  }

    // fn iterative_deepening_search<Game: game::Game>(&self, game: &Game) -> Game::Action {
    //     let search_canceled = AtomicBool::new(false);

    //     std::thread::scope(|s| {
    //         s.spawn(|| {
    //             std::thread::sleep(self.options.time_limit);
    //             search_canceled.store(true, std::sync::atomic::Ordering::SeqCst);
    //         });

    //         let maximizing_player = game.is_maximizing_player(&game.get_current_player());

    //         let mut chosen_action: Option<Game::Action> = None;
    //         let mut chosen_evaluation = if maximizing_player {
    //             f64::NEG_INFINITY
    //         } else {
    //             f64::INFINITY
    //         };

    //         for depth in 1..usize::MAX {
    //             if search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
    //                 break;
    //             }

    //             let (action, evaluation) =
    //                 PVSPlayer::principal_variation_search(game, f64::NEG_INFINITY, f64::INFINITY, depth, false);

    //             // break ties randomly
    //             if evaluation == chosen_evaluation && rand::random() {
    //                 chosen_action = Some(action);
    //             } else {
    //                 #[allow(clippy::collapsible_else_if)]
    //                 if maximizing_player {
    //                     if evaluation > chosen_evaluation {
    //                         chosen_action = Some(action);
    //                         chosen_evaluation = evaluation;
    //                     }
    //                 } else {
    //                     if evaluation < chosen_evaluation {
    //                         chosen_action = Some(action);
    //                         chosen_evaluation = evaluation;
    //                     }
    //                 }
    //             }
    //         }

    //         chosen_action.unwrap()
    //     })
    // }

    // fn principal_variation_search_old<Game: game::Game>(
    //     game: &Game,
    //     alpha: f64,
    //     beta: f64,
    //     depth: usize,
    //     force_null_move: bool,
    // ) -> f64 {
    //     if depth == 0 || game.is_terminated() {
    //         return evaluator.evaluate_node(game);
    //     }

    //     let mut b_search_pv = true;

    //     if force_null_move {
    //         // TODO:
    //     }

    //     let current_player = game.is_maximizing_player(&game.get_current_player());

    //     for action in game.get_valid_actions() {
    //         let next_state = game.do_action(&action);
    //         let next_player = game.is_maximizing_player(&game.get_current_player());
    //         let player_changed = current_player != next_player;

    //         let mut score;

    //         if b_search_pv {
    //             // TODO: minus
    //             score = -PVSPlayer::principal_variation_search(&next_state, -beta, -alpha, depth - 1, !player_changed);
    //         } else {
    //             // TODO: minus
    //             score = -PVSPlayer::zero_window_search(&next_state, -alpha, depth - 1, !player_changed);
    //             if score > alpha {
    //                 // TODO: minus
    //                 score =
    //                     -PVSPlayer::principal_variation_search(&next_state, -beta, -alpha, depth - 1, !player_changed);
    //             }
    //         };

    //         if score >= beta {
    //             return beta;
    //         }
    //         if score > alpha {
    //             alpha = score;
    //             b_search_pv = false;
    //         }
    //     }

    //     return alpha;
    // }

    // fn zero_window_search<Game: game::Game>(game: &Game, beta: f64, depth: usize, force_null_move: bool) -> f64 {
    //     if depth == 0 || game.is_terminated() {
    //         return evaluator.evaluate_node(game);
    //     }

    //     if force_null_move {
    //         // TODO:
    //     }

    //     let current_player = game.is_maximizing_player(&game.get_current_player());

    //     for action in game.get_valid_actions() {
    //         let next_state = game.do_action(&action);
    //         let next_player = game.is_maximizing_player(&game.get_current_player());
    //         let player_changed = current_player != next_player;

    //         // TODO: minus
    //         let score = -PVSPlayer::zero_window_search(&next_state, 1.0 - beta, depth - 1, !player_changed);

    //         if score >= beta {
    //             return beta;
    //         }
    //     }

    //     return beta - 1.0;
    // }
}

// int pvSearch( int alpha, int beta, int depth ) {
//     if( depth == 0 ) return quiesce(alpha, beta);
//     bool bSearchPv = true;
//     for ( all moves)  {
//        make
//        if ( bSearchPv ) {
//           score = -pvSearch(-beta, -alpha, depth - 1);
//        } else {
//           score = -zwSearch(-alpha, depth - 1);
//           if ( score > alpha ) // in fail-soft ... && score < beta ) is common
//              score = -pvSearch(-beta, -alpha, depth - 1); // re-search
//        }
//        unmake
//        if( score >= beta )
//           return beta;   // fail-hard beta-cutoff
//        if( score > alpha ) {
//           alpha = score; // alpha acts like max in MiniMax
//           bSearchPv = false;   // *1)
//        }
//     }
//     return alpha;
//  }

//  // fail-hard zero window search, returns either beta-1 or beta
//  int zwSearch(int beta, int depth ) {
//     // alpha == beta - 1
//     // this is either a cut- or all-node
//     if( depth == 0 ) return quiesce(beta-1, beta);
//     for ( all moves)  {
//       make
//       score = -zwSearch(1-beta, depth - 1);
//       unmake
//       if( score >= beta )
//          return beta;   // fail-hard beta-cutoff
//     }
//     return beta-1; // fail-hard, return alpha
//  }
