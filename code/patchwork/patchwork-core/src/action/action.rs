use std::{fmt::Display, hash::Hash};

use crate::{ActionId, NaturalActionId, PatchManager, PatchTransformation, QuiltBoard};

/// Represents an action that can be taken in the patchwork board game.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Action {
    /// The player is walking.
    Walking { starting_index: u8 },
    /// The player is placing a patch
    PatchPlacement {
        patch_id: u8,
        patch_index: u8,
        patch_transformation_index: u16,
        previous_player_was_1: bool,
    },
    /// The player is placing a special patch.
    SpecialPatchPlacement { quilt_board_index: u8 },
    /// The player is doing nothing (Phantom Move).
    ///
    /// This cannot occur in a normal game and is only useful for Game Engines
    /// that want to force a player switch like negamax.
    ///
    /// This action is created when a forced player switch occurs
    /// while the other player should have the turn.
    Phantom,
    /// The player is doing nothing ([Null Move](https://www.chessprogramming.org/Null_Move)).
    /// This cannot occur in a normal game and is only useful for Game Engines
    /// to indicate that this is an invalid action.
    Null,
}

impl Action {
    /// Creates a new action from the given surrogate action id.
    ///
    /// # Arguments
    ///
    /// * `surrogate_action_id` - The surrogate action id to create the action from.
    ///
    /// # Returns
    ///
    /// The action corresponding to the given surrogate action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[must_use]
    pub fn from_surrogate_action_id(surrogate_action_id: ActionId) -> Self {
        debug_assert!(
            ActionId::is_valid_action_id(surrogate_action_id.as_bits()),
            "[ActionId::from_surrogate_action_id] The given surrogate action id is invalid ({:064b})",
            surrogate_action_id.as_bits()
        );

        match surrogate_action_id.as_bits() {
            ActionId::PHANTOM_ACTION_ID => Self::Phantom,
            ActionId::NULL_ACTION_ID => Self::Null,
            _ => {
                if surrogate_action_id.is_walking() {
                    Self::Walking {
                        starting_index: surrogate_action_id.get_starting_index(),
                    }
                } else if surrogate_action_id.is_special_patch_placement() {
                    Self::SpecialPatchPlacement {
                        quilt_board_index: surrogate_action_id.get_quilt_board_index(),
                    }
                } else {
                    Self::PatchPlacement {
                        patch_id: surrogate_action_id.get_patch_id(),
                        patch_index: surrogate_action_id.get_patch_index(),
                        patch_transformation_index: surrogate_action_id.get_patch_transformation_index(),
                        previous_player_was_1: surrogate_action_id.get_previous_player_was_1(),
                    }
                }
            }
        }
    }

    /// Creates a new action from the given natural action id.
    ///
    /// # Arguments
    ///
    /// * `natural_action_id` - The natural action id to create the action from.
    ///
    /// # Returns
    ///
    /// The action corresponding to the given natural action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the given natural action id is walking or patch placement and does
    /// not contain hidden information. This will panic in debug mode.
    #[must_use]
    pub fn from_natural_action_id(natural_action_id: NaturalActionId) -> Self {
        debug_assert!(
            NaturalActionId::is_valid_natural_action_id(natural_action_id.as_bits()),
            "[Action::from_natural_action_id] The given natural action id is invalid ({:064b})",
            natural_action_id.as_bits()
        );

        match natural_action_id.as_bits() {
            NaturalActionId::WALKING_ACTION_ID => {
                debug_assert!(!natural_action_id.contains_hidden_information(),
                        "[Action::from_natural_action_id] The given natural action id does not contain hidden information ({:064b})",
                        natural_action_id.as_bits_with_hidden_information()
                    );

                let starting_index = natural_action_id.get_starting_index();

                Self::Walking { starting_index }
            }
            NaturalActionId::PHANTOM_ACTION_ID => Self::Phantom,
            NaturalActionId::NULL_ACTION_ID => Self::Null,
            _ => {
                if natural_action_id.is_special_patch_placement() {
                    Self::SpecialPatchPlacement {
                        quilt_board_index: natural_action_id.get_quilt_board_index(),
                    }
                } else {
                    debug_assert!(
                        !natural_action_id.is_patch_placement(),
                        "[Action::from_natural_action_id] The given natural action id is not a patch placement action ({:064b})",
                        natural_action_id.as_bits_with_hidden_information()
                    );

                    debug_assert!(
                        !natural_action_id.contains_hidden_information(),
                        "[Action::from_natural_action_id] The given natural action id does not contain hidden information ({:064b})",
                        natural_action_id.as_bits_with_hidden_information()
                    );

                    // patch placement
                    let patch_id = natural_action_id.get_patch_id();
                    let patch_index = natural_action_id.get_patch_index();
                    let patch_transformation_index = natural_action_id.get_patch_transformation_index();
                    let previous_player_was_1 = natural_action_id.get_previous_player_was_1();

                    Self::PatchPlacement {
                        patch_id,
                        patch_index,
                        patch_transformation_index,
                        previous_player_was_1,
                    }
                }
            }
        }
    }

