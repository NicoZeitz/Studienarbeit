use patchwork_core::{evaluator_constants, Action, ActionPayload, Patch};

use super::EvaluationType;

/// An entry in the transposition table. The table uses
/// [Lockless Hashing](https://www.chessprogramming.org/Shared_Hash_Table#Lock-less)
/// with XOR for the entries.
///
/// Furthermore extra_data is stored to allow for full action reconstruction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Entry {
    /// The key of this entry.
    /// This is the zobrist hash of the position XORed with the data.
    pub(crate) key: u64,
    /// The data of this entry. Consists of
    /// - `depth`           (8 Bits): The depth at which this entry was stored
    /// - `evaluation`      (43 Bits): The evaluation of the position
    /// - `evaluation_type` (2 Bits): The type of node this entry is
    /// - `best_action`     (11 Bits): The best action to take in this position
    ///
    /// The data is stored as `MSB evaluation(43 Bits)|action_id(11 Bits)|depth(8 Bits)|evaluation_type(2 Bits) LSB`
    pub(crate) data: u64,
    /// The age of this entry. Used to determine when to overwrite entries from
    /// searching previous positions during the game
    pub(crate) age: usize,
    /// Extra additional metadata needed for full action reconstruction
    pub(crate) extra_data: EntryExtraData,
}

/// Extra additional metadata needed for full action reconstruction that is
/// stored in the transposition table.
///
/// The action_id is not actually needed but instead used as a sanity check to
/// make sure the action is the same as the one that was stored in the data
/// field in Entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EntryExtraData {
    /// For Null Action
    Null,
    /// For Walking Action
    /// * starting_index
    Walking { starting_index: usize },
    /// For PatchPlacement Action
    /// * patch
    /// * starting_index
    /// * next_quilt_board
    /// * previous_quilt_board
    ///
    /// `next_quilt_board` and `previous_quilt_board` are guaranteed to exist as
    /// the only time they do not exist is when the action is parsed from
    /// notation. This is not the case here.
    PatchPlacement {
        action_id: usize,
        patch: &'static Patch,
        starting_index: usize,
        next_quilt_board: u128,
        previous_quilt_board: u128,
    },
    /// For SpecialPatchPlacement Action
    /// * patch
    /// * next_quilt_board
    /// * previous_quilt_board
    ///
    /// `next_quilt_board` and `previous_quilt_board` are guaranteed to exist as
    /// the only time they do not exist is when the action is parsed from
    /// notation. This is not the case here.
    SpecialPatchPlacement {
        action_id: usize,
        patch_id: usize,
        next_quilt_board: u128,
        previous_quilt_board: u128,
    },
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            key: 0,
            data: 0,
            age: 0,
            extra_data: EntryExtraData::Null,
        }
    }
}

impl Entry {
    /// Extracts the evaluation type from the data field.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `EvaluationType` - The evaluation type
    #[inline]
    pub(crate) fn get_evaluation_type(data: u64) -> EvaluationType {
        match data & 0b11 {
            0 => EvaluationType::Exact,
            1 => EvaluationType::UpperBound,
            2 => EvaluationType::LowerBound,
            _ => unreachable!(),
        }
    }

    /// Extracts the depth from the data field.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `usize` - The depth
    #[inline]
    pub(crate) fn get_depth(data: u64) -> usize {
        ((data >> 2) & 0xFF) as usize
    }

    /// Extracts the action id from the data field.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `usize` - The action id
    #[inline]
    pub(crate) fn get_action_id(data: u64) -> usize {
        ((data >> 10) & 0x7FF) as usize
    }

    /// Extracts the evaluation from the data field.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `isize` - The evaluation
    #[inline]
    pub(crate) fn get_evaluation(data: u64) -> isize {
        let extracted = (data >> 20) & 0x7FFFFFFFFFF;

        (unsafe { std::mem::transmute::<u64, i64>(extracted) }) as isize + evaluator_constants::NEGATIVE_INFINITY
    }

    /// Extracts the action from the data and extra_data field.
    ///
    /// Returns None if the data and extra_data do not match.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `Option<Action>` - The action
    /// * `None` - The data and extra_data do not match
    #[inline]
    pub(crate) fn get_action(data: u64, extra_data: EntryExtraData) -> Option<Action> {
        let action_id = Self::get_action_id(data);
        Self::get_action_internal(action_id, extra_data)
    }

    /// Unpacks the data field into the evaluation, evaluation type, depth and
    /// action.
    ///
    /// Returns None if the data and extra_data do not match.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    /// * `extra_data` - The extra data field of the entry
    ///
    /// # Returns
    ///
    /// * `Some((depth, evaluation, evaluation_type, action))` - The unpacked data
    /// * `None` - The data and extra_data do not match
    pub(crate) fn unpack_data(data: u64, extra_data: EntryExtraData) -> Option<(usize, isize, EvaluationType, Action)> {
        let evaluation_type = Self::get_evaluation_type(data);
        let depth = Self::get_depth(data);
        let action_id = Self::get_action_id(data);
        let evaluation = Self::get_evaluation(data);
        let action = Self::get_action_internal(action_id, extra_data)?;

        Some((depth, evaluation, evaluation_type, action))
    }

