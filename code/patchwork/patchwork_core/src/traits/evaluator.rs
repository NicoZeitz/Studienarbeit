use crate::{Patchwork, TerminationType};

pub mod evaluator_constants {
    /// The maximum evaluation for a winning game. No evaluation should be higher than this.
    pub const POSITIVE_INFINITY: i32 = 100_000;
    /// The minimum evaluation for a losing game. No evaluation should be lower than this.
    pub const NEGATIVE_INFINITY: i32 = -POSITIVE_INFINITY;
}

/// A game evaluator for the 2 player game Patchwork.
///
/// The evaluation is always an i32 ([Reason for Integer evaluations](https://www.talkchess.com/forum3/viewtopic.php?t=22817))
///
/// # Rules that a game evaluator must follow
///
/// * The maximum evaluation only for a winning game is [`evaluator_constants::POSITIVE_INFINITY`].
/// * The minimum evaluation only for a losing game is [`evaluator_constants::NEGATIVE_INFINITY`].
/// * The evaluation of a draw is always `0`.
/// * All other evaluations must be in between these values.
///   `0` is also allowed for positions that are evaluated as equal.
/// * The evaluation at the start of the game should be `0`.
/// * The evaluator is not required to return the same evaluation for equal states.
///   If the evaluator does return the same evaluation for equal states, the [`StableEvaluator`] trait
///   should be implemented.
/// * The evaluation is in terms of 1/100 of the end result. So if the evaluation at the end of the game is 10 for player 1 and -10 for player 2,
///   the evaluator should return 1000 for player 1 and -1000 for player 2. (This is not required, but it is recommended)
pub trait Evaluator: Sync + Send {
    /// Returns the evaluation of the given intermediate state.
    /// An intermediate state is a state that is not terminal.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32;

    /// Returns the evaluation of the given terminal state. Should be one of the following:
    /// * [`evaluator_constants::POSITIVE_INFINITY`] - for a win of player 1 / loss of player 2
    /// * [`evaluator_constants::NEGATIVE_INFINITY`] - for a loss of player 1 / win of player 2
    /// * 0 - for a draw between player 1 and player 2
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_terminal_node(&self, game: &Patchwork) -> i32 {
        match game.get_termination_result().termination {
            TerminationType::Player1Won => evaluator_constants::POSITIVE_INFINITY,
            TerminationType::Player2Won => evaluator_constants::NEGATIVE_INFINITY,
        }
    }

    /// Returns the evaluation of the given state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    ///
    /// # Returns
    ///
    /// The evaluation of the given state.
    fn evaluate_node(&self, game: &Patchwork) -> i32 {
        let score = if game.is_terminated() {
            self.evaluate_terminal_node(game)
        } else {
            self.evaluate_intermediate_node(game)
        };

        #[cfg(debug_assertions)]
        if !(evaluator_constants::NEGATIVE_INFINITY..=evaluator_constants::POSITIVE_INFINITY).contains(&score) {
            println!("Game: {}", game);
            println!("Score: {}", score);

            panic!("The score is not in the allowed range.");
        }

        score
    }
}

/// A game evaluator that is stable.
/// This means equal game states will always be evaluated the same.
pub trait StableEvaluator: Evaluator {}
