use patchwork_core::{evaluator_constants, ActionId};

use super::EvaluationType;

/// An entry in the transposition table. The table uses
/// [Lockless Hashing](https://www.chessprogramming.org/Shared_Hash_Table#Lock-less)
/// with XOR for the entries.
///
/// Furthermore an age is stored to determine when to overwrite entries from
/// searching previous positions during the game.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Entry {
    /// The key of this entry.
    /// This is the zobrist hash of the position XORed with the data.
    pub key: u64,
    /// The data of this entry. Consists of
    /// - `depth`           (8 Bits): The depth at which this entry was stored
    /// - `evaluation`      (37 Bits): The evaluation of the position
    /// - `evaluation_type` (2 Bits): The type of node this entry is
    /// - `best_action`     (17 Bits): The best action to take in this position-
    ///
    /// The data is stored as `MSB evaluation(37 Bits)|action_id(17 Bits)|depth(8 Bits)|evaluation_type(2 Bits) LSB`
    pub data: u64,
    /// The age of this entry. Used to determine when to overwrite entries from
    /// searching previous positions during the game
    pub age: usize,
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
    pub fn get_evaluation_type(data: u64) -> EvaluationType {
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
    pub fn get_depth(data: u64) -> usize {
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
    pub fn get_action_id(data: u64) -> ActionId {
        ActionId::from_bits(((data >> 10) & 0x1FFFF) as u32)
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
    pub fn get_evaluation(data: u64) -> i32 {
        let extracted = (data >> 27) & 0x1FFFFFFFFF;

        (unsafe { std::mem::transmute::<u64, i64>(extracted) }) as i32 + evaluator_constants::NEGATIVE_INFINITY
    }

    /// Unpacks the data field into the evaluation, evaluation type, depth and
    /// action.
    ///
    /// # Arguments
    ///
    /// * `data` - The data field of the entry
    ///
    /// # Returns
    ///
    /// * `Some((depth, evaluation, evaluation_type, action))` - The unpacked data
    pub fn unpack_data(data: u64) -> (usize, i32, EvaluationType, ActionId) {
        let evaluation_type = Self::get_evaluation_type(data);
        let depth = Self::get_depth(data);
        let action_id = Self::get_action_id(data);
        let evaluation = Self::get_evaluation(data);

        (depth, evaluation, evaluation_type, action_id)
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
    /// The packed data and extra data
    #[rustfmt::skip]
    pub fn pack_data(
        depth: usize,
        evaluation: i32,
        evaluation_type: EvaluationType,
        action_id: ActionId,
    ) -> u64 {
        // Force evaluation to be positive to allow reconstruction of the number later on
        let adjusted_evaluation = (evaluation - evaluator_constants::NEGATIVE_INFINITY) as i64;
        let mut data = 0u64;
        data |=  evaluation_type     as u64;        //  2 bits for evaluation type
        data |= (depth               as u64) << 2;  //  8 bits for depth     (as a max depth of 256 is used)
        data |= (action_id.as_bits() as u64) << 10; // 17 bits for action id (as a max of 2026 actions are possible)
        data |= unsafe {std::mem::transmute::<i64, u64>(adjusted_evaluation) } << 27; // 37 bits left for evaluation

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack_unpack(depth: usize, evaluation: i32, evaluation_type: EvaluationType, action_id: ActionId) {
        let data = Entry::pack_data(depth, evaluation, evaluation_type, action_id);
        let (unpacked_depth, unpacked_evaluation, unpacked_evaluation_type, unpacked_action_id) =
            Entry::unpack_data(data);

        assert_eq!(depth, unpacked_depth);
        assert_eq!(evaluation, unpacked_evaluation);
        assert_eq!(evaluation_type, unpacked_evaluation_type);
        assert_eq!(action_id, unpacked_action_id);
    }

    #[test]
    fn test_pack_unpack() {
        pack_unpack(255, 1000, EvaluationType::Exact, ActionId::walking(13));
        pack_unpack(
            3,
            -999,
            EvaluationType::UpperBound,
            ActionId::special_patch_placement(34),
        );
        pack_unpack(
            0,
            0,
            EvaluationType::LowerBound,
            ActionId::patch_placement(17, 2, 1, true),
        );
        pack_unpack(
            10,
            evaluator_constants::POSITIVE_INFINITY,
            EvaluationType::Exact,
            ActionId::null(),
        );
        pack_unpack(
            10,
            evaluator_constants::NEGATIVE_INFINITY,
            EvaluationType::LowerBound,
            ActionId::phantom(),
        );
    }
}
