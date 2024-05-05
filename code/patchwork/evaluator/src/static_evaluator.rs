use patchwork_core::{Evaluator, Patchwork, QuiltBoard, StableEvaluator, TimeBoard};

/// A static evaluator for [`Patchwork`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticEvaluator;

impl StaticEvaluator {
    /// Creates a new [`StaticEvaluator`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for StaticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl StableEvaluator for StaticEvaluator {}
impl Evaluator for StaticEvaluator {
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32 {
        let player_1_score = self.evaluate_state_for_player(game, Patchwork::get_player_1_flag());
        let player_2_score = self.evaluate_state_for_player(game, Patchwork::get_player_2_flag());
        (player_1_score - player_2_score) as i32
    }
}

impl StaticEvaluator {
    #[rustfmt::skip]
    const BOARD: [[bool; QuiltBoard::COLUMNS as usize + 2]; QuiltBoard::ROWS as usize + 2] = [
        [true, true,  true,  true,  true,  true,  true,  true,  true,  true,  true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false, false, true],
        [true, true,  true,  true,  true,  true,  true,  true,  true,  true,  true],
    ];

    /// Evaluates the given game state for the given player.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to evaluate.
    /// * `player` - The player to evaluate the game state for.
    ///
    /// # Returns
    ///
    /// The evaluation of the game state for the given player.
    #[must_use]
    pub fn evaluate_state_for_player(&self, game: &Patchwork, player: u8) -> f64 {
        let player_state = game.get_player(player);
        let quilt_board = &player_state.quilt_board;
        let percentage_played = f64::from(player_state.get_position()) / f64::from(TimeBoard::MAX_POSITION);

        let end_score = f64::from(game.get_score(player));
        let position_score = f64::from(TimeBoard::MAX_POSITION - player_state.get_position());
        let board_score = f64::from(get_board_score(quilt_board));
        let button_income_score = get_button_income_score(
            f64::from(quilt_board.button_income),
            &game.time_board,
            player_state.get_position(),
        );
        // let free_single_tiles_score = get_free_single_tiles_score(quilt_board);
        // let free_region_score = self.get_free_region_score(quilt_board);

        (board_score * 2.0).mul_add(
            1.0 - percentage_played,
            (end_score * 2.0).mul_add(percentage_played, position_score),
        ) + button_income_score
    }
}

#[rustfmt::skip]
#[allow(clippy::unused_self)]
fn get_board_score(quilt_board: &QuiltBoard) -> i32 {
    let mut board = StaticEvaluator::BOARD;
    for row in 1..=QuiltBoard::ROWS {
        for col in 1..=QuiltBoard::COLUMNS {
            board[row as usize + 1][col as usize + 1] = quilt_board.get(row, col);
        }
    }

    let mut board_score = i32::from(QuiltBoard::TILES) * 9;

    for row in 1..=(QuiltBoard::ROWS as usize) {
        for col in 1..=(QuiltBoard::COLUMNS as usize) {
            if !board[row][col] {
                board_score -= 9;
                continue;
            }

            // Moore neighborhood
            board_score -= i32::from(!board[row + 1][col + 1]);
            board_score -= i32::from(!board[row + 1][col    ]);
            board_score -= i32::from(!board[row + 1][col - 1]);
            board_score -= i32::from(!board[row    ][col + 1]);
            board_score -= i32::from(!board[row    ][col    ]);
            board_score -= i32::from(!board[row    ][col - 1]);
            board_score -= i32::from(!board[row - 1][col + 1]);
            board_score -= i32::from(!board[row - 1][col    ]);
            board_score -= i32::from(!board[row - 1][col - 1]);
        }
    }

    board_score
}

fn get_button_income_score(button_income: f64, time_board: &TimeBoard, position: u8) -> f64 {
    let amount_button_income_triggers_left = time_board.get_amount_button_income_trigger_in_range(
        ((position + 1).min(TimeBoard::MAX_POSITION) as usize)..(TimeBoard::MAX_POSITION + 1) as usize,
    );
    let amount_button_income_triggers_passed =
        TimeBoard::AMOUNT_OF_BUTTON_INCOME_TRIGGERS as i32 - amount_button_income_triggers_left as i32;

    // f(x) = 8exp(ln(1/8) * x / 8)
    8.0 * ((1.0 / 8f64).ln() * f64::from(amount_button_income_triggers_passed) / 8.0).exp() * button_income
}
