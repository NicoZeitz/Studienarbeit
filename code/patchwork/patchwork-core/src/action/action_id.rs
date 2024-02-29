use std::fmt::Display;

use crate::{Action, NaturalActionId, PatchManager, PatchTransformation, QuiltBoard};

/// The action id is a surrogate unique number that is used to identify an action.
/// It is used to represent an action in a more compact way than the action itself.
///
/// The action id is used as a fast and compact representation of all actions
/// in the game implementation to make moves
///
/// The id's are distributed as follows:
/// - \[0,  52]: Walking action
///     - Containing the starting index.
/// - \[53, 133]: Special patch placement actions.
///     - Containing the quilt board index.
/// - \[134, 88837]: Patch placement actions.
///    - Containing the patch id.
///    - Containing the patch index in the list of available patches
///      (always 0,1 or 2).
///    - Containing the index in the transformations of that patch in
///      patch transformations of the patch manager.
///    - Containing a flag if the previous player was player 1.
/// - \[88838, 88838]: Phantom action.
/// - \[88839, 88839]: Null action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ActionId(u32);

impl ActionId {
    /// The starting id of the walking actions.
    pub const WALKING_ACTION_ID_START: u32 = 0;
    /// The ending id of the walking actions.
    pub const WALKING_ACTION_ID_END: u32 = 52;
    /// The starting id of the special patch placement actions.
    pub const SPECIAL_PATCH_PLACEMENT_ID_START: u32 = 53;
    /// The ending id of the special patch placement actions.
    pub const SPECIAL_PATCH_PLACEMENT_ID_END: u32 = 133;
    /// The starting id of the patch placement actions.
    pub const PATCH_PLACEMENT_ID_START: u32 = 134;
    /// The ending id of the patch placement actions.
    pub const PATCH_PLACEMENT_ID_END: u32 = 88837;
    /// The id of the phantom action.
    pub const PHANTOM_ACTION_ID: u32 = 88838;
    /// The id of a null action.
    pub const NULL_ACTION_ID: u32 = 88839;

    /// The amount of available surrogate action ids for the game of patchwork.
    ///
    /// The actually allowed actions are way lower than this number.
    ///
    /// The actually best it is 88837, phantom action have id 88838 and
    /// null actions have id 88839
    pub const AMOUNT_OF_SURROGATE_ACTION_IDS: u32 = 88840;

