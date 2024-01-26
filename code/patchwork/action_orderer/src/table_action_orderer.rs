use patchwork_core::ActionId;

use crate::{
    ActionSorter, PATCH_PLACEMENT_ENDGAME_TABLE, PATCH_PLACEMENT_OPENING_TABLE, SPECIAL_PATCH_PLACEMENT_ENDGAME_TABLE,
    SPECIAL_PATCH_PLACEMENT_OPENING_TABLE, WALKING_ENDGAME_TABLE, WALKING_OPENING_TABLE,
};

/// An ActionSorter that uses a table of weights to score actions.
///
/// The implementation scores Actions by the following:
/// 1. PV-Action: If an action is the pv action is gets the highest score.
/// 2. Score via weights from a table.
///
/// The weight table is generated from a recording of 10.000.000 independently
/// uniform random sampled games. The weights are divided into opening and
/// endgame weights. The opening weights are used for the start of the game and
/// are linearly interpolated with the endgame weights. As linear interpolation
/// the current ply is used and divided by the average amount of plies from the
/// recorded games. The average amount of plies is 42.8176.
///
/// The walking action is penalized in addition to the normal weight the later
/// the game is.
///
/// # FEATURE: other features that could be implemented
/// - Killer Moves
/// - History Heuristic
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableActionOrderer;

const AVG_PATCHWORK_PLIES: f64 = 42.8176;

fn interpolate(ply: usize, opening_value: f64, endgame_value: f64) -> f64 {
    let ratio = (ply as f64 / AVG_PATCHWORK_PLIES).max(1.0);

    opening_value * (1.0 - ratio) + endgame_value * ratio
}

impl ActionSorter for TableActionOrderer {
    fn score_action(&self, action: ActionId, pv_action: Option<ActionId>, current_ply: usize) -> f64 {
        if pv_action.is_some() && action == pv_action.unwrap() {
            return 10000.0;
        }

        if action.is_walking() {
            return interpolate(current_ply, WALKING_OPENING_TABLE, WALKING_ENDGAME_TABLE)
                + interpolate(current_ply, 0.0, -1.0);
        }

        if action.is_special_patch_placement() {
            return interpolate(
                current_ply,
                SPECIAL_PATCH_PLACEMENT_OPENING_TABLE[action.get_quilt_board_index() as usize],
                SPECIAL_PATCH_PLACEMENT_ENDGAME_TABLE[action.get_quilt_board_index() as usize],
            );
        }

        if action.is_patch_placement() {
            let id = action.get_patch_id() as usize;
            let transformation = action.get_patch_transformation_index() as usize;

            return interpolate(
                current_ply,
                PATCH_PLACEMENT_OPENING_TABLE[id][transformation],
                PATCH_PLACEMENT_ENDGAME_TABLE[id][transformation],
            );
        }

        unreachable!("[TableActionSorter::score_action] Unknown action type");
    }
}

impl Default for TableActionOrderer {
    fn default() -> Self {
        Self
    }
}
