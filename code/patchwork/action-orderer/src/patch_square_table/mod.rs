#![allow(clippy::unreadable_literal)]

mod patch_placement;
mod special_patch_placement;
mod walking;

#[rustfmt::skip]
pub use walking::{
    OPENING_TABLE as WALKING_OPENING_TABLE,
    ENDGAME_TABLE as WALKING_ENDGAME_TABLE,
};
#[rustfmt::skip]
pub use special_patch_placement::{
    OPENING_TABLE as SPECIAL_PATCH_PLACEMENT_OPENING_TABLE,
    ENDGAME_TABLE as SPECIAL_PATCH_PLACEMENT_ENDGAME_TABLE,
};
#[rustfmt::skip]
pub use patch_placement::{
    OPENING_TABLE as PATCH_PLACEMENT_OPENING_TABLE,
    ENDGAME_TABLE as PATCH_PLACEMENT_ENDGAME_TABLE,
};
