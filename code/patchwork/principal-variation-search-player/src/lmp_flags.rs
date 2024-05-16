use action_orderer::ActionList;
use itertools::Itertools;
use patchwork_core::ActionId;

use crate::constants::DEFAULT_LMP_AMOUNT_OF_ACTIONS_PER_PATCH;

/// Different Flags and Actions for Late Move Pruning that are used to ensure
/// that every action type is tried at least once before pruning all other
/// actions.
#[allow(clippy::redundant_pub_crate)] // false positive
pub(crate) struct LMPFlags<const LMP_AMOUNT_OF_ACTIONS_PER_PATCH: usize = DEFAULT_LMP_AMOUNT_OF_ACTIONS_PER_PATCH> {
    walking: Option<ActionId>,
    patch1: Vec<ActionId>,
    patch2: Vec<ActionId>,
    patch3: Vec<ActionId>,
}

impl<const LMP_AMOUNT_OF_ACTIONS_PER_PATCH: usize> LMPFlags<LMP_AMOUNT_OF_ACTIONS_PER_PATCH> {
    pub const LMP_AMOUNT_OF_ACTIONS_PER_PATCH: usize = LMP_AMOUNT_OF_ACTIONS_PER_PATCH;

    /// Creates a fake LMP flags struct in constant time.
    ///
    /// # Returns
    ///
    /// The fake LMP flags struct.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub const fn fake() -> Self {
        Self {
            walking: None,
            patch1: vec![],
            patch2: vec![],
            patch3: vec![],
        }
    }

    /// Initializes the LMP flags from the given action list.
    ///
    /// # Arguments
    ///
    /// * `amount_of_actions_per_patch` - The amount of actions per patch.
    ///
    /// # Returns
    ///
    /// The initialized LMP flags.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the length of the action list.
    #[allow(clippy::collapsible_if)]
    #[inline]
    pub fn initialize_from(action_list: &ActionList<'_>) -> Self {
        let mut flags = Self {
            walking: None,
            patch1: Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH),
            patch2: Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH),
            patch3: Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH),
        };

        let mut patch_1_scores = Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH);
        let mut patch_2_scores = Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH);
        let mut patch_3_scores = Vec::with_capacity(Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH);

        for i in 0..action_list.len() {
            let action = action_list.get_action(i);
            let score = action_list.get_score(i);

            if action.is_walking() {
                flags.walking = Some(action);
            }

            if action.is_first_patch_taken() || action.is_special_patch_placement() {
                if flags.patch1.len() < Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH {
                    flags.patch1.push(action);
                    patch_1_scores.push(score);
                } else {
                    let min_index = patch_1_scores.iter().position_min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

                    if score > patch_1_scores[min_index] {
                        flags.patch1[min_index] = action;
                        patch_1_scores[min_index] = score;
                    }
                }
            }
            if action.is_second_patch_taken() {
                if flags.patch2.len() < Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH {
                    flags.patch2.push(action);
                    patch_2_scores.push(score);
                } else {
                    let min_index = patch_2_scores.iter().position_min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

                    if score > patch_2_scores[min_index] {
                        flags.patch2[min_index] = action;
                        patch_2_scores[min_index] = score;
                    }
                }
            }
            if action.is_third_patch_taken() {
                if flags.patch3.len() < Self::LMP_AMOUNT_OF_ACTIONS_PER_PATCH {
                    flags.patch3.push(action);
                    patch_3_scores.push(score);
                } else {
                    let min_index = patch_3_scores.iter().position_min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

                    if score > patch_3_scores[min_index] {
                        flags.patch3[min_index] = action;
                        patch_3_scores[min_index] = score;
                    }
                }
            }
        }

        flags
    }

    /// Gets an action as parameter and sets the flag for that action type to
    /// `None` so that the action type is done and does not prevent lmp.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to set the flag for.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[allow(dead_code)]
    pub fn set_action_type_done(&mut self, action: ActionId) {
        if action.is_walking() {
            self.walking = None;
        } else if action.is_first_patch_taken() || action.is_special_patch_placement() {
            self.patch1 = vec![];
        } else if action.is_second_patch_taken() {
            self.patch2 = vec![];
        } else if action.is_third_patch_taken() {
            self.patch3 = vec![];
        }
    }

    /// Gets the next action for the action type that is not done yet.
    ///
    /// # Returns
    ///
    /// The next action for the action type that is not done yet or `None` if
    /// all action types are done.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub fn get_next_missing(&mut self) -> Option<ActionId> {
        if let Some(action) = self.walking.take() {
            return Some(action);
        }

        if !self.patch1.is_empty() {
            return self.patch1.pop();
        }

        if !self.patch2.is_empty() {
            return self.patch2.pop();
        }

        if !self.patch2.is_empty() {
            return self.patch2.pop();
        }

        None
    }
}
