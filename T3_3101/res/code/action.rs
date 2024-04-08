#[derive(Debug, Clone, PartialEq, Eq, Hash, ...)]
pub enum Action {
    Walking { starting_index: u8 },
    PatchPlacement {
        patch_id: u8,
        patch_index: u8,
        patch_transformation_index: u16,
        previous_player_was_1: bool,
    },
    SpecialPatchPlacement { quilt_board_index: u8 },
    Phantom,
    Null,
}
