use patchwork_core::{Evaluator, Patchwork, QuiltBoard, StableEvaluator};

use crate::StaticEvaluator;

/// A static evaluator for [`Patchwork`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeavyStaticEvaluator;

impl HeavyStaticEvaluator {
    /// Creates a new [`HeavyStaticEvaluator`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for HeavyStaticEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl StableEvaluator for HeavyStaticEvaluator {}
impl Evaluator for HeavyStaticEvaluator {
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32 {
        let player_1_score = evaluate_state_for_player(game, Patchwork::get_player_1_flag());
        let player_2_score = evaluate_state_for_player(game, Patchwork::get_player_2_flag());
        (player_1_score - player_2_score) as i32
    }
}

fn evaluate_state_for_player(game: &Patchwork, player: u8) -> f64 {
    let quilt_board = &game.get_player(player).quilt_board;

    // TODO: real evaluation
    // static evaluator
    // + region getter (exponential debuff for 1,2,3,... free places 50,25,12.5,...)
    // maybe region tester if a patch that is still available fits into it
    let normal_eval = StaticEvaluator.evaluate_state_for_player(game, player);
    let free_single_tiles_score = get_free_single_tiles_score(quilt_board);
    let free_region_score = get_free_region_score(quilt_board);

    normal_eval + free_single_tiles_score + free_region_score
}

fn get_free_single_tiles_score(quilt_board: &QuiltBoard) -> f64 {
    let mut free_single_tiles_score = 0.0;

    for row in 1..=QuiltBoard::ROWS {
        for col in 1..=QuiltBoard::COLUMNS {
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

            free_single_tiles_score += usize::from(is_free_single_tile) as f64;
        }
    }

    free_single_tiles_score
}

/// Counts the amount of spaces per region and rewards bigger regions more
fn get_free_region_score(quilt_board: &QuiltBoard) -> f64 {
    let mut free_region_score = 0.0;

    let mut visited = [[false; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize];

    for row in 0..QuiltBoard::ROWS {
        for col in 0..QuiltBoard::COLUMNS {
            if visited[row as usize][col as usize] || quilt_board.get(row, col) {
                continue;
            }

            let mut region_size = 0;
            let mut stack = vec![(row, col)];

            while let Some((row, col)) = stack.pop() {
                // overflow will wrap around
                if row >= QuiltBoard::ROWS || col >= QuiltBoard::COLUMNS {
                    continue;
                }

                if visited[row as usize][col as usize] || quilt_board.get(row, col) {
                    continue;
                }

                visited[row as usize][col as usize] = true;
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

            free_region_score += 81.0 / f64::from(region_size);
        }
    }

    free_region_score
}
