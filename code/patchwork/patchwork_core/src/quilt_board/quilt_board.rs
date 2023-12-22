use std::fmt::Display;

use crate::{Action, ActionPayload, Patch, PatchManager};

// The quilt board of the player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuiltBoard {
    /// The tiles of the quilt board.
    pub(crate) tiles: u128,
    /// The amount of buttons this board generates.
    pub button_income: i32,
}

impl Default for QuiltBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl QuiltBoard {
    /// The amount of rows on the quilt board
    pub const ROWS: usize = 9;
    /// The amount of columns on the quilt board
    pub const COLUMNS: usize = 9;
    /// The amount of tiles on the quilt board
    pub const TILES: usize = QuiltBoard::ROWS * QuiltBoard::COLUMNS;

    /// Creates a new [`QuiltBoard`] which is empty.
    ///
    /// # Returns
    ///
    /// A new [`QuiltBoard`] which is empty.
    pub fn new() -> QuiltBoard {
        QuiltBoard {
            tiles: 0,
            button_income: 0,
        }
    }

    /// Whether the board is full.
    ///
    /// # Returns
    ///
    /// Whether the board is full.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn is_full(&self) -> bool {
        self.tiles.count_ones() == QuiltBoard::TILES as u32
    }

    /// The amount of tiles that are filled.
    ///
    /// # Returns
    ///
    /// The amount of tiles that are filled.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn tiles_filled(&self) -> u32 {
        self.tiles.count_ones()
    }

    /// The percentage of tiles that are filled.
    ///
    /// # Returns
    ///
    /// The percentage of tiles that are filled.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn percent_full(&self) -> f32 {
        self.tiles.count_ones() as f32 / QuiltBoard::TILES as f32
    }

    /// The score the player has with this quilt board.
    ///
    /// The score is calculated by taking the amount of tiles that are not filled and multiplying it by -2.
    ///
    /// # Returns
    ///
    /// The score the player has with this quilt board.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn score(&self) -> i32 {
        -2 * (QuiltBoard::TILES as u32 - self.tiles_filled()) as i32
    }

    /// Gets the tile at the given row and column.
    ///
    /// # Arguments
    ///
    /// * `row` - The row of the tile.
    /// * `column` - The column of the tile.
    ///
    /// # Returns
    ///
    /// The tile at the given row and column.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get(&self, row: usize, column: usize) -> bool {
        let index = row * QuiltBoard::COLUMNS + column;
        (self.tiles >> index) & 1 > 0
    }

    /// Gets the row at the given index.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to get.
    ///
    /// # Returns
    ///
    /// The row at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_row(&self, row: usize) -> u16 {
        let start = row * QuiltBoard::COLUMNS;
        let end = start + QuiltBoard::COLUMNS;
        (self.tiles >> start) as u16 & ((1 << end) - 1)
    }

    /// Gets the column at the given index.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to get.
    ///
    /// # Returns
    ///
    /// The column at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of rows, which is usually 9.
    pub fn get_column(&self, column: usize) -> u16 {
        let mut result = 0;
        for row in 0..QuiltBoard::ROWS {
            let index = row * QuiltBoard::COLUMNS + column;
            result |= (self.tiles >> index) & 1 << row;
        }
        result as u16
    }

    /// Applies the given action to the quilt board.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to apply.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)` when the `next_quilt_board` is given inside the action payload or the action is a special patch placement.
    ///
    /// `𝒪(𝑛)` otherwise where `𝑛` is the number of transformations for the given patch.
    pub fn do_action(&mut self, action: &Action) {
        match &action.payload {
            ActionPayload::PatchPlacement {
                next_quilt_board,
                patch,
                patch_orientation,
                patch_rotation,
                row,
                column,
                ..
            } => {
                self.button_income += patch.button_income as i32;

                if let Some(next_quilt_board) = *next_quilt_board {
                    self.tiles = next_quilt_board;
                    return;
                }

                // should only happen when the action was restored from notation
                // so this can be slow
                for transformation in PatchManager::get_instance().get_transformations(patch.id) {
                    if transformation.row != *row || transformation.column != *column {
                        continue;
                    }

                    if transformation.orientation_flag() as usize != *patch_orientation
                        || transformation.rotation_flag() as usize != *patch_rotation
                    {
                        continue;
                    }

                    self.tiles |= transformation.tiles;
                    break;
                }
            }
            ActionPayload::SpecialPatchPlacement {
                next_quilt_board,
                row,
                column,
                ..
            } => {
                if let Some(next_quilt_board) = *next_quilt_board {
                    self.tiles = next_quilt_board;
                    return;
                }

                // should only happen when the action was restored from notation
                let index = row * QuiltBoard::COLUMNS + column;

                // TODO: return error here instead of panic + add another safety check in the other branch
                #[cfg(debug_assertions)]
                if (self.tiles >> index) & 1 > 0 {
                    panic!(
                        "Invalid action! The tile at row {} and column {} is already filled!",
                        row, column
                    );
                }

                self.tiles |= 1 << index;
            }
            _ => {}
        }
    }

    pub(crate) fn undo_action(&mut self, action: &Action) {
        match action.payload {
            ActionPayload::PatchPlacement {
                #[cfg(debug_assertions)]
                next_quilt_board,
                previous_quilt_board,
                patch,
                patch_orientation,
                patch_rotation,
                row,
                column,
                ..
            } => {
                #[cfg(debug_assertions)]
                if let Some(next_quilt_board) = next_quilt_board {
                    if next_quilt_board != self.tiles {
                        println!("next:     {:081b}", next_quilt_board);
                        println!("tiles:    {:081b}", self.tiles);
                        println!("previous: {:081b}", previous_quilt_board.unwrap());

                        panic!("Invalid action! The next quilt board is not the same as the current one!");
                    }
                }

                self.button_income -= patch.button_income as i32;

                if let Some(previous_quilt_board) = previous_quilt_board {
                    self.tiles = previous_quilt_board;
                    return;
                }

                // should only happen when the action was restored from notation
                for transformation in PatchManager::get_instance().get_transformations(patch.id) {
                    if transformation.row != row || transformation.column != column {
                        continue;
                    }

                    if transformation.orientation_flag() as usize != patch_orientation
                        || transformation.rotation_flag() as usize != patch_rotation
                    {
                        continue;
                    }

                    self.tiles &= !transformation.tiles;
                    break;
                }
            }
            ActionPayload::SpecialPatchPlacement {
                #[cfg(debug_assertions)]
                next_quilt_board,
                previous_quilt_board,
                row,
                column,
                ..
            } => {
                #[cfg(debug_assertions)]
                if let Some(next_quilt_board) = next_quilt_board {
                    if next_quilt_board != self.tiles {
                        panic!("Invalid action! The next quilt board is not the same as the current one!");
                    }
                }

                if let Some(previous_quilt_board) = previous_quilt_board {
                    self.tiles = previous_quilt_board;
                    return;
                }

                // should only happen when the action was restored from notation
                let index = row * QuiltBoard::COLUMNS + column;

                #[cfg(debug_assertions)]
                if (self.tiles >> index) & 1 == 0 {
                    panic!(
                        "Invalid action! The tile at row {} and column {} is not filled!",
                        row, column
                    );
                }

                self.tiles &= !(1 << index);
            }
            _ => {}
        }
    }

    /// Gets the valid actions for the given patch.
    ///
    /// # Arguments
    ///
    /// * `patch` - The patch to get the valid actions for.
    /// * `patch_index` - The index of the patch in the list of patches.
    ///
    /// # Returns
    ///
    /// The valid actions for the given patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of transformations for the given patch.
    pub fn get_valid_actions_for_patch(
        &self,
        patch: &'static Patch,
        patch_index: usize,
        starting_index: usize,
    ) -> Vec<Action> {
        let mut actions = vec![];
        for transformation in PatchManager::get_instance().get_transformations(patch.id) {
            if (self.tiles & transformation.tiles) > 0 {
                continue;
            }

            let new_tiles = self.tiles | transformation.tiles;
            let action = Action::new(ActionPayload::PatchPlacement {
                patch,
                patch_index,
                patch_rotation: transformation.rotation_flag() as usize,
                patch_orientation: transformation.orientation_flag() as usize,
                row: transformation.row,
                column: transformation.column,
                starting_index,
                next_quilt_board: Some(new_tiles),
                previous_quilt_board: Some(self.tiles),
            });
            actions.push(action);
        }
        actions
    }

    /// Gets the valid actions for the given special patch.
    ///
    /// # Arguments
    ///
    /// * `special_patch` - The special patch to get the valid actions for.
    ///
    /// # Returns
    ///
    /// The valid actions for the given special patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles on the quilt board.
    pub fn get_valid_actions_for_special_patch(&self, special_patch: &Patch) -> Vec<Action> {
        let mut valid_actions = vec![];
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                if (self.tiles >> index) & 1 > 0 {
                    continue;
                }

                let action = Action::new(ActionPayload::SpecialPatchPlacement {
                    patch_id: special_patch.id,
                    row,
                    column,
                    next_quilt_board: Some(self.tiles | (1 << index)),
                    previous_quilt_board: Some(self.tiles),
                });
                valid_actions.push(action);
            }
        }
        valid_actions
    }
}

impl Display for QuiltBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                let tile = if (self.tiles >> index) & 1 > 0 { "█" } else { "░" };
                result.push_str(tile);
            }
            result.push('\n');
        }

        write!(f, "{}", result)?;
        write!(f, "Button income: {}", self.button_income)
    }
}
