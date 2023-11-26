use std::{fmt::Display, hash::Hash};

use derivative::Derivative;

use crate::{Patch, QuiltBoard};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionPatchPlacementPayload {
    pub patch: &'static Patch,
    pub patch_index: usize,
    pub patch_rotation: usize,
    pub patch_orientation: usize,
    pub row: usize,
    pub column: usize,
    pub next_quilt_board: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionSpecialPatchPlacementPayload {
    pub patch_id: usize,
    pub row: usize,
    pub column: usize,
    pub next_quilt_board: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionPayload {
    /// The player is walking.
    Walking,
    /// The player is placing a patch.
    PatchPlacement {
        payload: ActionPatchPlacementPayload,
    },
    /// The player is placing a special patch.
    SpecialPatchPlacement {
        payload: ActionSpecialPatchPlacementPayload,
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

    // Creates a new walking `Action`.
    pub fn walking() -> Action {
        Action {
            id: 0,
            payload: ActionPayload::Walking,
        }
    }

    pub fn new(payload: ActionPayload) -> Action {
        Action {
            id: Action::calculate_id(&payload),
            payload,
        }
    }

    /// Whether this action is a walking action.
    #[inline]
    pub fn is_walking(&self) -> bool {
        matches!(self.payload, ActionPayload::Walking)
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
            ActionPayload::Walking => false,
            ActionPayload::PatchPlacement { payload } => payload.patch_index == 0,
            ActionPayload::SpecialPatchPlacement { .. } => false,
        }
    }

    /// Whether this action took the second patch.
    #[inline]
    pub fn is_second_patch_taken(&self) -> bool {
        match &self.payload {
            ActionPayload::Walking => false,
            ActionPayload::PatchPlacement { payload } => payload.patch_index == 1,
            ActionPayload::SpecialPatchPlacement { .. } => false,
        }
    }

    /// Whether this action took the third patch.
    #[inline]
    pub fn is_third_patch_taken(&self) -> bool {
        match &self.payload {
            ActionPayload::Walking => false,
            ActionPayload::PatchPlacement { payload } => payload.patch_index == 2,
            ActionPayload::SpecialPatchPlacement { .. } => false,
        }
    }

    #[rustfmt::skip]
    fn calculate_id(payload: &ActionPayload) -> usize {
        const ROWS: usize = QuiltBoard::ROWS;
        const COLUMNS: usize = QuiltBoard::COLUMNS;

        match payload {
            // walking action [0, 0]
            ActionPayload::Walking => {
                0
            }
            // special patch placement action [1, 81]
            ActionPayload::SpecialPatchPlacement { payload } => {
                const OFFSET: usize = 1;
                payload.row * COLUMNS + payload.column + OFFSET
            }
            // the maximum amount of placement for a patch is actually 448. The patch is:
            // ▉
            // ▉▉▉
            // but as we want to be able to represent all the information in a single number, we need to use [(((index * ROWS + row) * COLUMNS + column) * ROTATIONS + rotation) * ORIENTATIONS + orientation + OFFSET] as limit for the action
            ActionPayload::PatchPlacement { payload } => {
                const OFFSET: usize = 82;
                const ROTATIONS: usize = 4;
                const ORIENTATIONS: usize = 2;

                payload.patch_index * ROWS * COLUMNS * ROTATIONS * ORIENTATIONS +
                       payload.row                * COLUMNS * ROTATIONS * ORIENTATIONS +
                       payload.column                       * ROTATIONS * ORIENTATIONS +
                       payload.patch_rotation                           * ORIENTATIONS +
                       payload.patch_orientation +
                       OFFSET
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action {}", self.id)?;

        match &self.payload {
            ActionPayload::Walking => {
                write!(f, " - Walking")
            }
            ActionPayload::SpecialPatchPlacement { payload } => {
                write!(
                    f,
                    " - Special patch({}) placement at ({}, {})",
                    payload.patch_id, payload.row, payload.column
                )
            }
            ActionPayload::PatchPlacement { payload } => {
                write!(
                    f,
                    " - Patch({}) placement (index {}) at ({}, {}) with (R {}, O {})",
                    payload.patch.id,
                    payload.patch_index,
                    payload.row,
                    payload.column,
                    payload.patch_rotation,
                    payload.patch_orientation
                )
            }
        }
    }
}
