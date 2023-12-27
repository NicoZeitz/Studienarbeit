use std::fmt::Display;

use crate::{Action, ActionPayload, Patch, PatchManager};

// The quilt board of the player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuiltBoard {
    /// The tiles of the quilt board.
    pub tiles: u128,
    /// The amount of buttons this board generates.
    pub button_income: isize,
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
    /// The amount of buttons a full board generates.
    pub const FULL_BOARD_BUTTON_INCOME: isize = 7;

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
    pub fn score(&self) -> isize {
        -2 * (QuiltBoard::TILES as u32 - self.tiles_filled()) as isize
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

    /// Gets the tile at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the tile.
    ///
    /// # Returns
    ///
    /// The tile at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_index(&self, index: usize) -> bool {
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
                self.button_income += patch.button_income as isize;

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

    /// Undoes the given action to the quilt board.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to undo.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)` when the `previous_quilt_board` is given inside the action payload or the action is a special patch placement.
    ///
    /// `𝒪(𝑛)` otherwise where `𝑛` is the number of transformations for the given patch.
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
                        panic!("Invalid action! The next quilt board is not the same as the current one!");
                    }
                }

                self.button_income -= patch.button_income as isize;

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

    /// Flips the tiles of the quilt board horizontally and then rotates them.
    ///
    /// # Arguments
    ///
    /// * `tiles` - The tiles to flip and rotate.
    /// * `rotate` - The amount of times to rotate the tiles. One of 0, 1, 2 or 3.
    /// * `flip` - Whether to flip the tiles horizontally.
    ///
    /// # Returns
    ///
    /// The tiles of the quilt board flipped horizontally and then rotated.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the amount of tiles on the quilt board.
    pub fn flip_horizontally_then_rotate_tiles(tiles: u128, rotate: usize, flip: bool) -> u128 {
        let tiles = if flip {
            QuiltBoard::flip_tiles_horizontally(tiles)
        } else {
            tiles
        };
        QuiltBoard::rotate_tiles(tiles, rotate)
    }

    /// Rotates the tiles of the given quilt board.
    ///
    /// # Arguments
    ///
    /// * `tiles` - The tiles to rotate.
    /// * `rotation` - The amount of times to rotate the tiles. One of 0, 1, 2 or 3.
    ///
    /// # Returns
    ///
    /// The tiles of the quilt board rotated.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the amount of tiles on the quilt board.
    ///
    /// # Examples
    ///
    /// ```txt
    /// ███░░░░░░
    /// █░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// Rotation 0  (0°)
    /// ███░░░░░░
    /// █░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// Rotation 1 (90°)
    /// ░░░░░░░██
    /// ░░░░░░░░█
    /// ░░░░░░░░█
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// Rotation 2 (180°)
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░█
    /// ░░░░░░███
    /// Rotation 3 (270°)
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// ░░░░░░░░░
    /// █░░░░░░░░
    /// █░░░░░░░░
    /// ██░░░░░░░
    /// ```
    pub fn rotate_tiles(tiles: u128, rotation: usize) -> u128 {
        let mut result = 0;
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                let tile = (tiles >> index) & 1 > 0;
                let new_index = match rotation {
                    0 => index,
                    1 => (column * QuiltBoard::ROWS) + (QuiltBoard::ROWS - row - 1),
                    2 => (QuiltBoard::TILES - 1) - index,
                    3 => (QuiltBoard::TILES - 1) - ((column * QuiltBoard::ROWS) + (QuiltBoard::ROWS - row - 1)),
                    _ => unreachable!(),
                };
                result |= (tile as u128) << new_index;
            }
        }
        result
    }

    /// Flips the tiles of the given quilt board horizontally.
    ///
    /// # Arguments
    ///
    /// * `tiles` - The tiles to flip.
    ///
    /// # Returns
    ///
    /// The tiles of the quilt board flipped horizontally.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the amount of tiles on the quilt board.
    ///
    /// # Example
    ///
    /// ```txt
    /// ███░░░░░░    ░░░░░░███
    /// █░░░░░░░░    ░░░░░░░░█
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░ -> ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ```
    pub fn flip_tiles_horizontally(tiles: u128) -> u128 {
        let mut result = 0;
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                let tile = (tiles >> index) & 1 > 0;
                let new_index = (row * QuiltBoard::COLUMNS) + (QuiltBoard::COLUMNS - column - 1);
                result |= (tile as u128) << new_index;
            }
        }
        result
    }

    /// Flips the tiles of the quilt board vertically.
    ///
    /// # Arguments
    ///
    /// * `tiles` - The tiles to flip.
    ///
    /// # Returns
    ///
    /// The tiles of the quilt board flipped vertically.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the amount of tiles on the quilt board.
    ///
    /// # Example
    ///
    /// ```txt
    /// ███░░░░░░    ░░░░░░░░░
    /// █░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░ -> ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    ░░░░░░░░░
    /// ░░░░░░░░░    █░░░░░░░░
    /// ░░░░░░░░░    ███░░░░░░
    /// ```
    pub fn flip_tiles_vertically(tiles: u128) -> u128 {
        let mut result = 0;
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                let tile = (tiles >> index) & 1 > 0;
                let new_index = ((QuiltBoard::ROWS - row - 1) * QuiltBoard::COLUMNS) + column;
                result |= (tile as u128) << new_index;
            }
        }
        result
    }

    /// Rotates and flips a row and column.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to rotate and flip.
    /// * `column` - The column to rotate and flip.
    /// * `rotation` - The amount of 90° rotations to apply.
    /// * `flip` - Whether to flip the row and column horizontally before rotating.
    ///
    /// # Returns
    ///
    /// * `(usize, usize)` - The rotated and flipped row and column.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[rustfmt::skip]
    pub fn flip_horizontally_then_rotate_row_and_column(row: usize, column: usize, rotation: usize, flip: bool) -> (usize, usize) {
        debug_assert!(rotation < 4, "[QuiltBoard][flip_horizontally_then_rotate_row_and_column] Invalid rotation: {}", rotation);

        match (rotation, flip) {
            //  ░░░ Identity
            // ░░█░    Row: 0,  Column: 0
            // ░░░░   █Row: 1, █Column: 2
            (0, false) => (row,                              column),
            // ░░
            // ░░░  Rotation 90°
            // ░█░     Row: 0,  Column: 2
            // ░░░    █Row: 2, █Column: 1
            (1, false) => (column,                           QuiltBoard::ROWS    - 1 - row),
            // ░░░░ Rotation 180°
            // ░█░░    Row: 2,  Column: 3
            // ░░░    █Row: 1, █Column: 1
            (2, false) => (QuiltBoard::ROWS    - 1 - row,    QuiltBoard::COLUMNS - 1 - column),
            // ░░░
            // ░█░  Rotation 270°
            // ░░░     Row: 3,  Column: 0
            //  ░░    █Row: 1, █Column: 1
            (3, false) => (QuiltBoard::COLUMNS - 1 - column, row),
            // ░░░  Horizontal Flip
            // ░█░░    Row: 0,  Column: 3
            // ░░░░   █Row: 1, █Column: 1
            (0, true)  => (row,                              QuiltBoard::COLUMNS - 1 - column),
            // ░░░
            // ░█░  Horizontal Flip then Rotation 90°
            // ░░░     Row: 3,  Column: 2
            // ░░     █Row: 1, █Column: 1
            (1, true)  => (QuiltBoard::COLUMNS - 1 - column, QuiltBoard::ROWS    - 1 - row),
            // ░░░░ Horizontal Flip then Rotation 180°
            // ░░█░    Row: 2,  Column: 0
            //  ░░░   █Row: 1, █Column: 2
            (2, true)  => (QuiltBoard::ROWS    - 1 - row,    column),
            //  ░░
            // ░░░  Horizontal Flip then Rotation 270°
            // ░█░     Row: 0,  Column: 0
            // ░░░    █Row: 2, █Column: 1
            (3, true)  => (column,                           row),
            _ => unreachable!("[QuiltBoard][flip_horizontally_then_rotate_row_and_column] Invalid rotation."),
        }
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
