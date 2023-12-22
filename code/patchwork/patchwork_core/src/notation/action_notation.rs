use const_format::concatcp;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{Action, ActionPayload, Notation, PatchManager, PatchworkError, QuiltBoard, TimeBoard};

lazy_static! {
    static ref ACTION_NOTATION_REGEX: Regex = Regex::new(r"^(?:(?P<null_action>N)|W(?P<start_index>\d+)|P(?P<p_patch_id>\d+)/(?P<p_index>\d+)/(?P<p_rotation>\d+)/(?P<p_orientation>\d+)/(?P<p_row>\d+)/(?P<p_column>\d+)/(?P<p_starting_index>\d+)|S(?P<s_patch_id>\d+)/(?P<s_row>\d+)/(?P<s_column>\d+))$").unwrap();
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
    ///     a. 'N' for a null action
    ///     b. 'W' for a walking action
    ///     c. 'P' for a normal patch placement action
    ///     d. 'S' for a special patch placement action
    /// 2. The payload of the action
    ///    a. For a null action this is empty
    ///    b. For a walking action this the starting index to walk from
    ///    c. For a normal patch placement action this is the patch id, the patch index, the patch rotation, the patch orientation, the row, the column and the starting index from where the player starts separated by slashes (e.g. P0/0/0/0/0/0)
    ///    d. For a special patch placement action this is the patch id, the row and the column separated by slashes (e.g. S0/0/0)
    ///
    /// # Example
    ///
    /// ```
    /// let null_action = "N";                         /* N */
    /// let walking_action = "W0";                     /* W starting_index */
    /// let patch_placement_action = "P0/0/0/0/0/0/0";   /* P patch_id / patch_index / rotation / orientation / row / column / starting_index */
    /// let special_patch_placement_action = "S0/0/0"; /* S patch_id / row / column */
    /// ```
    fn save_to_notation(&self) -> Result<String, PatchworkError> {
        Ok(match self.payload {
            ActionPayload::Null => "N".to_string(),
            ActionPayload::Walking { starting_index } => format!("W{:?}", starting_index),
            ActionPayload::PatchPlacement {
                starting_index,
                patch,
                patch_index,
                patch_rotation,
                patch_orientation,
                row,
                column,
                next_quilt_board: _,
                previous_quilt_board: _,
            } => format!(
                "P{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
                patch.id, patch_index, patch_rotation, patch_orientation, row, column, starting_index
            ),
            ActionPayload::SpecialPatchPlacement {
                patch_id,
                row,
                column,
                next_quilt_board: _,
                previous_quilt_board: _,
            } => format!("S{:?}/{:?}/{:?}", patch_id, row, column),
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
                reason: "[Action][load_from_notation] Invalid action notation",
            })?;

        let null_action = captures.name("null_action");
        if null_action.is_some() {
            return Ok(Action::new(ActionPayload::Null));
        }

        let starting_index = captures.name("start_index");
        if let Some(index) = starting_index {
            let index: usize = index
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid walking steps for action",
                })?;

            return Ok(Action::walking(index));
        }

        let p_patch_id = captures.name("p_patch_id");
        if let Some(patch_id) = p_patch_id {
            let p_index = captures.name("p_index").unwrap();
            let p_rotation = captures.name("p_rotation").unwrap();
            let p_orientation = captures.name("p_orientation").unwrap();
            let p_row = captures.name("p_row").unwrap();
            let p_column = captures.name("p_column").unwrap();

            let patch_id: usize = patch_id
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid patch id for action",
                })?;
            let patch_index: usize = p_index
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid patch index for action",
                })?;
            let patch_rotation: usize =
                p_rotation
                    .as_str()
                    .parse()
                    .map_err(|_| PatchworkError::InvalidNotationError {
                        notation: notation.to_string(),
                        reason: "[Action][load_from_notation] Invalid patch rotation for action",
                    })?;
            let patch_orientation: usize =
                p_orientation
                    .as_str()
                    .parse()
                    .map_err(|_| PatchworkError::InvalidNotationError {
                        notation: notation.to_string(),
                        reason: "[Action][load_from_notation] Invalid patch orientation for action",
                    })?;
            let row: usize = p_row
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid row for action",
                })?;
            let column: usize = p_column
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid column for action",
                })?;
            let starting_index: usize = captures
                .name("p_starting_index")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid starting index for action",
                })?;

            if patch_id > PatchManager::NORMAL_PATCHES + PatchManager::STARTING_PATCHES {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action][load_from_notation] Patch id cannot exceed ",
                        PatchManager::STARTING_PATCHES + PatchManager::NORMAL_PATCHES,
                    ),
                });
            }

            if patch_index > 2 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Patch index cannot exceed 2",
                });
            }

            if patch_rotation > 0b011 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Patch rotation cannot exceed 3",
                });
            }

            if patch_orientation > 0b1 {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Patch orientation cannot exceed 1",
                });
            }

            if row > QuiltBoard::ROWS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!("[Action][load_from_notation] Row cannot exceed ", QuiltBoard::ROWS),
                });
            }

            if column > QuiltBoard::COLUMNS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action][load_from_notation] Column cannot exceed ",
                        QuiltBoard::COLUMNS
                    ),
                });
            }

            if starting_index > TimeBoard::MAX_POSITION {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action][load_from_notation] Starting index cannot exceed ",
                        TimeBoard::MAX_POSITION
                    ),
                });
            }

            return Ok(Action::new(ActionPayload::PatchPlacement {
                starting_index,
                patch: &PatchManager::get_instance().patches[patch_id],
                patch_index,
                patch_rotation,
                patch_orientation,
                row,
                column,
                // The next and previous quilt board are saved as optimization and cannot be restored from the notation
                next_quilt_board: None,
                previous_quilt_board: None,
            }));
        }

        let s_patch_id = captures.name("s_patch_id");
        if let Some(patch_id) = s_patch_id {
            let s_row = captures.name("s_row").unwrap();
            let s_column = captures.name("s_column").unwrap();

            let patch_id: usize = patch_id
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid patch id for action",
                })?;
            let row: usize = s_row
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid row for action",
                })?;
            let column: usize = s_column
                .as_str()
                .parse()
                .map_err(|_| PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: "[Action][load_from_notation] Invalid column for action",
                })?;

            if !(PatchManager::STARTING_PATCHES + PatchManager::NORMAL_PATCHES..PatchManager::AMOUNT_OF_PATCHES)
                .contains(&patch_id)
            {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),

                    reason: concatcp!(
                        "[Action][load_from_notation] Patch id has to be in range from ",
                        PatchManager::STARTING_PATCHES + PatchManager::NORMAL_PATCHES,
                        " (inclusive) to ",
                        PatchManager::AMOUNT_OF_PATCHES - 1,
                        " (inclusive)",
                    ),
                });
            }

            if row > QuiltBoard::ROWS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!("[Action][load_from_notation] Row cannot exceed ", QuiltBoard::ROWS),
                });
            }

            if column > QuiltBoard::COLUMNS {
                return Err(PatchworkError::InvalidNotationError {
                    notation: notation.to_string(),
                    reason: concatcp!(
                        "[Action][load_from_notation] Column cannot exceed ",
                        QuiltBoard::COLUMNS
                    ),
                });
            }

            return Ok(Action::new(ActionPayload::SpecialPatchPlacement {
                patch_id,
                row,
                column,
                // The next and previous quilt board are saved as optimization and cannot be restored from the notation
                next_quilt_board: None,
                previous_quilt_board: None,
            }));
        }

        Err(PatchworkError::InvalidNotationError {
            notation: notation.to_string(),
            reason: "[Action][load_from_notation] Invalid action notation",
        })
    }
}
