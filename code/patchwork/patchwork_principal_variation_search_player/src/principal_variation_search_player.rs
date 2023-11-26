use std::sync::atomic::AtomicBool;

use game::Player;
use patchwork_core::Patchwork;

/// A computer player that uses the Negamax algorithm to choose an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrincipalVariationSearchPlayer {
    /// The name of the player.
    pub name: String,
}

impl PrincipalVariationSearchPlayer {
    /// Creates a new [`PrincipalVariationSearchPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        PrincipalVariationSearchPlayer { name: name.into() }
    }
}

impl Default for PrincipalVariationSearchPlayer {
    fn default() -> Self {
        Self::new("Principal Variation Search Player".to_string())
    }
}

impl Player for PrincipalVariationSearchPlayer {
    type Game = Patchwork;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Self::Game) -> <Self::Game as game::Game>::Action {
        PrincipalVariationSearchPlayer::iterative_deepening_search(game)
    }
}

impl PrincipalVariationSearchPlayer {
    fn iterative_deepening_search<Game: game::Game>(game: &Game) -> Game::Action {
        let search_canceled = AtomicBool::new(false);

        std::thread::scope(|s| {
            s.spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(5)); // TODO: variable time
                search_canceled.store(true, std::sync::atomic::Ordering::SeqCst);
            });

            let maximizing_player = game.is_maximizing_player(&game.get_current_player());

            let mut chosen_action: Option<Game::Action> = None;
            let mut chosen_evaluation = if maximizing_player {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };

            for depth in 1..usize::MAX {
                if search_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                let (action, evaluation) =
                    PrincipalVariationSearchPlayer::principal_variation_search(
                        game,
                        f64::NEG_INFINITY,
                        f64::INFINITY,
                        depth,
                        false,
                    );

                // break ties randomly
                if evaluation == chosen_evaluation && rand::random() {
                    chosen_action = Some(action);
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if maximizing_player {
                        if evaluation > chosen_evaluation {
                            chosen_action = Some(action);
                            chosen_evaluation = evaluation;
                        }
                    } else {
                        if evaluation < chosen_evaluation {
                            chosen_action = Some(action);
                            chosen_evaluation = evaluation;
                        }
                    }
                }
            }

            chosen_action.unwrap()
        })
    }

    fn principal_variation_search<Game: game::Game>(
        game: &Game,
        alpha: f64,
        beta: f64,
        depth: usize,
        force_null_move: bool,
    ) -> f64 {
        if depth == 0 || game.is_terminated() {
            return evaluator.evaluate_node(game);
        }

        let mut b_search_pv = true;

        if force_null_move {
            // TODO:
        }

        let current_player = game.is_maximizing_player(&game.get_current_player());

        for action in game.get_valid_actions() {
            let next_state = game.get_next_state(&action);
            let next_player = game.is_maximizing_player(&game.get_current_player());
            let player_changed = current_player != next_player;

            let mut score;

            if b_search_pv {
                // TODO: minus
                score = -PrincipalVariationSearchPlayer::principal_variation_search(
                    &next_state,
                    -beta,
                    -alpha,
                    depth - 1,
                    !player_changed,
                );
            } else {
                // TODO: minus
                score = -PrincipalVariationSearchPlayer::zero_window_search(
                    &next_state,
                    -alpha,
                    depth - 1,
                    !player_changed,
                );
                if score > alpha {
                    // TODO: minus
                    score = -PrincipalVariationSearchPlayer::principal_variation_search(
                        &next_state,
                        -beta,
                        -alpha,
                        depth - 1,
                        !player_changed,
                    );
                }
            };

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                b_search_pv = false;
            }
        }

        return alpha;
    }

    fn zero_window_search<Game: game::Game>(
        game: &Game,
        beta: f64,
        depth: usize,
        force_null_move: bool,
    ) -> f64 {
        if depth == 0 || game.is_terminated() {
            return evaluator.evaluate_node(game);
        }

        if force_null_move {
            // TODO:
        }

        let current_player = game.is_maximizing_player(&game.get_current_player());

        for action in game.get_valid_actions() {
            let next_state = game.get_next_state(&action);
            let next_player = game.is_maximizing_player(&game.get_current_player());
            let player_changed = current_player != next_player;

            // TODO: minus
            let score = -PrincipalVariationSearchPlayer::zero_window_search(
                &next_state,
                1.0 - beta,
                depth - 1,
                !player_changed,
            );

            if score >= beta {
                return beta;
            }
        }

        return beta - 1.0;
    }
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
