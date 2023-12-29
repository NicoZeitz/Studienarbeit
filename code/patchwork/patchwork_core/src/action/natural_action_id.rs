use std::{fmt::Display, hash::Hash};

use crate::{Action, ActionId, PatchManager, PatchTransformation, QuiltBoard};

/// The natural action id is a natural unique number that is used to identify an action.
/// It is used to represent an action as a single number where that number has a meaning associated to it
/// so that it can used for example as an output of a neural network.
///
/// The id's are distributed as follows:
/// - \[0,  0]: Walking action
/// - \[1, 81]: Special patch placement actions.
///    - Containing the row where the special patch will be placed.
///    - Containing the column where the special patch will be placed.
/// - \[82, 2025]: Patch placement actions.
///   - Containing the patch index in the list of available patches
///     (always 0,1 or 2).
///   - The row where the patch will be placed.
///   - The column where the patch will be placed.
///   - The rotation of the patch.
///   - The orientation of the patch.
/// - \[2026, 2026]: Phantom action.
/// - \[2027, 2027]: Null action.
///
/// Actually the ranges are not entirely correct as for walking actions the top
/// 8 bits are used to save the starting index of the walking action.
/// Furthermore for patch placement actions the top 25 bits are used to save the
/// patch id (8 bits), the patch transformation index (16 bits) and if the
/// previous player was player 1 (1 bit). This is done to allow for conversion
/// to action and surrogate action id. If the top bits are not set for these
/// actions it is not possible, to convert the natural action id to an action
/// or a surrogate action id.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NaturalActionId(u64);

impl PartialEq for NaturalActionId {
    fn eq(&self, other: &Self) -> bool {
        self.as_bits() == other.as_bits()
    }
}

impl Eq for NaturalActionId {}

impl Hash for NaturalActionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bits().hash(state)
    }
}

impl NaturalActionId {
    /// The natural id of the walking action.
    pub const WALKING_ACTION_ID: u64 = 0;
    /// The starting natural id of the special patch placement actions.
    pub const SPECIAL_PATCH_PLACEMENT_ID_START: u64 = 1;
    /// The ending natural id of the special patch placement actions.
    pub const SPECIAL_PATCH_PLACEMENT_ID_END: u64 = 81;
    /// The starting natural id of the patch placement actions.
    pub const PATCH_PLACEMENT_ID_START: u64 = 82;
    /// The ending natural id of the patch placement actions.
    pub const PATCH_PLACEMENT_ID_END: u64 = 2025;
    /// The natural id of the phantom action.
    pub const PHANTOM_ACTION_ID: u64 = 2026;
    /// The natural id of a null action.
    pub const NULL_ACTION_ID: u64 = 2027;

    /// The amount of available natural action ids for the game of patchwork.
    ///
    /// The actually allowed actions are way lower than this number,
    /// but we need to be able to represent all the possible actions in a single number.
    /// This is the maximum amount of actions that can be taken in a single turn.
    ///
    /// The actually best it is 2025, phantom action have id 2026 and
    /// null actions have id 2027
    pub const AMOUNT_OF_NATURAL_ACTION_IDS: u64 = 2028;

    /// The mask to remove the top 25 bits of the natural action id.
    const TOP_BIT_MASK: u64 = 0x0000_007F_FFFF_FFFFu64;

