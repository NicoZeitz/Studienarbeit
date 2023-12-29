use const_format::concatcp;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    Action, ActionId, NaturalActionId, Notation, PatchManager, PatchTransformation, PatchworkError, QuiltBoard,
};

lazy_static! {
    static ref ACTION_NOTATION_REGEX: Regex = Regex::new(r"^(?:W(?P<w_starting_index>\d+)|(?P<phantom_action>_)|(?P<null_action>N)|P(?P<p_patch_id>\d+)I(?P<p_index>\d+)═(?P<p_row>\d+)‖(?P<p_column>\d+)↻(?P<p_rotation>\d+)↔(?P<p_orientation>\d+)P(?P<p_previous>[01])|S═(?P<s_row>\d+)‖(?P<s_column>\d+))$").unwrap();
}

impl Notation for ActionId {
    fn save_to_notation(&self) -> Result<String, PatchworkError> {
        self.to_action().save_to_notation()
    }

    fn load_from_notation(notation: &str) -> Result<Self, PatchworkError> {
        let action = Action::load_from_notation(notation)?;
        Ok(ActionId::from_action(&action))
    }
}

impl Notation for NaturalActionId {
    fn save_to_notation(&self) -> Result<String, PatchworkError> {
        self.try_to_action()
            .ok_or(PatchworkError::InvalidNotationError {
                reason: "[NaturalActionId::save_to_notation] Cannot convert this natural action id to notation",
                notation: format!("{:?}", self),
            })?
            .save_to_notation()
    }

    fn load_from_notation(notation: &str) -> Result<Self, PatchworkError> {
        let action = Action::load_from_notation(notation)?;
        Ok(NaturalActionId::from_action(&action))
    }
}

impl Notation for Action {
    /// Saves an action as a string.
    /// The state can be loaded again with `load_from_notation`.
    ///
    /// # Returns
    ///
    /// The action as a string.
    ///
    /// # State Representation
    ///
    /// The state consists of
    /// 1. A character representing the type of action
    ///     a. 'W' for a walking action
    ///     c. 'P' for a normal patch placement action
    ///     d. 'S' for a special patch placement action
    ///     a. '_' for a phantom action
    ///     a. 'N' for a null action
    /// 2. The payload of the action
    ///    a. For a walking action this is the starting index (e.g. W0)
    ///    b. For a normal patch placement action this is the patch id, the patch index, the row, the column, the patch rotation, the patch orientation and if the previous player was player 1 (e.g. P0I0═0‖0↻0↔0P0)
    ///    c. For a special patch placement action this is the patch id, the row and the column (e.g. S0═0‖0)
    ///    d. For a phantom action this is empty (e.g. _)
    ///    e. For a null action this is empty (e.g. N)
    ///
    /// # Example
    ///
    /// ```
    /// let walking_action = Action::load_from_notation("W0");                     /* W0 */
    /// let patch_placement_action = Action::load_from_notation("P0I0═0‖0↻0↔0");   /* P patch_id I patch_index ═ row ‖ column ↻ rotation ↔ orientation */
    /// let special_patch_placement_action = Action::load_from_notation("S═0‖0");  /* S ═ row ‖ column */
    /// let phantom_action = Action::load_from_notation("N");                      /* N */
    /// let null_action = Action::load_from_notation("_");                         /* _ */
    ///
    /// let walking_action_notation = walking_action.save_to_notation().unwrap();
    /// let patch_placement_action_notation = patch_placement_action.save_to_notation().unwrap();
    /// let special_patch_placement_action_notation = special_patch_placement_action.save_to_notation().unwrap();
    /// let phantom_action_notation = phantom_action.save_to_notation().unwrap();
    /// let null_action_notation = null_action.save_to_notation().unwrap();
    /// ```
    fn save_to_notation(&self) -> Result<String, PatchworkError> {
        Ok(match self {
            Action::Walking { starting_index } => format!("W{}", starting_index),
            Action::PatchPlacement {
                patch_id,
                patch_index,
                patch_transformation_index,
                previous_player_was_1,
            } => {
                let transformation = PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                let row = transformation.row;
                let column = transformation.column;
                let rotation = transformation.rotation_flag();
                let orientation = transformation.orientation_flag();

                format!(
                    "P{:?}I{:?}═{:?}‖{:?}↻{:?}↔{:?}P{:?}",
                    patch_id, patch_index, row, column, rotation, orientation, *previous_player_was_1 as u8
                )
            }
            Action::SpecialPatchPlacement { quilt_board_index } => {
                let (row, column) = QuiltBoard::get_row_column(*quilt_board_index);
                format!("S═{:?}‖{:?}", row, column)
            }
            Action::Phantom => "_".to_string(),
            Action::Null => "N".to_string(),
        })
    }

