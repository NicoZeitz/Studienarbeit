use std::f64::consts::E;

use game::{Evaluator, Game};
use patchwork_core::{Patchwork, QuiltBoard, TerminationType, TimeBoard};

/// A static evaluator for [`Patchwork`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticEvaluator;

impl StaticEvaluator {
    /// Creates a new [`StaticEvaluator`].
    pub fn new() -> Self {
        StaticEvaluator {}
    }
}

impl Default for StaticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for StaticEvaluator {
    type Game = Patchwork;

    fn evaluate_intermediate_node(&self, game: &Self::Game) -> f64 {
        let player_1_score = self.evaluate_state_for_player(game, &game.get_player_1_flag());
        let player_2_score = self.evaluate_state_for_player(game, &game.get_player_2_flag());

        player_1_score - player_2_score
    }

    fn evaluate_terminal_node(&self, game: &Self::Game) -> f64 {
        match game.get_termination_result().termination {
            TerminationType::Player1Won => f64::INFINITY,
            TerminationType::Player2Won => f64::NEG_INFINITY,
            TerminationType::Draw => 0.0,
        }
    }
}

impl StaticEvaluator {
    #[rustfmt::skip]
    const BOARD: [[bool; QuiltBoard::COLUMNS + 2]; QuiltBoard::ROWS + 2] = [
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
    fn evaluate_state_for_player(
        &self,
        game: &Patchwork,
        player: &<Patchwork as Game>::Player,
    ) -> f64 {
        let player_state = game.get_player(*player);
        let quilt_board = &player_state.quilt_board;
        let percentage_played = player_state.position as f64 / TimeBoard::MAX_POSITION as f64;

        let end_score = game.get_score(*player) as f64;
        let position_score = (TimeBoard::MAX_POSITION - player_state.position) as f64;
        let board_score = self.get_board_score(quilt_board);
        let button_income_score = self.get_button_income_score(
            quilt_board.button_income as f64,
            &game.time_board,
            player_state.position,
        );
        // let free_single_tiles_score = get_free_single_tiles_score(quilt_board);
        // let free_region_score = self.get_free_region_score(quilt_board);

        end_score * 2.0 * percentage_played
            + position_score
            + board_score * 2.0 * (1.0 - percentage_played)
            + button_income_score
    }

    #[rustfmt::skip]
    fn get_board_score(&self, quilt_board: &QuiltBoard) -> f64 {
        let mut board = Self::BOARD;
        for row in 1..(QuiltBoard::ROWS + 1) {
            for col in 1..(QuiltBoard::COLUMNS + 1) {
                board[row + 1][col + 1] = quilt_board.get(row, col);
            }
        }

        let mut board_score = QuiltBoard::TILES * 9;

        for row in 1..(QuiltBoard::ROWS + 1) {
            for col in 1..(QuiltBoard::COLUMNS + 1) {
                if !board[row][col] {
                    board_score -= 9;
                    continue;
                }

                // Moore neighborhood
                board_score -= !board[row + 1][col + 1] as usize;
                board_score -= !board[row + 1][col    ] as usize;
                board_score -= !board[row + 1][col - 1] as usize;
                board_score -= !board[row    ][col + 1] as usize;
                board_score -= !board[row    ][col    ] as usize;
                board_score -= !board[row    ][col - 1] as usize;
                board_score -= !board[row - 1][col + 1] as usize;
                board_score -= !board[row - 1][col    ] as usize;
                board_score -= !board[row - 1][col - 1] as usize;
            }
        }

        board_score as f64
    }

    fn get_button_income_score(
        &self,
        button_income: f64,
        time_board: &TimeBoard,
        position: usize,
    ) -> f64 {
        let amount_button_income_triggers_left = time_board
            .get_amount_button_income_triggers_in_range(
                &((position + 1).min(TimeBoard::MAX_POSITION)..TimeBoard::MAX_POSITION + 1),
            );
        let amount_button_income_triggers_passed =
            TimeBoard::AMOUNT_BUTTON_INCOME_TRIGGERS as i32 - amount_button_income_triggers_left;

        // f(x) = 8exp(ln(1/8) * x / 8)
        8.0 * E.powf((amount_button_income_triggers_passed as f64 / 8.0).ln() / 8.0) * button_income
    }

    #[allow(dead_code)]
    fn get_free_single_tiles_score(&self, quilt_board: &QuiltBoard) -> f64 {
        let mut free_single_tiles_score = 0.0;

        for row in 1..(QuiltBoard::ROWS + 1) {
            for col in 1..(QuiltBoard::COLUMNS + 1) {
                if quilt_board.get(row, col) {
                    continue;
                }

                let mut is_free_single_tile = true;

                // Moore neighborhood
                is_free_single_tile &= !quilt_board.get(row + 1, col + 1);
                is_free_single_tile &= !quilt_board.get(row + 1, col);
                is_free_single_tile &= !quilt_board.get(row + 1, col - 1);
                is_free_single_tile &= !quilt_board.get(row, col + 1);
                is_free_single_tile &= !quilt_board.get(row, col - 1);
                is_free_single_tile &= !quilt_board.get(row - 1, col + 1);
                is_free_single_tile &= !quilt_board.get(row - 1, col);
                is_free_single_tile &= !quilt_board.get(row - 1, col - 1);

                free_single_tiles_score += is_free_single_tile as usize as f64;
            }
        }

        free_single_tiles_score
    }

    /// Counts the amount of spaces per region and rewards bigger regions more
    #[allow(dead_code)]
    fn get_free_region_score(&self, quilt_board: &QuiltBoard) -> f64 {
        let mut free_region_score = 0.0;

        let mut visited = [[false; QuiltBoard::COLUMNS]; QuiltBoard::ROWS];

        for row in 0..QuiltBoard::ROWS {
            for col in 0..QuiltBoard::COLUMNS {
                if visited[row][col] || quilt_board.get(row, col) {
                    continue;
                }

                let mut region_size = 0;
                let mut stack = vec![(row, col)];

                while let Some((row, col)) = stack.pop() {
                    // overflow will wrap around
                    if row >= QuiltBoard::ROWS || col >= QuiltBoard::COLUMNS {
                        continue;
                    }

                    if visited[row][col] || quilt_board.get(row, col) {
                        continue;
                    }

                    visited[row][col] = true;
                    region_size += 1;

                    stack.push((row + 1, col + 1));
                    stack.push((row + 1, col));
                    stack.push((row + 1, col - 1));
                    stack.push((row, col + 1));
                    stack.push((row, col - 1));
                    stack.push((row - 1, col + 1));
                    stack.push((row - 1, col));
                    stack.push((row - 1, col - 1));
                }

                free_region_score += 81.0 / region_size as f64;
            }
        }

        free_region_score
    }
}