    /// Creates a walking surrogate action id
    ///
    /// # Arguments
    ///
    /// * `starting_index` - The starting index of the walking action.
    ///
    /// # Returns
    ///
    /// The walking surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn walking(starting_index: u8) -> Self {
        Self(Self::WALKING_ACTION_ID_START + starting_index as u32)
    }

    /// Creates a patch placement surrogate action id
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The patch id of the patch placement action
    /// * `patch_index` - The patch index of the patch placement action
    /// * `patch_transformation_index` - The patch transformation index of the patch placement action
    ///
    /// # Returns
    ///
    /// The patch placement surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn patch_placement(
        patch_id: u8,
        patch_index: u8,
        patch_transformation_index: u16,
        previous_player_was_1: bool,
    ) -> Self {
        Self(transform_patch_placement_to_surrogate_id(
            patch_id,
            patch_index,
            patch_transformation_index,
            previous_player_was_1,
        ))
    }

    /// Creates a special patch placement surrogate action id
    ///
    /// # Arguments
    ///
    /// * `quilt_board_index` - The quilt board index of the special patch placement action
    ///
    /// # Returns
    ///
    /// The special patch placement surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn special_patch_placement(quilt_board_index: u8) -> Self {
        Self(quilt_board_index as u32 + Self::SPECIAL_PATCH_PLACEMENT_ID_START)
    }

    /// Creates a phantom surrogate action id
    ///
    /// # Returns
    ///
    /// The phantom surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn phantom() -> Self {
        Self(Self::PHANTOM_ACTION_ID)
    }

    /// Creates a null surrogate action id
    ///
    /// # Returns
    ///
    /// The null surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(Self::NULL_ACTION_ID)
    }

    /// Returns if the given id is a valid action id.
    ///
    /// # Arguments
    ///
    /// * `action_id` - The given id to check.
    ///
    /// # Returns
    ///
    /// If the given id is a valid action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_valid_action_id(action_id: u32) -> bool {
        action_id <= Self::NULL_ACTION_ID
    }

    /// Gets the surrogate action id as a u16
    ///
    /// # Returns
    ///
    /// The surrogate action id as a u16
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn as_bits(&self) -> u32 {
        self.0
    }

    /// Creates a new action id from the given bits.
    ///
    /// # Arguments
    ///
    /// * `bits` - The bits to create the action id from.
    ///
    /// # Returns
    ///
    /// The action id corresponding to the given bits.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the given bits do not represent a valid action id.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        debug_assert!(
            Self::is_valid_action_id(bits),
            "[ActionId::from_bits] The given bits do not represent a valid action id."
        );
        Self(bits)
    }

    /// Creates a new action id from the given action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to create the action id from.
    ///
    /// # Returns
    ///
    /// The action id corresponding to the given action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[must_use]
    pub const fn from_action(action: &Action) -> Self {
        Self(match action {
            Action::Walking { starting_index } => Self::WALKING_ACTION_ID_START + *starting_index as u32,
            Action::SpecialPatchPlacement { quilt_board_index } => {
                (*quilt_board_index as u32) + Self::SPECIAL_PATCH_PLACEMENT_ID_START
            }
            Action::PatchPlacement {
                patch_id,
                patch_index,
                patch_transformation_index,
                previous_player_was_1,
            } => transform_patch_placement_to_surrogate_id(
                *patch_id,
                *patch_index,
                *patch_transformation_index,
                *previous_player_was_1,
            ),
            Action::Phantom => Self::PHANTOM_ACTION_ID,
            Action::Null => Self::NULL_ACTION_ID,
        })
    }

    /// Creates a new surrogate action id from the given natural action id.
    ///
    /// # Arguments
    ///
    /// * `natural_action_id` - The natural action id to create the surrogate action id from.
    ///
    /// # Returns
    ///
    /// The surrogate action id corresponding to the given natural action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the given natural action id is a walking action or a patch placement
    /// action and does not contain hidden information.
    /// This will panic in debug mode.
    ///
    /// # Panics
    ///
    /// If the given natural action id is not a valid natural action id and the
    /// program is run in debug mode otherwise this will cause undefined behavior.

    #[must_use]
    pub fn from_natural_action_id(natural_action_id: NaturalActionId) -> Self {
        debug_assert!(
            NaturalActionId::is_valid_natural_action_id(natural_action_id.as_bits()),
            "[ActionId::from_natural_action_id] The given natural action id is not a valid natural action id."
        );

        let masked_natural_action_id = natural_action_id.as_bits();
        Self(match masked_natural_action_id {
            NaturalActionId::WALKING_ACTION_ID => {
                debug_assert!(
                    !natural_action_id.contains_hidden_information(),
                    "[ActionId::from_natural_action_id] The given natural action id does not contain hidden information ({:064b})",
                    natural_action_id.as_bits_with_hidden_information()
                );

                Self::WALKING_ACTION_ID_START + u32::from(natural_action_id.get_starting_index())
            }
            NaturalActionId::SPECIAL_PATCH_PLACEMENT_ID_START..=NaturalActionId::SPECIAL_PATCH_PLACEMENT_ID_END => {
                masked_natural_action_id as u32 - NaturalActionId::SPECIAL_PATCH_PLACEMENT_ID_START as u32
                    + Self::SPECIAL_PATCH_PLACEMENT_ID_START
            }
            NaturalActionId::PATCH_PLACEMENT_ID_START..=NaturalActionId::PATCH_PLACEMENT_ID_END => {
                debug_assert!(
                    !natural_action_id.contains_hidden_information(),
                    "[ActionId::from_natural_action_id] The given natural action id does not contain hidden information ({:064b})",
                    natural_action_id.as_bits_with_hidden_information()
                );

                let patch_id = natural_action_id.get_patch_id();
                let patch_index = natural_action_id.get_patch_index();
                let patch_transformation_index = natural_action_id.get_patch_transformation_index();
                let previous_player_was_1 = natural_action_id.get_previous_player_was_1();

                transform_patch_placement_to_surrogate_id(
                    patch_id,
                    patch_index,
                    patch_transformation_index,
                    previous_player_was_1,
                )
            }
            NaturalActionId::PHANTOM_ACTION_ID => Self::PHANTOM_ACTION_ID,
            NaturalActionId::NULL_ACTION_ID => Self::NULL_ACTION_ID,
            _ => unreachable!("[ActionId::from_natural_action_id] Invalid natural action id."),
        })
    }

    /// Creates the corresponding action from the action id.
    ///
    /// # Returns
    ///
    /// The corresponding action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn to_action(&self) -> Action {
        Action::from_surrogate_action_id(*self)
    }

    /// Creates the corresponding natural action id from the action id.
    ///
    /// # Returns
    ///
    /// The corresponding natural action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn to_natural_action_id(&self) -> NaturalActionId {
        NaturalActionId::from_surrogate_action_id(*self)
    }

    /// Returns if the action is a walking action.
    ///
    /// # Returns
    ///
    /// If the action is a walking action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_walking(&self) -> bool {
        self.0 <= Self::WALKING_ACTION_ID_END
    }

    /// Returns if the action is a special patch placement action.
    ///
    /// # Returns
    ///
    /// If the action is a special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_special_patch_placement(&self) -> bool {
        self.0 >= Self::SPECIAL_PATCH_PLACEMENT_ID_START && self.0 <= Self::SPECIAL_PATCH_PLACEMENT_ID_END
    }

    /// Returns if the action is a normal patch placement action.
    ///
    /// # Returns
    ///
    /// If the action is a normal patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_patch_placement(&self) -> bool {
        self.0 >= Self::PATCH_PLACEMENT_ID_START && self.0 <= Self::PATCH_PLACEMENT_ID_END
    }

    /// Returns if the action is a phantom action.
    ///
    /// # Returns
    ///
    /// If the action is a phantom action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_phantom(&self) -> bool {
        self.0 == Self::PHANTOM_ACTION_ID
    }

    /// Returns if the action is a null action.
    ///
    /// # Returns
    ///
    /// If the action is a null action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.0 == Self::NULL_ACTION_ID
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
        self.is_patch_placement() && self.get_patch_index() == 0
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
        self.is_patch_placement() && self.get_patch_index() == 1
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
        self.is_patch_placement() && self.get_patch_index() == 2
    }

    /// Returns the starting index of the action.
    /// Only available for walking actions.
    ///
    /// # Returns
    ///
    /// The starting index of the action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a walking action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]

    pub const fn get_starting_index(&self) -> u8 {
        debug_assert!(
            self.is_walking(),
            "[ActionId::get_starting_index] Action id is not a walking action."
        );
        (self.0 - Self::WALKING_ACTION_ID_START) as u8
    }

    /// Returns the patch id of the action.
    /// Only available for normal patch placement actions.
    ///
    /// # Returns
    ///
    /// The patch id of the action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a normal patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]

    pub const fn get_patch_id(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_patch_id] Action id is not a patch placement action."
        );
        ((self.0 - Self::PATCH_PLACEMENT_ID_START) / PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS
            % PatchManager::AMOUNT_OF_NORMAL_PATCHES as u32) as u8
    }

    /// Returns the patch index of the action.
    /// Only available for normal patch placement actions.
    ///
    /// # Returns
    ///
    /// The patch index of the action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a normal patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub const fn get_patch_index(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_patch_index] Action id is not a patch placement action."
        );
        ((self.0 - Self::PATCH_PLACEMENT_ID_START)
            / PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS
            / (PatchManager::AMOUNT_OF_NORMAL_PATCHES as u32)
            % PatchManager::MAX_AMOUNT_OF_CHOOSABLE_TILES) as u8
    }

    /// Returns the quilt board index of the action.
    /// Only available for patch placement and special patch placement actions.
    ///
    /// # Returns
    ///
    /// The quilt board index of the action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement or special patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]

    pub fn get_quilt_board_index(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement() || self.is_special_patch_placement(),
            "[ActionId::get_quilt_board_index] Action id is not a patch placement or special patch placement action."
        );

        if self.is_special_patch_placement() {
            return (self.0 - Self::SPECIAL_PATCH_PLACEMENT_ID_START) as u8;
        }

        let patch_id = self.get_patch_id();
        let patch_transformation_index = self.get_patch_transformation_index();

        let PatchTransformation { row, column, .. } =
            PatchManager::get_transformation(patch_id, patch_transformation_index);
        QuiltBoard::get_index(*row, *column)
    }

    /// Returns the row where the patch is to be placed
    /// Only available for patch placement and special patch placement actions.
    ///
    /// # Returns
    ///
    /// The row where the patch is to be placed.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement or special patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]

    pub fn get_row(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement() || self.is_special_patch_placement(),
            "[ActionId::get_row] Action id is not a patch placement or special patch placement action."
        );

        if self.is_special_patch_placement() {
            let (row, _) = QuiltBoard::get_row_column((self.0 - Self::SPECIAL_PATCH_PLACEMENT_ID_START) as u8);
            return row;
        }

        let patch_id = self.get_patch_id();
        let patch_transformation_index = self.get_patch_transformation_index();

        let PatchTransformation { row, .. } = PatchManager::get_transformation(patch_id, patch_transformation_index);
        *row
    }

    /// Returns the column where the patch is to be placed
    /// Only available for patch placement and special patch placement actions.
    ///
    /// # Returns
    ///
    /// The column where the patch is to be placed.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement or special patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub fn get_column(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement() || self.is_special_patch_placement(),
            "[ActionId::get_column] Action id is not a patch placement or special patch placement action."
        );

        if self.is_special_patch_placement() {
            let (_, column) = QuiltBoard::get_row_column((self.0 - Self::SPECIAL_PATCH_PLACEMENT_ID_START) as u8);
            return column;
        }

        let patch_id = self.get_patch_id();
        let patch_transformation_index = self.get_patch_transformation_index();

        let PatchTransformation { column, .. } = PatchManager::get_transformation(patch_id, patch_transformation_index);
        *column
    }

    /// Returns the rotation of the patch to be placed
    /// Only available for patch placement actions.
    ///
    /// # Returns
    ///
    /// The rotation of the patch to be placed.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub fn get_rotation(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_rotation] Action id is not a patch placement action."
        );

        let patch_id = self.get_patch_id();
        let patch_transformation_index = self.get_patch_transformation_index();

        let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);
        transformation.rotation_flag()
    }

    /// Returns the orientation of the patch to be placed
    /// Only available for patch placement actions.
    ///
    /// # Returns
    ///
    /// The orientation of the patch to be placed.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub fn get_orientation(&self) -> u8 {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_orientation] Action id is not a patch placement action."
        );

        let patch_id = self.get_patch_id();
        let patch_transformation_index = self.get_patch_transformation_index();

        let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);
        transformation.orientation_flag()
    }

    /// Returns the patch transformation index of the action.
    /// Only available for normal patch placement actions.
    ///
    /// # Returns
    ///
    /// The patch transformation index of the action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a normal patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub const fn get_patch_transformation_index(&self) -> u16 {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_patch_transformation_index] Action id is not a patch placement action."
        );
        ((self.0 - Self::PATCH_PLACEMENT_ID_START) % PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS) as u16
    }

    /// Returns if the previous player was player 1.
    /// Only available for normal patch placement actions.
    ///
    /// # Returns
    ///
    /// If the previous player was player 1.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a normal patch placement action.
    /// This will panic in debug mode.
    #[inline]
    #[must_use]
    pub const fn get_previous_player_was_1(&self) -> bool {
        debug_assert!(
            self.is_patch_placement(),
            "[ActionId::get_previous_player_was_1] Action id is not a patch placement action."
        );
        (self.0 - Self::PATCH_PLACEMENT_ID_START)
            / PatchManager::MAX_AMOUNT_OF_CHOOSABLE_TILES
            / PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS
            / PatchManager::AMOUNT_OF_NORMAL_PATCHES as u32
            % 2
            == 1
    }
}