    /// Creates a walking natural action id
    ///
    /// # Arguments
    ///
    /// * `starting_index` - The starting index of the walking action.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding to the given walking action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn walking(starting_index: u8) -> NaturalActionId {
        Self(transform_walking_to_natural_id(starting_index))
    }

    /// Creates a patch placement natural action id
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch to place.
    /// * `patch_index` - The index of the patch to place.
    /// * `patch_transformation_index` - The index of the transformation of the patch to place.
    /// * `was_previous_player_1` - Whether the previous player was player 1.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding to the given patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub fn patch_placement(
        patch_id: u8,
        patch_index: u8,
        patch_transformation_index: u16,
        was_previous_player_1: bool,
    ) -> NaturalActionId {
        Self(transform_patch_placement_to_natural_id(
            patch_id,
            patch_index,
            patch_transformation_index,
            was_previous_player_1,
        ))
    }

    /// Creates a special patch placement natural action id
    ///
    /// # Arguments
    ///
    /// * `quilt_board_index` - The index of the quilt board where the special patch will be placed.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding to the given special patch placement action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn special_patch_placement(quilt_board_index: u8) -> NaturalActionId {
        Self(quilt_board_index as u64 + Self::SPECIAL_PATCH_PLACEMENT_ID_START)
    }

    /// Creates a phantom natural action id
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding to the phantom action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn phantom() -> NaturalActionId {
        Self(Self::PHANTOM_ACTION_ID)
    }

    /// Creates a null natural action id
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding to the null action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn null() -> NaturalActionId {
        Self(Self::NULL_ACTION_ID)
    }

    /// Returns if the given id is a valid natural action id.
    ///
    /// # Arguments
    ///
    /// * `natural_action_id` - The given id to check.
    ///
    /// # Returns
    ///
    /// * `true` if the given id is a valid natural action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn is_valid_natural_action_id(natural_action_id: u64) -> bool {
        (natural_action_id & Self::TOP_BIT_MASK) <= Self::NULL_ACTION_ID
    }

    /// Gets the natural action id as a u64
    ///
    /// # Returns
    ///
    /// The natural action id as a u64
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn as_bits(&self) -> u64 {
        // mask of the top 24 bits
        self.0 & Self::TOP_BIT_MASK
    }

    /// Gets the natural action id without the masked hidden information removed as a u64
    ///
    /// # Returns
    ///
    /// The natural action id without the masked hidden information removed as a u64
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn as_bits_with_hidden_information(&self) -> u64 {
        self.0
    }

    /// Returns if the natural action id contains hidden information.
    ///
    /// # Returns
    ///
    /// If the natural action id contains hidden information.
    /// Hidden Information is the patch id and the patch transformation index
    /// for patch placement actions.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn contains_hidden_information(&self) -> bool {
        self.0 & Self::TOP_BIT_MASK != self.0
    }

    /// Creates a new natural action id from the given bits.
    ///
    /// # Arguments
    ///
    /// * `bits` - The bits to create the natural action id from.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding the given bits.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the given bits are not a valid natural action id.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn from_bits(bits: u64) -> Self {
        debug_assert!(Self::is_valid_natural_action_id(bits));
        Self(bits)
    }

    /// Creates a new natural action id from the given action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to create the natural action id from.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding the given action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[rustfmt::skip]
    pub fn from_action(action: &Action) -> Self {
        Self(match action {
            Action::Walking { starting_index } => transform_walking_to_natural_id(*starting_index),
            Action::SpecialPatchPlacement { quilt_board_index } => {
                (*quilt_board_index) as u64 + Self::SPECIAL_PATCH_PLACEMENT_ID_START
            }
            Action::PatchPlacement {
                patch_id,
                patch_index,
                patch_transformation_index,
                previous_player_was_1
            } => transform_patch_placement_to_natural_id(*patch_id, *patch_index, *patch_transformation_index, *previous_player_was_1),
            Action::Phantom => Self::PHANTOM_ACTION_ID,
            Action::Null => Self::NULL_ACTION_ID,
        })
    }

    /// Creates a new natural action id from the given surrogate action id.
    ///
    /// # Arguments
    ///
    /// * `action_id` - The surrogate action id to create the natural action id from.
    ///
    /// # Returns
    ///
    /// * The natural action id corresponding the given surrogate action id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn from_surrogate_action_id(action_id: ActionId) -> NaturalActionId {
        debug_assert!(ActionId::is_valid_action_id(action_id.as_bits()));

        let masked_surrogate_action_id = action_id.as_bits();
        Self(match masked_surrogate_action_id {
            ActionId::WALKING_ACTION_ID_START..=ActionId::WALKING_ACTION_ID_END => {
                transform_walking_to_natural_id(action_id.get_starting_index())
            }
            ActionId::SPECIAL_PATCH_PLACEMENT_ID_START..=ActionId::SPECIAL_PATCH_PLACEMENT_ID_END => {
                masked_surrogate_action_id as u64 - ActionId::SPECIAL_PATCH_PLACEMENT_ID_START as u64
                    + Self::SPECIAL_PATCH_PLACEMENT_ID_START
            }
            ActionId::PATCH_PLACEMENT_ID_START..=ActionId::PATCH_PLACEMENT_ID_END => {
                let patch_id = action_id.get_patch_id();
                let patch_index = action_id.get_patch_index();
                let patch_transformation_index = action_id.get_patch_transformation_index();
                let previous_player_was_1 = action_id.get_previous_player_was_1();

                transform_patch_placement_to_natural_id(
                    patch_id,
                    patch_index,
                    patch_transformation_index,
                    previous_player_was_1,
                )
            }
            ActionId::PHANTOM_ACTION_ID => NaturalActionId::PHANTOM_ACTION_ID,
            ActionId::NULL_ACTION_ID => NaturalActionId::NULL_ACTION_ID,
            _ => unreachable!("[NaturalActionId::from_action_id] Invalid surrogate action id."),
        })
    }

    /// Creates the corresponding action from the this natural action id.
    ///
    /// # Returns
    ///
    /// The corresponding action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Panics
    ///
    /// Panics if this natural action id is a walking action or a patch placement and does not
    /// contain hidden information.
    #[inline(always)]
    pub fn to_action(&self) -> Action {
        Action::from_natural_action_id(*self)
    }

    /// Tries to create the corresponding action from the this natural action id.
    ///
    /// # Returns
    ///
    /// * `Some(action)` if the action a walking action and contains hidden information or if the
    ///   action is a patch placement action and contains hidden information or if it is any other
    ///   action.
    /// * `None` if the action is a walking action and does not contain hidden information or if the
    ///   action is a patch placement action and does not contain hidden information.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub fn try_to_action(&self) -> Option<Action> {
        if (self.is_walking() || self.is_patch_placement()) && !self.contains_hidden_information() {
            None
        } else {
            Some(Action::from_natural_action_id(*self))
        }
    }

    // TODO: method to convert while providing the necessary information (e.g. starting index, patch id, patch transformation index)

    /// Gets the surrogate action id from this natural action id
    ///
    /// # Returns
    ///
    /// The surrogate action id
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Panics
    ///
    /// Panics if this natural action id is a walking action or a patch placement and does not
    /// contain hidden information.
    #[inline(always)]
    pub fn to_surrogate_action_id(&self) -> ActionId {
        ActionId::from_natural_action_id(*self)
    }

    /// Tries to get the surrogate action id from this natural action id
    ///
    /// # Returns
    ///
    /// * `Some(action_id)` if the action a walking action and contains hidden information or if the
    ///   action is a patch placement action and contains hidden information or if it is any other
    ///   action.
    /// * `None` if the action is a walking action and does not contain hidden information or if the
    ///   action is a patch placement action and does not contain hidden information.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub fn try_to_surrogate_action_id(&self) -> Option<ActionId> {
        if (self.is_walking() || self.is_patch_placement()) && !self.contains_hidden_information() {
            None
        } else {
            Some(ActionId::from_natural_action_id(*self))
        }
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
    #[inline(always)]
    pub const fn is_walking(&self) -> bool {
        self.as_bits() == Self::WALKING_ACTION_ID
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
    #[inline(always)]
    pub const fn is_special_patch_placement(&self) -> bool {
        let masked_id = self.as_bits();
        masked_id >= Self::SPECIAL_PATCH_PLACEMENT_ID_START && masked_id <= Self::SPECIAL_PATCH_PLACEMENT_ID_END
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
    #[inline(always)]
    pub const fn is_patch_placement(&self) -> bool {
        let masked_id = self.as_bits();
        masked_id >= Self::PATCH_PLACEMENT_ID_START && masked_id <= Self::PATCH_PLACEMENT_ID_END
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
    #[inline(always)]
    pub const fn is_phantom(&self) -> bool {
        self.as_bits() == Self::PHANTOM_ACTION_ID
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
    #[inline(always)]
    pub const fn is_null(&self) -> bool {
        self.as_bits() == Self::NULL_ACTION_ID
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
    ///
    /// # Undefined Behavior
    ///
    /// If the natural action id does not contain hidden information.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn is_first_patch_taken(&self) -> bool {
        debug_assert!(self.contains_hidden_information());
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
    ///
    /// # Undefined Behavior
    ///
    /// If the natural action id does not contain hidden information.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn is_second_patch_taken(&self) -> bool {
        debug_assert!(self.contains_hidden_information());
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
    ///
    /// # Undefined Behavior
    ///
    /// If the natural action id does not contain hidden information.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn is_third_patch_taken(&self) -> bool {
        debug_assert!(self.contains_hidden_information());
        self.is_patch_placement() && self.get_patch_index() == 2
    }

    /// Returns the starting index of the walking action.
    /// Only available for walking actions.
    ///
    /// # Returns
    ///
    /// The starting index of the walking action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a walking action or does not contain hidden
    /// information. This will panic in debug mode.
    #[inline(always)]
    pub const fn get_starting_index(&self) -> u8 {
        debug_assert!(self.is_walking());
        debug_assert!(self.contains_hidden_information());

        (self.0 >> 56) as u8
    }

    /// Returns the patch id of the patch to be placed
    /// Only available for patch placement actions.
    ///
    /// # Returns
    ///
    /// The patch id of the patch to be placed
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement action or does not contain hidden
    /// information. This will panic in debug mode.
    #[inline(always)]
    pub const fn get_patch_id(&self) -> u8 {
        debug_assert!(self.is_patch_placement());
        debug_assert!(self.contains_hidden_information());

        (self.0 >> 56) as u8
    }

    /// Returns the patch index of the action.
    /// Only available for patch placement actions.
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
    /// If the action is not a patch placement action.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn get_patch_index(&self) -> u8 {
        debug_assert!(self.is_patch_placement());

        const ROWS: u64 = QuiltBoard::ROWS as u64;
        const COLUMNS: u64 = QuiltBoard::COLUMNS as u64;
        const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        ((self.as_bits() - Self::PATCH_PLACEMENT_ID_START) / (ROWS * COLUMNS * ROTATIONS * ORIENTATIONS)) as u8
    }

    /// Returns the quilt board index where the patch is to be placed
    /// Only available for patch placement or special patch placement actions.
    ///
    /// # Returns
    ///
    /// The quilt board index where the patch is to be placed
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement or special patch placement action.
    /// This will panic in debug mode.
    #[inline(always)]
    pub const fn get_quilt_board_index(&self) -> u8 {
        debug_assert!(self.is_patch_placement() || self.is_special_patch_placement());

        const ROWS: u64 = QuiltBoard::ROWS as u64;
        const COLUMNS: u64 = QuiltBoard::COLUMNS as u64;
        const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        let masked_id = self.as_bits();

        if self.is_special_patch_placement() {
            return (masked_id - Self::SPECIAL_PATCH_PLACEMENT_ID_START) as u8;
        }

        (((masked_id - Self::PATCH_PLACEMENT_ID_START) / (COLUMNS * ROTATIONS * ORIENTATIONS)) % ROWS) as u8
            * COLUMNS as u8
            + (((masked_id - Self::PATCH_PLACEMENT_ID_START) / (ROTATIONS * ORIENTATIONS)) % COLUMNS) as u8
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
    pub const fn get_row(&self) -> u8 {
        debug_assert!(self.is_patch_placement() || self.is_special_patch_placement());

        const ROWS: u64 = QuiltBoard::ROWS as u64;
        const COLUMNS: u64 = QuiltBoard::COLUMNS as u64;
        const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        let masked_id = self.as_bits();

        if self.is_special_patch_placement() {
            return ((masked_id - Self::SPECIAL_PATCH_PLACEMENT_ID_START) / COLUMNS) as u8;
        }

        (((masked_id - Self::PATCH_PLACEMENT_ID_START) / (COLUMNS * ROTATIONS * ORIENTATIONS)) % ROWS) as u8
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
    pub const fn get_column(&self) -> u8 {
        debug_assert!(self.is_patch_placement() || self.is_special_patch_placement());

        const COLUMNS: u64 = QuiltBoard::COLUMNS as u64;
        const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        let masked_id = self.as_bits();

        if self.is_special_patch_placement() {
            return ((masked_id - Self::SPECIAL_PATCH_PLACEMENT_ID_START) % COLUMNS) as u8;
        }

        (((masked_id - Self::PATCH_PLACEMENT_ID_START) / (ROTATIONS * ORIENTATIONS)) % COLUMNS) as u8
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
    #[inline(always)]
    pub const fn get_rotation(&self) -> u8 {
        debug_assert!(self.is_patch_placement());

        const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        ((self.as_bits() - Self::PATCH_PLACEMENT_ID_START) / (ORIENTATIONS) % ROTATIONS) as u8
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
    #[inline(always)]
    pub const fn get_orientation(&self) -> u8 {
        debug_assert!(self.is_patch_placement());

        const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

        (self.as_bits() - Self::PATCH_PLACEMENT_ID_START % ORIENTATIONS) as u8
    }

    /// Returns the patch transformation index of the patch to be placed
    /// Only available for patch placement actions.
    ///
    /// # Returns
    ///
    /// The patch transformation index of the patch to be placed
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// If the action is not a patch placement action or does not contain hidden
    /// information. This will panic in debug mode.
    #[inline(always)]
    pub const fn get_patch_transformation_index(&self) -> u16 {
        debug_assert!(self.is_patch_placement());
        debug_assert!(self.contains_hidden_information());

        ((self.0 >> 40) & 0xFFFF) as u16
    }

    /// Returns if the previous player was player 1.
    /// Only available for patch placement actions.
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
    /// If the action is not a patch placement action or does not contain hidden
    /// information. This will panic in debug mode.
    #[inline(always)]
    pub const fn get_previous_player_was_1(&self) -> bool {
        debug_assert!(self.is_patch_placement());
        debug_assert!(self.contains_hidden_information());

        ((self.0 >> 39) & 0x01) == 1
    }
}

impl From<Action> for NaturalActionId {
    fn from(action: Action) -> Self {
        Self::from_action(&action)
    }
}

impl From<ActionId> for NaturalActionId {
    fn from(action_id: ActionId) -> Self {
        Self::from_surrogate_action_id(action_id)
    }
}

impl Display for NaturalActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(action) = self.try_to_action() {
            return action.fmt(f);
        }

        f.write_fmt(format_args!("NaturalActionId({})", self.as_bits()))
    }
}

/// Transforms the given walking action into a natural action id.
///
/// # Arguments
///
/// * `starting_index` - The starting index of the walking action.
///
/// # Returns
///
/// * The natural action id corresponding to the given walking action.
///
/// # Complexity
///
/// `𝒪(𝟣)`
const fn transform_walking_to_natural_id(starting_index: u8) -> u64 {
    let id = NaturalActionId::WALKING_ACTION_ID;
    let hidden_information = (starting_index as u64) << 56;
    id + hidden_information
}

/// Transforms the given patch placement action into a natural action id.
///
/// # Arguments
///
/// * `patch_id` - The id of the patch to place.
/// * `patch_index` - The index of the patch to place.
/// * `patch_transformation_index` - The index of the transformation of the patch to place.
/// * `was_previous_player_1` - Whether the previous player was player 1.
///
/// # Returns
///
/// * The natural action id corresponding to the given patch placement action.
///
/// # Complexity
///
/// `𝒪(𝟣)`
fn transform_patch_placement_to_natural_id(
    patch_id: u8,
    patch_index: u8,
    patch_transformation_index: u16,
    was_previous_player_1: bool,
) -> u64 {
    // the maximum amount of placement for a patch is actually 448. The patch is:
    // ▉
    // ▉▉▉
    // but as we want to be able to represent all the information in a single number, we need to use
    // [(((index * ROWS + row) * COLUMNS + column) * ROTATIONS + rotation) * ORIENTATIONS + orientation + OFFSET] as limit for the action
    const ROWS: u64 = QuiltBoard::ROWS as u64;
    const COLUMNS: u64 = QuiltBoard::COLUMNS as u64;
    const ROTATIONS: u64 = PatchTransformation::AMOUNT_OF_ROTATIONS as u64;
    const ORIENTATIONS: u64 = PatchTransformation::AMOUNT_OF_ORIENTATIONS as u64;

    let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);

    println!("Transformation Index: {:?}", patch_transformation_index);

    let patch_index = patch_index as u64;
    let row = transformation.row as u64;
    let column = transformation.column as u64;
    let rotation = transformation.rotation_flag() as u64;
    let orientation = transformation.orientation_flag() as u64;

    let id = patch_index * ROWS * COLUMNS * ROTATIONS * ORIENTATIONS
        + row * COLUMNS * ROTATIONS * ORIENTATIONS
        + column * ROTATIONS * ORIENTATIONS
        + rotation * ORIENTATIONS
        + orientation
        + NaturalActionId::PATCH_PLACEMENT_ID_START;

    // the top 25 bits are used to save the patch id (8 bits), the patch transformation index (16 bits) and if the
    // previous player was player 1 (1 bit). This allows for conversion to action and surrogate action id
    let hidden_information = ((patch_id as u64) << 56)
        + ((patch_transformation_index as u64) << 40)
        + ((was_previous_player_1 as u64) << 39);

    id + hidden_information
}

#[cfg(test)]
mod tests {
    use super::{Action, ActionId, NaturalActionId};

    use pretty_assertions::assert_eq;

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

        let action_id = NaturalActionId::from_action(&action);

        assert!(action_id.is_walking(), "Action id is not a walking action.");

        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!("Natural Action Id - Starting Index: {}", action_id.get_starting_index());

        assert_eq!(
            action,
            action_id.to_action(),
            "Natural Action Id does not reconstruct the walking Action."
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

        let action_id = NaturalActionId::from_action(&action);

        assert!(
            action_id.is_patch_placement(),
            "Action id is not a patch placement action"
        );

        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!("Natural Action Id - Patch Id: {}", action_id.get_patch_id());
        println!("Natural Action Id - Patch Index: {}", action_id.get_patch_index());
        println!(
            "Natural Action Id - Patch Transformation Index: {}",
            action_id.get_patch_transformation_index()
        );
        println!(
            "Natural Action Id - Prev Player 1: {}",
            action_id.get_previous_player_was_1()
        );

        assert_eq!(
            action,
            action_id.to_action(),
            "Natural Action Id does not reconstruct the patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_special_patch_placement() {
        let action = Action::SpecialPatchPlacement { quilt_board_index: 13 };

        let action_id = NaturalActionId::from_action(&action);

        assert!(
            action_id.is_special_patch_placement(),
            "Action id is not a special patch placement action"
        );

        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!(
            "Natural Action Id - Quilt Board Index: {}",
            action_id.get_quilt_board_index()
        );

        assert_eq!(
            action,
            action_id.to_action(),
            "Natural Action Id does not reconstruct the special patch placement Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_phantom() {
        let action = Action::Phantom;

        let action_id = NaturalActionId::from_action(&action);

        assert!(action_id.is_phantom(), "Action id is not a phantom action.");
        println!("Natural Action Id - {}", as_bits_string(action_id));
        assert_eq!(
            action,
            action_id.to_action(),
            "Natural Action Id does not reconstruct the phantom Action."
        );
    }

    #[test]
    pub fn convert_to_action_and_back_null() {
        let action = Action::Null;

        let action_id = NaturalActionId::from_action(&action);

        assert!(action_id.is_null(), "Action id is not a null action.");
        println!("Natural Action Id - {}", as_bits_string(action_id));
        assert_eq!(
            action,
            action_id.to_action(),
            "Natural Action Id does not reconstruct the null Action."
        );
    }

    // ──────────────────────────────────────── TO SURROGATE ACTION ID AND BACK ────────────────────────────────────────

    #[test]
    pub fn convert_to_surrogate_action_id_and_back_walking() {
        let action = ActionId::walking(13);

        let action_id = NaturalActionId::from_surrogate_action_id(action);

        assert!(action_id.is_walking(), "Action id is not a walking action.");

        println!("Surrogate Action Id - {:032b}", action.as_bits());
        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!("Natural Action Id - Starting Index: {}", action_id.get_starting_index());

        assert_eq!(
            action,
            action_id.to_surrogate_action_id(),
            "Natural Action Id does not reconstruct the walking Surrogate Action Id."
        );
    }

    #[test]
    pub fn convert_to_surrogate_action_id_and_back_patch_placement() {
        let action = ActionId::patch_placement(13, 2, 5, true);

        let action_id = NaturalActionId::from_surrogate_action_id(action);

        assert!(
            action_id.is_patch_placement(),
            "Action id is not a patch placement action"
        );

        println!("Surrogate Action Id - {:032b}", action.as_bits());
        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!("Natural Action Id - Patch Id: {}", action_id.get_patch_id());
        println!("Natural Action Id - Patch Index: {}", action_id.get_patch_index());
        println!(
            "Natural Action Id - Patch Transformation Index: {}",
            action_id.get_patch_transformation_index()
        );
        println!(
            "Natural Action Id - Prev Player 1: {}",
            action_id.get_previous_player_was_1()
        );

        assert_eq!(
            action,
            action_id.to_surrogate_action_id(),
            "Natural Action Id does not reconstruct the patch placement Surrogate Action Id."
        );
    }

    #[test]
    pub fn convert_to_surrogate_action_id_and_back_special_patch_placement() {
        let action = ActionId::special_patch_placement(13);

        let action_id = NaturalActionId::from_surrogate_action_id(action);

        assert!(
            action_id.is_special_patch_placement(),
            "Action id is not a special patch placement action"
        );

        println!("Surrogate Action Id - {:032b}", action.as_bits());
        println!("Natural Action Id - {}", as_bits_string(action_id));
        println!(
            "Natural Action Id - Quilt Board Index: {}",
            action_id.get_quilt_board_index()
        );

        assert_eq!(
            action,
            action_id.to_surrogate_action_id(),
            "Natural Action Id does not reconstruct the special patch placement Surrogate Action Id."
        );
    }

    #[test]
    pub fn convert_to_surrogate_action_id_and_back_phantom() {
        let action = ActionId::phantom();

        let action_id = NaturalActionId::from_surrogate_action_id(action);

        assert!(action_id.is_phantom(), "Action id is not a phantom action.");
        println!("Surrogate Action Id - {:032b}", action.as_bits());
        println!("Natural Action Id - {}", as_bits_string(action_id));
        assert_eq!(
            action,
            action_id.to_surrogate_action_id(),
            "Natural Action Id does not reconstruct the phantom Surrogate Action Id."
        );
    }

    #[test]
    pub fn convert_to_surrogate_action_id_and_back_null() {
        let action = ActionId::null();

        let action_id = NaturalActionId::from_surrogate_action_id(action);

        assert!(action_id.is_null(), "Action id is not a null action.");
        println!("Surrogate Action Id - {:032b}", action.as_bits());
        println!("Natural Action Id - {}", as_bits_string(action_id));
        assert_eq!(
            action,
            action_id.to_surrogate_action_id(),
            "Natural Action Id does not reconstruct the null Surrogate Action Id."
        );
    }
}
