use patchwork_core::ActionId;

use crate::{
    ActionSorter, PATCH_PLACEMENT_ENDGAME_TABLE, PATCH_PLACEMENT_OPENING_TABLE, SPECIAL_PATCH_PLACEMENT_ENDGAME_TABLE,
    SPECIAL_PATCH_PLACEMENT_OPENING_TABLE, WALKING_ENDGAME_TABLE, WALKING_OPENING_TABLE,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TableActionSorter;

const AVG_PATCHWORK_PLIES: f64 = 42.8176;

fn interpolate(ply: usize, opening_value: f64, endgame_value: f64) -> f64 {
    let ratio = ply as f64 / AVG_PATCHWORK_PLIES;

    opening_value * (1.0 - ratio) + endgame_value * ratio
}

impl ActionSorter for TableActionSorter {
    fn score_action(&self, action: ActionId, pv_action: Option<ActionId>, current_ply: usize) -> f64 {
        if pv_action.is_some() && action == pv_action.unwrap() {
            return 10000.0;
        }

        if action.is_walking() {
            return interpolate(current_ply, WALKING_OPENING_TABLE, WALKING_ENDGAME_TABLE);
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

        0.0
    }
}

impl Default for TableActionSorter {
    fn default() -> Self {
        Self
    }
}

// // TODO: write something like this for the real sorter
// #[cfg(feature = "performance_tests")]
// mod performance_tests {
//     use super::*;
//     use patchwork_core::Patchwork;
//     use std::time::Instant;

//     #[test]
//     fn sort_actions() {
//         let action_orderer = TableActionSorter;
//         let game = Patchwork::get_initial_state(None);
//         let mut actions = game.get_valid_actions();

//         let start = Instant::now();
//         action_orderer.sort_actions(&mut actions, None);
//         let end = Instant::now();
//         println!("Sorting took: {:?}", end - start);
//     }
// }
