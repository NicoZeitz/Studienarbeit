use std::{fmt::Display, hash::Hash};

use derivative::Derivative;

use crate::{Patch, QuiltBoard};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionPayload {
    /// The player is doing nothing. (null move)
    Null,
    /// The player is walking.
    Walking { starting_index: usize },
    /// The player is placing a patch.
    PatchPlacement {
        patch: &'static Patch,
        patch_index: usize,
        patch_rotation: usize,
        patch_orientation: usize,
        row: usize,
        column: usize,
        starting_index: usize,
        next_quilt_board: Option<u128>,
        previous_quilt_board: Option<u128>,
    },
    /// The player is placing a special patch.
    SpecialPatchPlacement {
        patch_id: usize,
        row: usize,
        column: usize,
        next_quilt_board: Option<u128>,
        previous_quilt_board: Option<u128>,
    },
}

/// Represents an action that can be taken in the patchwork board game.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub struct Action {
    //  The id of the action. This is a number between 0 and 2025 (both inclusive).
    pub id: usize,
    /// The patch that is being placed or Walking if the player is walking.
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub payload: ActionPayload,
}

impl Action {
    /// The amount of available actions for the game of patchwork. The actually allowed actions are way lower than this number, but we need to be able to represent all the possible actions in a single number. This is the maximum amount of actions that can be taken in a single turn.
    ///
    /// (MAX_PATCH_INDEX(2) * ROWS(9) + MAX_ROW(8)) * COLUMNS(9) + MAX_COLUMN(8)) * ROTATIONS(4) + MAX_ROTATION(3)) * ORIENTATIONS(2) + MAX_ORIENTATION(1) + ACTIONS_OTHER_THAN_NORMAL_PATCH_PLACEMENT_ACTION(83)
    pub const AMOUNT_OF_ACTIONS: u32 = 2026;

    /// Creates a new null `Action`.
    #[inline]
    pub fn null() -> Action {
        Action {
            id: usize::MAX,
            payload: ActionPayload::Null,
        }
    }

    // Creates a new walking `Action`.
    #[inline]
    pub fn walking(starting_index: usize) -> Action {
        Action {
            id: 0,
            payload: ActionPayload::Walking { starting_index },
        }
    }

    #[inline]
    pub fn new(payload: ActionPayload) -> Action {
        Action {
            id: Action::calculate_id(&payload),
            payload,
        }
    }

    /// Whether this action is a null move.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.id == usize::MAX
    }

    /// Whether this action is a walking action.
    #[inline]
    pub fn is_walking(&self) -> bool {
        self.id == 0
    }

    /// Whether this action is a special patch placement action.
    #[inline]
    pub fn is_special_patch_placement(&self) -> bool {
        matches!(self.payload, ActionPayload::SpecialPatchPlacement { .. })
    }

    /// Whether this action is a normal patch placement action.
    #[inline]
    pub fn is_patch_placement(&self) -> bool {
        matches!(self.payload, ActionPayload::PatchPlacement { .. })
    }

    /// Whether this action took the first patch.
    #[inline]
    pub fn is_first_patch_taken(&self) -> bool {
        match &self.payload {
            ActionPayload::PatchPlacement { patch_index, .. } => *patch_index == 0,
            _ => false,
        }
    }

    /// Whether this action took the second patch.
    #[inline]
    pub fn is_second_patch_taken(&self) -> bool {
        match &self.payload {
            ActionPayload::PatchPlacement { patch_index, .. } => *patch_index == 1,
            _ => false,
        }
    }

    /// Whether this action took the third patch.
    #[inline]
    pub fn is_third_patch_taken(&self) -> bool {
        match &self.payload {
            ActionPayload::PatchPlacement { patch_index, .. } => *patch_index == 2,
            _ => false,
        }
    }

    #[rustfmt::skip]
    fn calculate_id(payload: &ActionPayload) -> usize {
        const ROWS: usize = QuiltBoard::ROWS;
        const COLUMNS: usize = QuiltBoard::COLUMNS;

        match payload {
            // null action [usize::MAX, usize::MAX]
            ActionPayload::Null => {
                usize::MAX
            }
            // walking action [0, 0]
            ActionPayload::Walking { .. } => {
                0
            }
            // special patch placement action [1, 81]
            ActionPayload::SpecialPatchPlacement { row, column, .. } => {
                const OFFSET: usize = 1;
                *row * COLUMNS + *column + OFFSET
            }
            // the maximum amount of placement for a patch is actually 448. The patch is:
            // ▉
            // ▉▉▉
            // but as we want to be able to represent all the information in a single number, we need to use [(((index * ROWS + row) * COLUMNS + column) * ROTATIONS + rotation) * ORIENTATIONS + orientation + OFFSET] as limit for the action
            ActionPayload::PatchPlacement { patch_index, row, column, patch_rotation, patch_orientation, .. } => {
                const OFFSET: usize = 82;
                const ROTATIONS: usize = 4;
                const ORIENTATIONS: usize = 2;

                *patch_index * ROWS * COLUMNS * ROTATIONS * ORIENTATIONS +
                       *row         * COLUMNS * ROTATIONS * ORIENTATIONS +
                       *column                * ROTATIONS * ORIENTATIONS +
                       *patch_rotation                    * ORIENTATIONS +
                       *patch_orientation +
                       OFFSET
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action {}", self.id)?;

        match &self.payload {
            ActionPayload::Null => {
                write!(f, " - NULL")
            }
            ActionPayload::Walking { starting_index } => {
                write!(f, " - Walking from index {}", starting_index)
            }
            ActionPayload::SpecialPatchPlacement {
                patch_id, row, column, ..
            } => {
                write!(
                    f,
                    " - Special patch({}) placement at ({}, {})",
                    *patch_id, *row, *column
                )
            }
            ActionPayload::PatchPlacement {
                patch,
                patch_index,
                row,
                column,
                patch_rotation,
                patch_orientation,
                ..
            } => {
                write!(
                    f,
                    " - Patch({}) placement (index {}) at ({}, {}) with (R {}, O {})",
                    patch.id, *patch_index, *row, *column, *patch_rotation, *patch_orientation
                )
            }
        }
    }
}
