use std::fmt::Display;

use crate::{
    Action, ActionPatchPlacementPayload, ActionPayload, ActionSpecialPatchPlacementPayload, Patch,
    PatchManager,
};

// The quilt board of the player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuiltBoard {
    /// The tiles of the quilt board.
    tiles: u128,
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
    pub const ROWS: u32 = 9;
    /// The amount of columns on the quilt board
    pub const COLUMNS: u32 = 9;
    /// The amount of tiles on the quilt board
    pub const TILES: u32 = QuiltBoard::ROWS * QuiltBoard::COLUMNS;

    /// Creates a new empty quilt board.
    pub fn new() -> QuiltBoard {
        QuiltBoard {
            tiles: 0,
            button_income: 0,
        }
    }

    /// Whether the board is full.
    pub fn is_full(&self) -> bool {
        self.tiles.count_ones() == QuiltBoard::TILES
    }

    /// The amount of tiles that are filled.
    pub fn tiles_filled(&self) -> u32 {
        self.tiles.count_ones()
    }

    /// The percentage of tiles that are filled.
    pub fn percent_full(&self) -> f32 {
        self.tiles.count_ones() as f32 / QuiltBoard::TILES as f32
    }

    /// The score the player has with this quilt board.
    ///
    /// The score is calculated by taking the amount of tiles that are not filled and multiplying it by -2.
    pub fn score(&self) -> i32 {
        -2 * (QuiltBoard::TILES - self.tiles_filled()) as i32
    }

    /// Updates the quilt board with the next quilt board.
    ///
    /// # Arguments
    ///
    /// * `next_quilt_board` - The next quilt board.
    pub fn update(&mut self, next_quilt_board: u128, button_income: i32) {
        self.button_income += button_income;
        self.tiles = next_quilt_board;
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
    pub fn get_valid_actions_for_patch(
        &self,
        patch: &'static Patch,
        patch_index: usize,
    ) -> Vec<Action> {
        // BEGIN: SIMD
        // rust nightly needed and slower with this implementation
        // #![feature(portable_simd)]
        // use std::simd::u64x64;
        // // windows of size 32
        // let lower_bits = (self.tiles & u64::MAX as u128) as u64;
        // let upper_bits = (self.tiles >> 64) as u64;

        // let board_tiles = u64x64::from_slice(&[upper_bits, lower_bits].repeat(32));

        // for transformations in PatchManager::get_instance()
        //     .get_transformations(patch.id)
        //     .chunks(32)
        // {
        //     let mut tiles = transformations
        //         .iter()
        //         .flat_map(|t| {
        //             let lower_bits = (t.tiles & u64::MAX as u128) as u64;
        //             let upper_bits = (t.tiles >> 64) as u64;
        //             [upper_bits, lower_bits]
        //         })
        //         .collect::<Vec<_>>();
        //     if tiles.len() != 64 {
        //         tiles.append(&mut vec![0; 64 - tiles.len()]);
        //     }

        //     let chunk_tiles = u64x64::from_slice(&tiles);

        //     let new_tiles = board_tiles & chunk_tiles;
        //     let tiles_array = new_tiles.as_array();

        //     for (i, transformation) in transformations.iter().enumerate() {
        //         let index = i * 2;
        //         let new_upper_tiles = tiles_array[index];
        //         let new_lower_tiles = tiles_array[index + 1];

        //         if new_upper_tiles == 0 && new_lower_tiles == 0 {
        //             let new_tiles = self.tiles | transformation.tiles;
        //             let action = Action::new(ActionPayload::PatchPlacement {
        //                 payload: ActionPatchPlacementPayload {
        //                     patch,
        //                     patch_index,
        //                     patch_rotation: transformation.rotation_flag() as usize,
        //                     patch_orientation: transformation.orientation_flag() as usize,
        //                     row: transformation.row,
        //                     column: transformation.column,
        //                     next_quilt_board: new_tiles,
        //                 },
        //             });
        //             actions.push(action);
        //         }
        //     }
        // }
        // END: SIMD

        // // BEGIN: ITER
        // also slower
        // PatchManager::get_instance()
        //     .get_transformations(patch.id)
        //     .iter()
        //     .filter_map(|transformation| {
        //         if (self.tiles & transformation.tiles) > 0 {
        //             return None;
        //         }

        //         let new_tiles = self.tiles | transformation.tiles;
        //         let action = Action::new(ActionPayload::PatchPlacement {
        //             payload: ActionPatchPlacementPayload {
        //                 patch,
        //                 patch_index,
        //                 patch_rotation: transformation.rotation_flag() as usize,
        //                 patch_orientation: transformation.orientation_flag() as usize,
        //                 row: transformation.row,
        //                 column: transformation.column,
        //                 next_quilt_board: new_tiles,
        //             },
        //         });
        //         Some(action)
        //     })
        //     .collect()
        // // END: ITER

        let mut actions = vec![];
        for transformation in PatchManager::get_instance().get_transformations(patch.id) {
            if (self.tiles & transformation.tiles) > 0 {
                continue;
            }

            let new_tiles = self.tiles | transformation.tiles;
            let action = Action::new(ActionPayload::PatchPlacement {
                payload: ActionPatchPlacementPayload {
                    patch,
                    patch_index,
                    patch_rotation: transformation.rotation_flag() as usize,
                    patch_orientation: transformation.orientation_flag() as usize,
                    row: transformation.row,
                    column: transformation.column,
                    next_quilt_board: new_tiles,
                },
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
    pub fn get_valid_actions_for_special_patch(&self, special_patch: &Patch) -> Vec<Action> {
        let mut valid_actions = vec![];
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;
                if (self.tiles >> index) & 1 > 0 {
                    continue;
                }

                let action = Action::new(ActionPayload::SpecialPatchPlacement {
                    payload: ActionSpecialPatchPlacementPayload {
                        patch_id: special_patch.id,
                        row: row as usize,
                        column: column as usize,
                        next_quilt_board: self.tiles | (1 << index),
                    },
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
                let tile = if (self.tiles >> index) & 1 > 0 {
                    "█"
                } else {
                    "░"
                };
                result.push_str(tile);
            }
            result.push('\n');
        }

        write!(f, "{}", result)?;
        write!(f, "Button income: {}", self.button_income)
    }
}