    /// Packs the evaluation, evaluation type, depth and action into the data
    /// field.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth at which the evaluation was calculated
    /// * `evaluation` - The evaluation of the position
    /// * `evaluation_type` - The type of node this entry is
    /// * `action` - The best action to take in this position
    ///
    /// # Returns
    ///
    /// * `(data, extra_data)` - The packed data and extra data
    #[rustfmt::skip]
    pub(crate) fn pack_data(
        depth: usize,
        evaluation: isize,
        evaluation_type: EvaluationType,
        action: Action,
    ) -> (u64, EntryExtraData) {
        // Force evaluation to be positive to allow reconstruction of the number later on
        let adjusted_evaluation = (evaluation - evaluator_constants::NEGATIVE_INFINITY) as i64;
        let action_id = action.id;
        let mut data = 0u64;
        data |=  evaluation_type as u64;        //  2 bits for evaluation type
        data |= (depth           as u64) << 2;  //  8 bits for depth     (as a max depth of 256 is used)
        data |= (action_id       as u64) << 10; // 11 bits for action id (as a max of 2026 actions are possible)
        data |= unsafe {std::mem::transmute::<i64, u64>(adjusted_evaluation) } << 20; // 43 bits left for evaluation

        let extra_data = match action.payload {
            patchwork_core::ActionPayload::Null => EntryExtraData::Null,
            patchwork_core::ActionPayload::Walking { starting_index } => EntryExtraData::Walking { starting_index },
            patchwork_core::ActionPayload::PatchPlacement {
                patch,
                starting_index,
                next_quilt_board,
                previous_quilt_board,
                ..
            } => EntryExtraData::PatchPlacement {
                action_id,
                patch,
                starting_index,
                next_quilt_board: next_quilt_board.unwrap(), // This is guaranteed to exist
                previous_quilt_board: previous_quilt_board.unwrap(), // This is guaranteed to exist
            },
            patchwork_core::ActionPayload::SpecialPatchPlacement {
                patch_id,
                next_quilt_board,
                previous_quilt_board,
                ..
            } => EntryExtraData::SpecialPatchPlacement {
                action_id,
                patch_id,
                next_quilt_board: next_quilt_board.unwrap(), // This is guaranteed to exist
                previous_quilt_board: previous_quilt_board.unwrap(), // This is guaranteed to exist
            },
        };

        (data, extra_data)
    }

    fn get_action_internal(action_id: usize, extra_data: EntryExtraData) -> Option<Action> {
        match action_id {
            Action::NULL_ACTION_ID => Some(Action::null()),
            Action::WALKING_ACTION_ID => match extra_data {
                EntryExtraData::Walking { starting_index } => Some(Action::walking(starting_index)),
                _ => None, // data & extra_data mismatch
            },
            Action::SPECIAL_PATCH_PLACEMENT_ID_START..=Action::SPECIAL_PATCH_PLACEMENT_ID_END => match extra_data {
                EntryExtraData::SpecialPatchPlacement {
                    action_id: stored_action_id,
                    patch_id: stored_patch_id,
                    next_quilt_board,
                    previous_quilt_board,
                } => {
                    if action_id != stored_action_id {
                        return None; // data & extra_data mismatch
                    }

                    let (patch_id, row, column) = Action::get_from_special_patch_placement_id(action_id);

                    debug_assert_eq!(patch_id, stored_patch_id);

                    Some(Action::new(ActionPayload::SpecialPatchPlacement {
                        patch_id,
                        row,
                        column,
                        next_quilt_board: Some(next_quilt_board),
                        previous_quilt_board: Some(previous_quilt_board),
                    }))
                }
                _ => None, // data & extra_data mismatch
            },
            Action::PATCH_PLACEMENT_ID_START..=Action::PATCH_PLACEMENT_ID_END => match extra_data {
                EntryExtraData::PatchPlacement {
                    action_id: stored_action_id,
                    patch,
                    starting_index,
                    next_quilt_board,
                    previous_quilt_board,
                } => {
                    if action_id != stored_action_id {
                        return None; // data & extra_data mismatch
                    }

                    let (patch_index, row, column, patch_rotation, patch_orientation) =
                        Action::get_from_patch_placement_id(action_id);

                    Some(Action::new(ActionPayload::PatchPlacement {
                        patch,
                        starting_index,
                        patch_index,
                        patch_rotation,
                        patch_orientation,
                        row,
                        column,
                        next_quilt_board: Some(next_quilt_board),
                        previous_quilt_board: Some(previous_quilt_board),
                    }))
                }
                _ => None, // data & extra_data mismatch
            },
            _ => None, // invalid action_id
        }
    }
}