    /// Loads an action from a string.
    ///
    /// # Arguments
    ///
    /// * `notation` - The notation to load the action from.
    ///
    /// # Returns
    ///
    /// The action or an error if the notation is invalid.
    fn load_from_notation(notation: &str) -> Result<Self, PatchworkError> {
        let captures = ACTION_NOTATION_REGEX
            .captures(notation)
            .ok_or(PatchworkError::InvalidNotationError {
                notation: notation.to_string(),
                reason: "[Action::load_from_notation] Invalid action notation",
            })?;

        if let Some(w_starting_index) = captures.name("w_starting_index") {
            let starting_index =
                w_starting_index
                    .as_str()
                    .parse()
                    .map_err(|_| PatchworkError::InvalidNotationError {
                        notation: notation.to_string(),
                        reason: "[Action::load_from_notation] Invalid starting index for action",
                    })?;
            return Ok(Action::Walking { starting_index });
        }
        if captures.name("null_action").is_some() {
            return Ok(Action::Null);
        }
        if captures.name("phantom_action").is_some() {
            return Ok(Action::Phantom);
        }

        if let Some(patch_id) = captures.name("p_patch_id") {
            let patch_id: u8 = patch_id
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch id for action",
                })?;
            let patch_index: u8 = captures
                .name("p_index")
                .expect("p_index should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch index for action",
                })?;
            let row: u8 = captures
                .name("p_row")
                .expect("p_row should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid row for action",
                })?;
            let column: u8 = captures
                .name("p_column")
                .expect("p_column should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid column for action",
                })?;
            let rotation: u8 = captures
                .name("p_rotation")
                .expect("p_rotation should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch rotation for action",
                })?;
            let orientation: u8 = captures
                .name("p_orientation")
                .expect("p_orientation should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch orientation for action",
                })?;
            let previous_player_was_1: u8 = captures
                .name("p_previous")
                .expect("p_previous should be present")
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid previous player for action",
                })?;

            if patch_id > PatchManager::AMOUNT_OF_NON_STARTING_PATCHES + PatchManager::AMOUNT_OF_STARTING_PATCHES {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action::load_from_notation] Patch id cannot exceed ",
                        PatchManager::AMOUNT_OF_STARTING_PATCHES + PatchManager::AMOUNT_OF_NON_STARTING_PATCHES,
                    ),
                });
            }

            if patch_index > 2 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Patch index cannot exceed 2",
                });
            }

            if rotation > 0b011 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Patch rotation cannot exceed 3",
                });
            }

            if orientation > 0b1 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Patch orientation cannot exceed 1",
                });
            }

            if row > QuiltBoard::ROWS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!("[Action::load_from_notation] Row cannot exceed ", QuiltBoard::ROWS),
                });
            }

            if column > QuiltBoard::COLUMNS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action::load_from_notation] Column cannot exceed ",
                        QuiltBoard::COLUMNS
                    ),
                });
            }

            // transform to tiling to also match patch transformation that are not saved because of redundancy
            let tiling = get_tiling_from_transformation(patch_id, row, column, rotation, orientation);

            let patch_transformation_index = PatchManager::get_transformations(patch_id)
                .iter()
                .position(|transformation| transformation.tiles == tiling)
                .ok_or(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch transformation (row, column, rotation and orientation combination) for action",
                })? as u16;

            return Ok(Action::PatchPlacement {
                patch_id,
                patch_index,
                patch_transformation_index,
                previous_player_was_1: previous_player_was_1 == 1,
            });
        }

        if let Some(patch_id) = captures.name("s_patch_id") {
            let s_row = captures.name("s_row").unwrap();
            let s_column = captures.name("s_column").unwrap();

            let patch_id: u8 = patch_id
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid patch id for action",
                })?;
            let row: u8 = s_row
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid row for action",
                })?;
            let column: u8 = s_column
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action::load_from_notation] Invalid column for action",
                })?;

            if !(PatchManager::AMOUNT_OF_STARTING_PATCHES + PatchManager::AMOUNT_OF_NON_STARTING_PATCHES
                ..PatchManager::AMOUNT_OF_PATCHES)
                .contains(&patch_id)
            {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),

                    reason: concatcp!(
                        "[Action::load_from_notation] Patch id has to be in range from ",
                        PatchManager::AMOUNT_OF_STARTING_PATCHES + PatchManager::AMOUNT_OF_NON_STARTING_PATCHES,
                        " (inclusive) to ",
                        PatchManager::AMOUNT_OF_PATCHES - 1,
                        " (inclusive)",
                    ),
                });
            }

            if row > QuiltBoard::ROWS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!("[Action::load_from_notation] Row cannot exceed ", QuiltBoard::ROWS),
                });
            }

            if column > QuiltBoard::COLUMNS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action::load_from_notation] Column cannot exceed ",
                        QuiltBoard::COLUMNS
                    ),
                });
            }

            return Ok(Action::SpecialPatchPlacement {
                quilt_board_index: QuiltBoard::get_index(row, column),
            });
        }

        Err(PatchworkError::InvalidNotationError {
            notation: notation.to_string(),
            reason: "[Action::load_from_notation] Invalid action notation",
        })
    }
}

