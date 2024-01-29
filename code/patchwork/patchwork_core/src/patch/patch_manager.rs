use lazy_static::lazy_static;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

use patchwork_macros::generate_patches;

use crate::patch::{Patch, PatchTransformation};

pub struct PatchManager {
    /// The patches.
    pub patches: [Patch; Self::AMOUNT_OF_PATCHES as usize],
    /// The tiles of every patch.
    pub tiles: [Vec<Vec<u8>>; Self::AMOUNT_OF_PATCHES as usize],
    /// The normalized tiles of every patch.
    pub normalized_tiles: [[[u8; 5]; 3]; Self::AMOUNT_OF_PATCHES as usize],
    /// The different ways this patch can be placed on the board.
    pub transformations: [Vec<PatchTransformation>; Self::AMOUNT_OF_PATCHES as usize],
}

impl PatchManager {
    /// The amount of starting patches in the game.
    pub const AMOUNT_OF_STARTING_PATCHES: u8 = 1;
    /// The amount of patches in the game that are not special and not a starting patch
    pub const AMOUNT_OF_NON_STARTING_PATCHES: u8 = 32;
    /// The amount of special patches in the game.
    pub const AMOUNT_OF_SPECIAL_PATCHES: u8 = 5;
    /// The amount of normal patches in the game. All patches except the special patches.
    pub const AMOUNT_OF_NORMAL_PATCHES: u8 = Self::AMOUNT_OF_STARTING_PATCHES + Self::AMOUNT_OF_NON_STARTING_PATCHES;
    /// The amount of all patches in the game.
    pub const AMOUNT_OF_PATCHES: u8 = Self::AMOUNT_OF_NORMAL_PATCHES + Self::AMOUNT_OF_SPECIAL_PATCHES;

    /// The maximum amount of transformations a patch can have.
    pub const MAX_AMOUNT_OF_TRANSFORMATIONS: u32 = 448;
    /// The maximum amount of tiles a player can chose from all tiles.
    pub const MAX_AMOUNT_OF_CHOOSABLE_TILES: u32 = 3;

    /// Gets the instance of the patch manager.
    ///
    /// # Returns
    ///
    /// * The instance of the patch manager
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub fn get_instance() -> &'static Self {
        &INSTANCE
    }

    /// Gets the patch with the given id.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch.
    ///
    /// # Returns
    ///
    /// The patch with the given id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub fn get_patch(patch_id: u8) -> &'static Patch {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager::get_patch] Invalid patch id"
        );
        &PatchManager::get_instance().patches[patch_id as usize]
    }

    /// Gets the transformation of the patch with the given id.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch.
    ///
    /// # Returns
    ///
    /// The transformations of the patch with the given id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_transformations(patch_id: u8) -> &'static Vec<PatchTransformation> {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager::get_transformations] Invalid patch id"
        );
        &PatchManager::get_instance().transformations[patch_id as usize]
    }

    /// Gets the transformation of the patch with the given id and transformation index.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch.
    /// * `patch_transformation_index` - The index of the transformation.
    ///
    /// # Returns
    ///
    /// The transformation of the patch with the given id and transformation index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    #[rustfmt::skip]
    pub fn get_transformation(patch_id: u8, patch_transformation_index: u16) -> &'static PatchTransformation {
        let transformations = PatchManager::get_transformations(patch_id);

        debug_assert!((patch_transformation_index as usize) < transformations.len(), "[PatchManager::get_transformations] Invalid patch transformation index");

        &transformations[patch_transformation_index as usize]
    }

    ///  Generates all patches in the game (excluding special patches) and shuffles them randomly.
    ///
    /// # Arguments
    ///
    /// * `seed` - The seed to use for the random shuffle. If None, no seed is used.
    ///
    /// # Returns
    ///
    /// A list of all patches in the game (excluding special patches) in a random order.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the amount of patches (33)
    pub fn generate_patches(seed: Option<u64>) -> Vec<&'static Patch> {
        const PATCH_AMOUNT: usize = PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize;

        let mut patches = Vec::with_capacity(PATCH_AMOUNT);
        for patch in &PatchManager::get_instance().patches[(Self::AMOUNT_OF_STARTING_PATCHES) as usize
            ..(Self::AMOUNT_OF_STARTING_PATCHES + Self::AMOUNT_OF_NON_STARTING_PATCHES) as usize]
        {
            patches.push(patch);
        }

        if let Some(seed) = seed {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
            patches.shuffle(&mut rng);
        } else {
            patches.shuffle(&mut thread_rng());
        }
        patches.push(PatchManager::get_starting_patch());
        patches
    }

    ///  Gets the special patch with the given index.
    ///
    /// # Remarks
    ///
    /// Reverse function of `get_position_from_special_patch_id`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the special patch.
    ///
    /// # Returns
    ///
    /// The special patch with the given index.
    ///
    /// # Panics
    ///
    /// If the index is not a valid special patch index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_special_patch(index: usize) -> &'static Patch {
        let mapped_index = match index {
            26 => 0,
            32 => 1,
            38 => 2,
            44 => 3,
            50 => 4,
            _ => panic!("[PatchManager::get_special_patch] Invalid special patch index!"),
        };
        let id = (Self::AMOUNT_OF_STARTING_PATCHES + Self::AMOUNT_OF_NON_STARTING_PATCHES) as usize + mapped_index;
        &PatchManager::get_instance().patches[id]
    }

    /// Gets the position of the special patch with the given id.
    ///
    /// # Remarks
    ///
    /// Reverse function of `get_special_patch`.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the special patch.
    ///
    /// # Returns
    ///
    /// The position of the special patch with the given id.
    ///
    /// # Panics
    ///
    /// If the patch id is not a valid special patch id.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_position_from_special_patch_id(&self, patch_id: usize) -> usize {
        match patch_id {
            33 => 26,
            34 => 32,
            35 => 38,
            36 => 44,
            37 => 50,
            _ => panic!("[PatchManager::get_position_from_special_patch_id] Invalid special patch id!"),
        }
    }

    /// Gets the tiles of the patch with the given id.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch
    ///
    /// # Returns
    ///
    /// * The tiles of the patch
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_tiles(patch_id: u8) -> &'static Vec<Vec<u8>> {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager::get_tiles] Invalid patch id"
        );
        &PatchManager::get_instance().tiles[patch_id as usize]
    }

    /// Gets the normalized tiles (fit into 5x3) of the patch with the given id.
    ///
    /// # Arguments
    ///
    /// * `patch_id` - The id of the patch
    ///
    /// # Returns
    ///
    /// * The normalized tiles of the patch
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_normalized_tiles(patch_id: u8) -> &'static [[u8; 5]; 3] {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager::get_normalized_tiles] Invalid patch id"
        );
        &PatchManager::get_instance().normalized_tiles[patch_id as usize]
    }

    /// Returns the starting patch.
    ///
    /// # Returns
    ///
    /// The starting patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_starting_patch() -> &'static Patch {
        &PatchManager::get_instance().patches[0]
    }

    /// Generates all patches in the game (excluding the special as well as the starting patches).
    ///
    /// # Returns
    ///
    /// A list of all patches in the game (excluding the special as well as the starting patches).
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_normal_patches() -> Vec<&'static Patch> {
        PatchManager::get_instance().patches[(Self::AMOUNT_OF_STARTING_PATCHES as usize)
            ..(Self::AMOUNT_OF_STARTING_PATCHES + Self::AMOUNT_OF_NON_STARTING_PATCHES) as usize]
            .iter()
            .collect()
    }
}

