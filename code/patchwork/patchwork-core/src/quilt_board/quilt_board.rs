use std::fmt::Display;

use crate::{ActionId, Patch, PatchManager};

// The quilt board of the player.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct QuiltBoard {
    /// The tiles of the quilt board.
    pub tiles: u128,
    /// The amount of buttons this board generates.
    pub button_income: u8,
}

impl Default for QuiltBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl QuiltBoard {
    /// The amount of rows on the quilt board
    pub const ROWS: u8 = 9;
    /// The amount of columns on the quilt board
    pub const COLUMNS: u8 = 9;
    /// The amount of tiles on the quilt board
    pub const TILES: u8 = Self::ROWS * Self::COLUMNS;
    /// The amount of buttons a 7x7 board generates.
    pub const BOARD_EXTRA_BUTTON_INCOME: i32 = 7;

    // ─────────────────────────────────────────────── UTILITY FUNCTIONS ───────────────────────────────────────────────

    /// Converts the given row and column to an index.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to convert.
    /// * `column` - The column to convert.
    ///
    /// # Returns
    ///
    /// The index of the given row and column.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn get_index(row: u8, column: u8) -> u8 {
        row * Self::COLUMNS + column
    }

    /// Converts the given index to a row and column.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to convert.
    ///
    /// # Returns
    ///
    /// The row and column of the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn get_row_column(index: u8) -> (u8, u8) {
        (index / Self::COLUMNS, index % Self::COLUMNS)
    }

    /// Creates a new [`QuiltBoard`] which is empty.
    ///
    /// # Returns
    ///
    /// A new [`QuiltBoard`] which is empty.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
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
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.tiles.count_ones() == Self::TILES as u32
    }

    /// Whether the board has a special tile condition.
    ///
    /// A special tile condition is when at least a 7x7 square is filled with patches.
    ///
    /// # Returns
    ///
    /// Whether the board has a special tile condition.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[rustfmt::skip]
    #[must_use]
    pub const fn is_special_tile_condition_reached(&self) -> bool {
        const BOARD_1X1: u128 = 0b0_0001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_u128;
        const BOARD_1X2: u128 = 0b0_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_u128;
        const BOARD_1X3: u128 = 0b0_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_u128;
        const BOARD_2X1: u128 = 0b00_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_0000_0000_u128;
        const BOARD_2X2: u128 = 0b00_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_0000_0000_u128;
        const BOARD_2X3: u128 = 0b00_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1000_0000_0000_u128;
        const BOARD_3X1: u128 = 0b000_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_0000_0000_0000_0000_u128;
        const BOARD_3X2: u128 = 0b000_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0011_1111_1000_0000_0000_0000_0000_u128;
        const BOARD_3X3: u128 = 0b001_1111_1100_1111_1110_0111_1111_0011_1111_1001_1111_1100_1111_1110_0111_1111_0000_0000_0000_0000_0000_u128;

        (self.tiles & BOARD_1X1) == BOARD_1X1 ||
        (self.tiles & BOARD_1X2) == BOARD_1X2 ||
        (self.tiles & BOARD_1X3) == BOARD_1X3 ||
        (self.tiles & BOARD_2X1) == BOARD_2X1 ||
        (self.tiles & BOARD_2X2) == BOARD_2X2 ||
        (self.tiles & BOARD_2X3) == BOARD_2X3 ||
        (self.tiles & BOARD_3X1) == BOARD_3X1 ||
        (self.tiles & BOARD_3X2) == BOARD_3X2 ||
        (self.tiles & BOARD_3X3) == BOARD_3X3
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
    #[inline]
    #[must_use]
    pub const fn tiles_filled(&self) -> u32 {
        self.tiles.count_ones()
    }

    /// The amount of tiles that are not filled.
    ///
    /// # Returns
    ///
    /// The amount of tiles that are not filled.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn tiles_free(&self) -> u32 {
        Self::TILES as u32 - self.tiles_filled()
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
    #[inline]
    #[must_use]
    pub fn percent_full(&self) -> f32 {
        self.tiles.count_ones() as f32 / f32::from(Self::TILES)
    }

    /// The score the player has with this quilt board.
    ///
    /// The score is calculated by taking the amount of tiles that are not
    /// filled and multiplying it by -2.
    ///
    /// # Returns
    ///
    /// The score the player has with this quilt board.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn score(&self) -> i32 {
        -2 * (self.tiles_free() as i32)
    }

    // ──────────────────────────────────────────────────── GETTERS ────────────────────────────────────────────────────

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
    #[inline]
    #[must_use]
    pub const fn get(&self, row: u8, column: u8) -> bool {
        let index = Self::get_index(row, column);
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
    #[inline]
    #[must_use]
    pub const fn get_at(&self, index: u8) -> bool {
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
    #[inline]
    #[must_use]
    pub const fn get_row(&self, row: u8) -> u16 {
        let start = row * Self::COLUMNS;
        let end = start + Self::COLUMNS;
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
    #[inline]
    #[must_use]
    pub fn get_column(&self, column: u8) -> u16 {
        let mut result = 0;
        for row in 0..Self::ROWS {
            let index = Self::get_index(row, column);
            result |= (self.tiles >> index) & 1 << row;
        }
        result as u16
    }

    // ────────────────────────────────────────────── DO AND UNDO ACTION ───────────────────────────────────────────────

    /// Applies the given action to the quilt board.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to apply.
    ///
    /// # Panics
    ///
    /// When the given action cannot be done because it is not a patch placement or because the tiles that are
    /// affected are already filled.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn do_action(&mut self, action: ActionId) {
        if action.is_patch_placement() {
            let patch_id = action.get_patch_id();
            let patch_transformation_index = action.get_patch_transformation_index();
            let patch = PatchManager::get_patch(patch_id);
            let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);

            self.button_income += patch.button_income;
            self.tiles |= transformation.tiles;
        } else if action.is_special_patch_placement() {
            let index = action.get_quilt_board_index();

            #[cfg(debug_assertions)]
            if (self.tiles >> index) & 1 > 0 {
                let (row, column) = Self::get_row_column(index);
                panic!(
                    "[QuiltBoard::do_action] Invalid action! The tile at row {row} and column {column} is already filled!"
                );
            }

            self.tiles |= 1 << index;
        }
    }

    /// Undoes the given action to the quilt board.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to undo.
    ///
    /// # Panics
    ///
    /// When the given action cannot be undone because it is not a patch placement or because the tiles that are
    /// affected are already empty.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn undo_action(&mut self, action: ActionId) {
        if action.is_patch_placement() {
            let patch_id = action.get_patch_id();
            let patch_transformation_index = action.get_patch_transformation_index();
            let patch = PatchManager::get_patch(patch_id);
            let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);

            self.button_income -= patch.button_income;
            self.tiles &= !transformation.tiles;
        } else if action.is_special_patch_placement() {
            let index = action.get_quilt_board_index();

            #[cfg(debug_assertions)]
            if (self.tiles >> index) & 1 == 0 {
                let (row, column) = Self::get_row_column(index);
                panic!(
                    "[QuiltBoard::undo_action] Invalid action! The tile at row {row} and column {column} is not filled!"
                );
            }

            self.tiles &= !(1 << index);
        }
    }

    // ─────────────────────────────────────────────── GET VALID ACTIONS ───────────────────────────────────────────────

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
    #[must_use]
    pub fn get_valid_actions_for_patch(
        &self,
        patch: &'static Patch,
        patch_index: u8,
        is_player_1: bool,
    ) -> Vec<ActionId> {
        let mut actions = vec![];
        for (patch_transformation_index, transformation) in
            PatchManager::get_transformations(patch.id).iter().enumerate()
        {
            if (self.tiles & transformation.tiles) > 0 {
                continue;
            }

            let action =
                ActionId::patch_placement(patch.id, patch_index, patch_transformation_index as u16, is_player_1);
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
    #[must_use]
    pub fn get_valid_actions_for_special_patch(&self) -> Vec<ActionId> {
        let mut valid_actions: Vec<ActionId> = vec![];
        for index in 0..Self::TILES {
            if (self.tiles >> index) & 1 > 0 {
                continue;
            }

            let action_id = ActionId::special_patch_placement(index);
            valid_actions.push(action_id);
        }
        valid_actions
    }

    // ─────────────────────────────────────────── ROTATE AND FLIP UTILITIES ───────────────────────────────────────────

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
    #[inline]
    #[must_use]
    pub fn flip_horizontally_then_rotate_tiles(tiles: u128, rotate: u8, flip: bool) -> u128 {
        let tiles = if flip {
            Self::flip_tiles_horizontally(tiles)
        } else {
            tiles
        };
        Self::rotate_tiles(tiles, rotate)
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
    /// Rotation 0  (0°)   Rotation 1 (90°)   Rotation 2 (180°)   Rotation 3 (270°)
    ///    ███░░░░░░          ░░░░░░░██          ░░░░░░░░░           ░░░░░░░░░
    ///    █░░░░░░░░          ░░░░░░░░█          ░░░░░░░░░           ░░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░█          ░░░░░░░░░           ░░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░░░░           ░░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░░░░           ░░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░░░░           ░░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░░░░           █░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░░░█           █░░░░░░░░
    ///    ░░░░░░░░░          ░░░░░░░░░          ░░░░░░███           ██░░░░░░░
    /// ```
    #[must_use]
    pub fn rotate_tiles(tiles: u128, rotation: u8) -> u128 {
        let mut result = 0;
        for (row, column) in itertools::iproduct!(0..Self::ROWS, 0..Self::COLUMNS) {
            let index = Self::get_index(row, column);
            let tile = (tiles >> index) & 1 > 0;
            let new_index = match rotation {
                0 => index,
                1 => (column * Self::ROWS) + (Self::ROWS - row - 1),
                2 => (Self::TILES - 1) - index,
                3 => (Self::TILES - 1) - ((column * Self::ROWS) + (Self::ROWS - row - 1)),
                _ => unreachable!("[QuiltBoard::rotate_tiles] Invalid rotation: {}.", rotation),
            };
            result |= u128::from(tile) << new_index;
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
    /// ███░░░░░░   ░░░░░░███
    /// █░░░░░░░░   ░░░░░░░░█
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░ → ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ```
    #[must_use]
    pub fn flip_tiles_horizontally(tiles: u128) -> u128 {
        let mut result = 0;
        for (row, column) in itertools::iproduct!(0..Self::ROWS, 0..Self::COLUMNS) {
            let index = Self::get_index(row, column);
            let tile = (tiles >> index) & 1 > 0;
            let new_index = (row * Self::COLUMNS) + (Self::COLUMNS - column - 1);
            result |= u128::from(tile) << new_index;
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
    /// ███░░░░░░   ░░░░░░░░░
    /// █░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░ → ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   ░░░░░░░░░
    /// ░░░░░░░░░   █░░░░░░░░
    /// ░░░░░░░░░   ███░░░░░░
    /// ```
    #[must_use]
    pub fn flip_tiles_vertically(tiles: u128) -> u128 {
        let mut result = 0;
        for (row, column) in itertools::iproduct!(0..Self::ROWS, 0..Self::COLUMNS) {
            let index = Self::get_index(row, column);
            let tile = (tiles >> index) & 1 > 0;
            let new_index = ((Self::ROWS - row - 1) * Self::COLUMNS) + column;
            result |= u128::from(tile) << new_index;
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
    /// * `(u8, u8)` - The rotated and flipped row and column.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[rustfmt::skip]
    #[must_use]    pub fn flip_horizontally_then_rotate_row_and_column(row: u8, column: u8, rotation: u8, flip: bool) -> (u8, u8) {
        debug_assert!(rotation < 4, "[QuiltBoard::flip_horizontally_then_rotate_row_and_column] Invalid rotation: {rotation}");

        match (rotation, flip) {
            // ☐░░░ Identity
            // ░░█░   ☐ Row: 0, ☐ Column: 0
            // ░░░░   █ Row: 1, █ Column: 2
            (0, false) => (row,                              column),
            // ░░☐
            // ░░░  Rotation 90°
            // ░█░    ☐ Row: 0, ☐ Column: 2
            // ░░░    █ Row: 2, █ Column: 1
            (1, false) => (column,                           Self::ROWS    - 1 - row),
            // ░░░░ Rotation 180°
            // ░█░░   ☐ Row: 2, ☐ Column: 3
            // ░░░☐   █ Row: 1, █ Column: 1
            (2, false) => (Self::ROWS    - 1 - row,    Self::COLUMNS - 1 - column),
            // ░░░
            // ░█░  Rotation 270°
            // ░░░    ☐ Row: 3, ☐ Column: 0
            // ☐░░    █ Row: 1, █ Column: 1
            (3, false) => (Self::COLUMNS - 1 - column, row),
            // ░░░☐ Horizontal Flip
            // ░█░░   ☐ Row: 0, ☐ Column: 3
            // ░░░░   █ Row: 1, █ Column: 1
            (0, true)  => (row,                              Self::COLUMNS - 1 - column),
            // ░░░
            // ░█░  Horizontal Flip then Rotation 90°
            // ░░░    ☐ Row: 3, ☐ Column: 2
            // ░░☐    █ Row: 1, █ Column: 1
            (1, true)  => (Self::COLUMNS - 1 - column, Self::ROWS    - 1 - row),
            // ░░░░ Horizontal Flip then Rotation 180°
            // ░░█░   ☐ Row: 2, ☐ Column: 0
            // ☐░░░   █ Row: 1, █ Column: 2
            (2, true)  => (Self::ROWS    - 1 - row,    column),
            // ☐░░
            // ░░░  Horizontal Flip then Rotation 270°
            // ░█░    ☐ Row: 0, ☐ Column: 0
            // ░░░    █ Row: 2, █ Column: 1
            (3, true)  => (column,                           row),
            _ => unreachable!("[QuiltBoard::flip_horizontally_then_rotate_row_and_column] Invalid rotation."),
        }
    }
}

impl Display for QuiltBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for row in 0..Self::ROWS {
            for column in 0..Self::COLUMNS {
                let index = row * Self::COLUMNS + column;
                let tile = if (self.tiles >> index) & 1 > 0 { "█" } else { "░" };
                result.push_str(tile);
            }
            result.push('\n');
        }

        write!(f, "{result}")?;
        write!(f, "Button income: {}", self.button_income)
    }
}