/// Code partially copied from code/patchwork/patchwork_macros/src/lib.rs
fn get_tiling_from_transformation(patch_id: u8, row: u8, column: u8, rotation: u8, orientation: u8) -> u128 {
    let tiles = &PatchManager::get_instance().tiles[patch_id as usize];
    let row = row as usize;
    let column = column as usize;

    let transformed_tiles = get_transformed_tiles(tiles, rotation | (orientation << 2));

    let mut tiling = 0u128;
    for (i, tile_row) in transformed_tiles.iter().enumerate() {
        for (j, tile) in tile_row.iter().enumerate() {
            if *tile == 1 {
                tiling |= 1u128 << ((row + i) * QuiltBoard::COLUMNS as usize + (column + j));
            }
        }
    }
    tiling
}

/// Copied from code/patchwork/patchwork_macros/src/lib.rs
fn get_transformed_tiles(tiles: &Vec<Vec<u8>>, transformation: u8) -> Vec<Vec<u8>> {
    match transformation {
        PatchTransformation::ROTATION_0 => tiles.clone(),
        PatchTransformation::ROTATION_90 => {
            let mut new_tiles = vec![vec![0; tiles.len()]; tiles[0].len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[j][tiles.len() - i - 1] = *tile;
                }
            }
            new_tiles
        }
        PatchTransformation::ROTATION_180 => {
            let mut new_tiles = vec![vec![0; tiles[0].len()]; tiles.len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tiles.len() - i - 1][tile_row.len() - j - 1] = *tile;
                }
            }
            new_tiles
        }
        PatchTransformation::ROTATION_270 => {
            let mut new_tiles = vec![vec![0; tiles.len()]; tiles[0].len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tile_row.len() - j - 1][i] = *tile;
                }
            }
            new_tiles
        }
        PatchTransformation::FLIPPED => {
            let mut new_tiles = tiles.clone();
            new_tiles.reverse();
            new_tiles
        }
        PatchTransformation::FLIPPED_ROTATION_90 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles.len()]; flipped_tiles[0].len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[j][flipped_tiles.len() - i - 1] = *tile;
                }
            }
            new_tiles
        }
        PatchTransformation::FLIPPED_ROTATION_180 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles[0].len()]; flipped_tiles.len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[flipped_tiles.len() - i - 1][tile_row.len() - j - 1] = *tile;
                }
            }
            new_tiles
        }
        PatchTransformation::FLIPPED_ROTATION_270 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles.len()]; flipped_tiles[0].len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tile_row.len() - j - 1][i] = *tile;
                }
            }
            new_tiles
        }
        _ => tiles.clone(),
    }
}