lazy_static! {
    static ref INSTANCE: PatchManager = generate_patches!(
        // starting patch
        patch(
            id: 0,
            button_cost: 2,
            time_cost: 1,
            button_income: 0,
            tiles:[
                [1, 1]
            ]
        ),
        // normal patches
        patch(
            id: 1,
            button_cost: 10,
            time_cost: 4,
            button_income: 3,
            tiles:[
                [1,0,0],
                [1,1,0],
                [0,1,1],
            ]
        ),
        patch(
            id: 2,
            button_cost: 5,
            time_cost: 3,
            button_income: 1,
            tiles:[
                [0,1,1,1,0],
                [1,1,1,1,1],
                [0,1,1,1,0]
            ]
        ),
        patch(
            id: 3,
            button_cost: 8,
            time_cost: 6,
            button_income: 3,
            tiles:[
                [0,1,1],
                [0,1,1],
                [1,1,0]
            ]
        ),
        patch(
            id: 4,
            button_cost: 7,
            time_cost: 6,
            button_income: 3,
            tiles:[
                [0,1,1],
                [1,1,0]
            ]
        ),
        patch(
            id: 5,
            button_cost: 4,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [1,0],
                [1,1],
                [1,1],
                [0,1]
            ]
        ),
        patch(
            id: 6,
            button_cost: 2,
            time_cost: 1,
            button_income: 0,
            tiles:[
                [0,1,0],
                [0,1,1],
                [1,1,0],
                [0,1,0]
            ]
        ),
        patch(
            id: 7,
            button_cost: 2,
            time_cost: 3,
            button_income: 0,
            tiles:[
                [1,0,1],
                [1,1,1],
                [1,0,1]
            ]
        ),
        patch(
            id: 8,
            button_cost: 2,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [1,0],
                [1,1],
                [1,1]
            ]
        ),
        patch(
            id: 9,
            button_cost: 6,
            time_cost: 5,
            button_income: 2,
            tiles:[
                [1,1],
                [1,1]
            ]
        ),
        patch(
            id: 10,
            button_cost: 2,
            time_cost: 3,
            button_income: 1,
            tiles:[
                [0,1],
                [0,1],
                [1,1],
                [1,0]
            ]
        ),
        patch(
            id: 11,
            button_cost: 1,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [0,0,0,1],
                [1,1,1,1],
                [1,0,0,0]
            ]
        ),
        patch(
            id: 12,
            button_cost: 10,
            time_cost: 5,
            button_income: 3,
            tiles:[
                [1,1],
                [1,1],
                [0,1],
                [0,1],
            ]
        ),
        patch(
            id: 13,
            button_cost: 7,
            time_cost: 2,
            button_income: 2,
            tiles:[
                [0,1,0],
                [0,1,0],
                [0,1,0],
                [1,1,1]
            ]
        ),
        patch(
            id: 14,
            button_cost: 4,
            time_cost: 6,
            button_income: 2,
            tiles:[
                [0,1],
                [0,1],
                [1,1]
            ]
        ),
        patch(
            id: 15,
            button_cost: 7,
            time_cost: 4,
            button_income: 2,
            tiles:[
                [0,1,1,0],
                [1,1,1,1]
            ]
        ),
        patch(
            id: 16,
            button_cost: 1,
            time_cost: 5,
            button_income: 1,
            tiles:[
                [1,1],
                [0,1],
                [0,1],
                [1,1]
            ]
        ),
        patch(
            id: 17,
            button_cost: 5,
            time_cost: 4,
            button_income: 2,
            tiles:[
                [0,1,0],
                [1,1,1],
                [0,1,0]
            ]
        ),
        patch(
            id: 18,
            button_cost: 10,
            time_cost: 3,
            button_income: 2,
            tiles:[
                [1,0,0,0],
                [1,1,1,1]
            ]
        ),
        patch(
            id: 19,
            button_cost: 4,
            time_cost: 2,
            button_income: 1,
            tiles:[
                [0,0,1],
                [1,1,1]
            ]
        ),
        patch(
            id: 20,
            button_cost: 1,
            time_cost: 4,
            button_income: 1,
            tiles:[
                [0,0,1,0,0],
                [1,1,1,1,1],
                [0,0,1,0,0]
            ]
        ),
        patch(
            id: 21,
            button_cost: 1,
            time_cost: 3,
            button_income: 0,
            tiles:[
                [0,1],
                [1,1]
            ]
        ),
        patch(
            id: 22,
            button_cost: 1,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [1,0,1],
                [1,1,1]
            ]
        ),
        patch(
            id: 23,
            button_cost: 3,
            time_cost: 1,
            button_income: 0,
            tiles:[
                [0,1],
                [1,1]
            ]
        ),
        patch(
            id: 24,
            button_cost: 2,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [0,1],
                [1,1],
                [0,1]
            ]
        ),
        patch(
            id: 25,
            button_cost: 2,
            time_cost: 2,
            button_income: 0,
            tiles:[
                [1,1,1]
            ]
        ),
        patch(
            id: 26,
            button_cost: 3,
            time_cost: 2,
            button_income: 1,
            tiles:[
                [0,1],
                [1,1],
                [1,0]
            ]
        ),
        patch(
            id: 27,
            button_cost: 7,
            time_cost: 1,
            button_income: 1,
            tiles:[
                [1,1,1,1,1]
            ]
        ),
        patch(
            id: 28,
            button_cost: 3,
            time_cost: 3,
            button_income: 1,
            tiles:[
                [1,1,1,1]
            ]
        ),
        patch(
            id: 29,
            button_cost: 5,
            time_cost: 5,
            button_income: 2,
            tiles:[
                [0,1,0],
                [0,1,0],
                [1,1,1]
            ]
        ),
        patch(
            id: 30,
            button_cost: 3,
            time_cost: 6,
            button_income: 2,
            tiles:[
                [0,1,0],
                [1,1,1],
                [1,0,1]
            ]
        ),
        patch(
            id: 31,
            button_cost: 3,
            time_cost: 4,
            button_income: 1,
            tiles:[
                [0,0,1,0],
                [1,1,1,1]
            ]
        ),
        patch(
            id: 32,
            button_cost: 0,
            time_cost: 3,
            button_income: 1,
            tiles:[
                [0,1,0,0],
                [1,1,1,1],
                [0,1,0,0],
            ]
        ),
        // special patches
        patch(
            id: 33,
            button_cost: 0,
            time_cost: 0,
            button_income: 0,
            tiles: [
                [1]
            ]
        ),
        patch(
            id: 34,
            button_cost: 0,
            time_cost: 0,
            button_income: 0,
            tiles: [
                [1]
            ]
        ),
        patch(
            id: 35,
            button_cost: 0,
            time_cost: 0,
            button_income: 0,
            tiles: [
                [1]
            ]
        ),
        patch(
            id: 36,
            button_cost: 0,
            time_cost: 0,
            button_income: 0,
            tiles: [
                [1]
            ]
        ),
        patch(
            id: 37,
            button_cost: 0,
            time_cost: 0,
            button_income: 0,
            tiles: [
                [1]
            ]
        )
    );
}