impl From<Action> for ActionId {
    fn from(action: Action) -> Self {
        Self::from_action(&action)
    }
}

impl From<NaturalActionId> for ActionId {
    fn from(natural_action_id: NaturalActionId) -> Self {
        Self::from_natural_action_id(natural_action_id)
    }
}

impl Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_action().fmt(f)
    }
}

#[rustfmt::skip]
const fn transform_patch_placement_to_surrogate_id(patch_id: u8, patch_index: u8, patch_transformation_index: u16, previous_player_was_1: bool) -> u32 {
    previous_player_was_1 as u32 * PatchManager::MAX_AMOUNT_OF_CHOOSABLE_TILES * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as u32) * PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS +
        (patch_index as u32)                                                   * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as u32) * PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS +
        (patch_id as u32)                                                                                                        * PatchManager::MAX_AMOUNT_OF_TRANSFORMATIONS +
        (patch_transformation_index as u32) +
        ActionId::PATCH_PLACEMENT_ID_START
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionId, NaturalActionId};

    use pretty_assertions::assert_eq;

    #[allow(clippy::unreadable_literal)]
    fn as_bits_string(natural_action_id: NaturalActionId) -> String {
        let bits = natural_action_id.as_bits_with_hidden_information();
        format!(
            "{:08b}|{:032b}|{:01b}|{:039b}",
            bits >> 56,
            (bits >> 40) & 0xFFFF,
            (bits >> 41) & 0x1,
            bits & 0x7FFFFFFFFFFF
        )
    }

    // ─────────────────────────────────────────────── TO ACTION AND BACK ───────────────────────────────────────────────

    #[test]
    pub fn convert_to_action_and_back_walking() {
        let action = Action::Walking { starting_index: 13 };

        let action_id = ActionId::from_action(&action);

        assert!(action_id.is_walking(), "Action id is not a walking action.");

        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!(
            "Surrogate Action Id - Starting Index: {}",
            action_id.get_starting_index()
        );

        assert_eq!(
            action,
            action_id.to_action(),
            "Surrogate Action Id does not reconstruct the walking Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_patch_placement() {
        let action = Action::PatchPlacement {
            patch_id: 13,
            patch_index: 2,
            patch_transformation_index: 5,
            previous_player_was_1: true,
        };

        let action_id = ActionId::from_action(&action);

        assert!(
            action_id.is_patch_placement(),
            "Action id is not a patch placement action"
        );

        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!("Surrogate Action Id - Patch Id: {}", action_id.get_patch_id());
        println!("Surrogate Action Id - Patch Index: {}", action_id.get_patch_index());
        println!(
            "Surrogate Action Id - Patch Transformation Index: {}",
            action_id.get_patch_transformation_index()
        );
        println!(
            "Surrogate Action Id - Prev Player 1: {}",
            action_id.get_previous_player_was_1()
        );

        assert_eq!(
            action,
            action_id.to_action(),
            "Surrogate Action Id does not reconstruct the patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_special_patch_placement() {
        let action = Action::SpecialPatchPlacement { quilt_board_index: 13 };

        let action_id = ActionId::from_action(&action);

        assert!(
            action_id.is_special_patch_placement(),
            "Action id is not a special patch placement action"
        );

        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!(
            "Surrogate Action Id - Quilt Board Index: {}",
            action_id.get_quilt_board_index()
        );

        assert_eq!(
            action,
            action_id.to_action(),
            "Surrogate Action Id does not reconstruct the special patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_phantom() {
        let action = Action::Phantom;

        let action_id = ActionId::from_action(&action);

        assert!(action_id.is_phantom(), "Action id is not a phantom action.");
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        assert_eq!(
            action,
            action_id.to_action(),
            "Surrogate Action Id does not reconstruct the phantom Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_null() {
        let action = Action::Null;

        let action_id = ActionId::from_action(&action);

        assert!(action_id.is_null(), "Action id is not a null action.");
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        assert_eq!(
            action,
            action_id.to_action(),
            "Surrogate Action Id does not reconstruct the null Action."
        );
    }

    // ───────────────────────────────────────── TO NATURAL ACTION ID AND BACK ─────────────────────────────────────────

    #[test]
    pub fn convert_to_natural_action_id_and_back_walking() {
        let action = NaturalActionId::walking(13);

        let action_id = ActionId::from_natural_action_id(action);

        assert!(action_id.is_walking(), "Action id is not a walking action.");

        println!("Natural Action Id - {}", as_bits_string(action));
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!(
            "Surrogate Action Id - Starting Index: {}",
            action_id.get_starting_index()
        );

        assert_eq!(
            action,
            action_id.to_natural_action_id(),
            "Surrogate Action Id does not reconstruct the walking Action."
        );
    }

    #[test]
    pub fn convert_to_natural_action_id_and_back_patch_placement() {
        let action = NaturalActionId::patch_placement(13, 2, 5, true);

        let action_id = ActionId::from_natural_action_id(action);

        assert!(
            action_id.is_patch_placement(),
            "Action id is not a patch placement action"
        );

        println!("Natural Action Id - {}", as_bits_string(action));
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!("Surrogate Action Id - Patch Id: {}", action_id.get_patch_id());
        println!("Surrogate Action Id - Patch Index: {}", action_id.get_patch_index());
        println!(
            "Surrogate Action Id - Patch Transformation Index: {}",
            action_id.get_patch_transformation_index()
        );
        println!(
            "Surrogate Action Id - Prev Player 1: {}",
            action_id.get_previous_player_was_1()
        );

        assert_eq!(
            action,
            action_id.to_natural_action_id(),
            "Surrogate Action Id does not reconstruct the patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_natural_action_id_and_back_special_patch_placement() {
        let action = NaturalActionId::special_patch_placement(13);

        let action_id = ActionId::from_natural_action_id(action);

        assert!(
            action_id.is_special_patch_placement(),
            "Action id is not a special patch placement action"
        );

        println!("Natural Action Id - {}", as_bits_string(action));
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        println!(
            "Surrogate Action Id - Quilt Board Index: {}",
            action_id.get_quilt_board_index()
        );

        assert_eq!(
            action,
            action_id.to_natural_action_id(),
            "Surrogate Action Id does not reconstruct the special patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_natural_action_id_and_back_phantom() {
        let action = NaturalActionId::phantom();

        let action_id = ActionId::from_natural_action_id(action);

        assert!(action_id.is_phantom(), "Action id is not a phantom action.");
        println!("Natural Action Id - {}", as_bits_string(action));
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        assert_eq!(
            action,
            action_id.to_natural_action_id(),
            "Surrogate Action Id does not reconstruct the phantom Action."
        );
    }

    #[test]
    pub fn convert_to_natural_action_id_and_back_null() {
        let action = NaturalActionId::null();

        let action_id = ActionId::from_natural_action_id(action);

        assert!(action_id.is_null(), "Action id is not a null action.");
        println!("Natural Action Id - {}", as_bits_string(action));
        println!("Surrogate Action Id - {:032b}", action_id.as_bits());
        assert_eq!(
            action,
            action_id.to_natural_action_id(),
            "Surrogate Action Id does not reconstruct the null Action."
        );
    }
}