    /// Gets the surrogate action id from this action
    ///
    /// # Returns
    ///
    /// The surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn to_surrogate_action_id(&self) -> ActionId {
        ActionId::from_action(self)
    }

    /// Gets the natural action id from this action
    ///
    /// # Returns
    ///
    /// The natural action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn to_natural_action_id(&self) -> NaturalActionId {
        NaturalActionId::from_action(self)
    }

    /// Whether this action is a walking action.
    ///
    /// # Returns
    ///
    /// Whether this action is a walking action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_walking(&self) -> bool {
        matches!(self, Self::Walking { .. })
    }

    /// Whether this action is a special patch placement action.
    ///
    /// # Returns
    ///
    /// Whether this action is a special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_special_patch_placement(&self) -> bool {
        matches!(self, Self::SpecialPatchPlacement { .. })
    }

    /// Whether this action is a normal patch placement action.
    ///
    /// # Returns
    ///
    /// Whether this action is a normal patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_patch_placement(&self) -> bool {
        matches!(self, Self::PatchPlacement { .. })
    }

    /// Whether this action is a phantom action.
    ///
    /// # Returns
    ///
    /// Whether this action is a phantom action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_phantom(&self) -> bool {
        matches!(self, Self::Phantom)
    }

    /// Whether this action is a null action.
    ///
    /// # Returns
    ///
    /// Whether this action is a null action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Whether this action took the first patch.
    ///
    /// # Returns
    ///
    /// Whether this action took the first patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_first_patch_taken(&self) -> bool {
        match self {
            Self::PatchPlacement { patch_index, .. } => *patch_index == 0,
            _ => false,
        }
    }

    /// Whether this action took the second patch.
    ///
    /// # Returns
    ///
    /// Whether this action took the second patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_second_patch_taken(&self) -> bool {
        match self {
            Self::PatchPlacement { patch_index, .. } => *patch_index == 1,
            _ => false,
        }
    }

    /// Whether this action took the third patch.
    ///
    /// # Returns
    ///
    /// Whether this action took the third patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_third_patch_taken(&self) -> bool {
        match self {
            Self::PatchPlacement { patch_index, .. } => *patch_index == 2,
            _ => false,
        }
    }

    /// Tries to get the starting index of the walking action. If the action is
    /// not a walking action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(starting_index)` - If the action is a walking action.
    /// * `None` - If the action is not a walking action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn try_get_starting_index(&self) -> Option<u8> {
        match self {
            Self::Walking { starting_index } => Some(*starting_index),
            _ => None,
        }
    }

    /// Tries to get the patch id of the patch to be placed. If the action is
    /// not a patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(patch_id)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn try_get_patch_id(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement { patch_id, .. } => Some(*patch_id),
            _ => None,
        }
    }

    /// Tries to get the patch index of the action. If the action is not a
    /// patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(patch_index)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn try_get_patch_index(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement { patch_index, .. } => Some(*patch_index),
            _ => None,
        }
    }

    /// Tries to get the quilt board index of the action. If the action is
    /// not a patch placement or special patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(quilt_board_index)` - If the action is a patch placement or special patch placement action.
    /// * `None` - If the action is not a patch placement or special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn try_get_quilt_board_index(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement {
                patch_id,
                patch_transformation_index,
                ..
            } => {
                let PatchTransformation { row, column, .. } =
                    PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                Some(QuiltBoard::get_index(*row, *column))
            }
            Self::SpecialPatchPlacement { quilt_board_index } => Some(*quilt_board_index),
            _ => None,
        }
    }

    /// Tries to get the row of the action. If the action is not a
    /// patch placement or special patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(row)` - If the action is a patch placement or special patch placement action.
    /// * `None` - If the action is not a patch placement or special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn try_get_row(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement {
                patch_id,
                patch_transformation_index,
                ..
            } => {
                let PatchTransformation { row, .. } =
                    PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                Some(*row)
            }
            Self::SpecialPatchPlacement { quilt_board_index } => Some(QuiltBoard::get_row_column(*quilt_board_index).0),
            _ => None,
        }
    }

    /// Tries to get the column of the action. If the action is not a
    /// patch placement or special patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(column)` - If the action is a patch placement or special patch placement action.
    /// * `None` - If the action is not a patch placement or special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn try_get_column(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement {
                patch_id,
                patch_transformation_index,
                ..
            } => {
                let PatchTransformation { column, .. } =
                    PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                Some(*column)
            }
            Self::SpecialPatchPlacement { quilt_board_index } => Some(QuiltBoard::get_row_column(*quilt_board_index).1),
            _ => None,
        }
    }

    /// Tries to get the rotation of the patch to be placed
    ///
    /// # Returns
    ///
    /// * `Some(rotation)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn try_get_rotation(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement {
                patch_id,
                patch_transformation_index,
                ..
            } => {
                let transformation = PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                Some(transformation.rotation_flag())
            }
            _ => None,
        }
    }

    /// Tries to get the orientation of the patch to be placed
    ///
    /// # Returns
    ///
    /// * `Some(orientation)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn try_get_orientation(&self) -> Option<u8> {
        match self {
            Self::PatchPlacement {
                patch_id,
                patch_transformation_index,
                ..
            } => {
                let transformation = PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                Some(transformation.orientation_flag())
            }
            _ => None,
        }
    }

    /// Tries to get the patch transformation index of the action. If the action is
    /// not a patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(patch_transformation_index)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn try_get_patch_transformation_index(&self) -> Option<u16> {
        match self {
            Self::PatchPlacement {
                patch_transformation_index,
                ..
            } => Some(*patch_transformation_index),
            _ => None,
        }
    }

    /// Tries to get whether the previous player was 1. If the action is
    /// not a patch placement action this will return None
    ///
    /// # Returns
    ///
    /// * `Some(previous_player_was_1)` - If the action is a patch placement action.
    /// * `None` - If the action is not a patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn try_get_previous_player_was_1(&self) -> Option<bool> {
        match self {
            Self::PatchPlacement {
                previous_player_was_1, ..
            } => Some(*previous_player_was_1),
            _ => None,
        }
    }
}

