use crate::patch::{Patch, PatchTransformation};

use lazy_static::lazy_static;
use patchwork_macros::generate_patches;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;

pub struct PatchManager {
    /// The patches.
    pub patches: [Patch; Self::AMOUNT_OF_PATCHES],
    /// The tiles of every patch.
    pub tiles: [Vec<Vec<u8>>; Self::AMOUNT_OF_PATCHES],
    /// The different ways this patch can be placed on the board.
    pub transformations: [Vec<PatchTransformation>; Self::AMOUNT_OF_PATCHES],
}

impl PatchManager {
    /// The amount of starting patches in the game.
    const STARTING_PATCHES: usize = 1;

    /// The amount of normal patches in the game (not special and not starting).
    const NORMAL_PATCHES: usize = 32;

    /// The amount of special patches in the game.
    const SPECIAL_PATCHES: usize = 5;

    /// The amount of patches in the game.
    const AMOUNT_OF_PATCHES: usize =
        Self::SPECIAL_PATCHES + Self::STARTING_PATCHES + Self::NORMAL_PATCHES;

    /// Gets the instance of the patch manager.
    ///
    /// # Returns
    ///
    /// * The instance of the patch manager
    pub fn get_instance() -> &'static Self {
        &INSTANCE
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
    pub fn generate_patches(&self, seed: Option<u64>) -> Vec<&Patch> {
        let mut patches = self.get_normal_patches();
        if let Some(seed) = seed {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
            patches.shuffle(&mut rng);
        } else {
            patches.shuffle(&mut thread_rng());
        }
        patches.push(self.get_starting_patch());
        patches
    }

    ///  Gets the special patch with the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the special patch.
    ///
    /// # Returns
    ///
    /// The special patch with the given index.
    pub fn get_special_patch(&self, index: usize) -> &Patch {
        let mapped_index = match index {
            26 => 0,
            32 => 1,
            38 => 2,
            44 => 3,
            50 => 4,
            _ => panic!("[PatchManager][get_special_patch] Invalid special patch index!"),
        };
        let id = Self::STARTING_PATCHES + Self::NORMAL_PATCHES + mapped_index;
        &self.patches[id]
    }

    /// Gets the starting patch.
    ///
    /// # Arguments
    ///
    /// * `self` - The patch manager
    /// * `patch_id` - The id of the patch
    ///
    /// # Returns
    ///
    /// * The tiles of the patch
    pub fn get_tiles(&self, patch_id: usize) -> &Vec<Vec<u8>> {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager][get_tiles] Invalid patch id"
        );
        &self.tiles[patch_id]
    }

    /// Gets the transformations of the patch.
    ///
    /// # Arguments
    ///
    /// * `self` - The patch manager
    /// * `patch_id` - The id of the patch
    ///
    /// # Returns
    ///
    /// * The transformations of the patch
    pub fn get_transformations(&self, patch_id: usize) -> &Vec<PatchTransformation> {
        debug_assert!(
            patch_id < Self::AMOUNT_OF_PATCHES,
            "[PatchManager][get_transformations] Invalid patch id"
        );
        &self.transformations[patch_id]
    }

    /// Returns the starting patch.
    ///
    /// # Returns
    ///
    /// The starting patch.
    fn get_starting_patch(&self) -> &Patch {
        &self.patches[0]
    }

    /// Generates all patches in the game (excluding the special as well as the starting patches).
    ///
    /// # Returns
    ///
    /// A list of all patches in the game (excluding the special as well as the starting patches).
    fn get_normal_patches(&self) -> Vec<&Patch> {
        self.patches[Self::STARTING_PATCHES..Self::STARTING_PATCHES + Self::NORMAL_PATCHES]
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