impl From<ActionId> for Action {
    fn from(action_id: ActionId) -> Self {
        Self::from_surrogate_action_id(action_id)
    }
}

impl From<NaturalActionId> for Action {
    fn from(natural_action_id: NaturalActionId) -> Self {
        Self::from_natural_action_id(natural_action_id)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let natural_action = self.to_natural_action_id();

        write!(f, "Action {}", natural_action.as_bits())?;

        match self {
            Self::Walking { starting_index } => {
                write!(f, " - Walking (starting at {starting_index})")
            }
            Self::SpecialPatchPlacement { quilt_board_index } => {
                let (row, column) = QuiltBoard::get_row_column(*quilt_board_index);
                write!(f, " - Special patch placement at ({row}, {column})")
            }
            Self::PatchPlacement {
                patch_id,
                patch_index,
                patch_transformation_index,
                previous_player_was_1: _,
            } => {
                let transformation = PatchManager::get_transformation(*patch_id, *patch_transformation_index);
                let row = transformation.row;
                let column = transformation.column;
                let rotation = transformation.rotation();
                let orientation = if transformation.flipped() { "flipped" } else { "normal" };

                write!(
                    f,
                    " - Patch({}) placement (index {}) at ({}, {}) with (R {}°, O {})",
                    *patch_id, *patch_index, row, column, rotation, orientation
                )
            }
            Self::Phantom => {
                write!(f, " - Phantom")
            }
            Self::Null => {
                write!(f, " - Null")
            }
        }
    }
}
