use patchwork_core::{PatchManager, Patchwork, PlayerState, QuiltBoard, TimeBoard};

/// A Zobrist hash implementation for Patchwork.
/// This is used to hash the game state.
///
/// The Zobrist hash needs at least ≈16.68 kiB of memory to store the random numbers.
/// This is calculated as follows:
///
/// ```math
/// 33 * 33 + // a hash for every of the 33 patches at each possible position
/// 5       + // a hash for every of the 5 special patches
/// 1         // the hash if it is player 2's turn
/// 81  * 2 + // every piece on the quilt board for both players
/// 54  * 2 + // every position on the time board for both players
/// 353 * 2 + // every button balance for both players
/// 32  * 2   // every button income for both players
/// = 2135 u64s
///
/// 2135 * 8 = 17080 bytes // each u64 is 8 bytes
/// 17080 bytes / 1024 = 16.6796875 kiB
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    /// A table of random numbers for each patch.
    zobrist_patches_table:
        [u64; (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize) * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize)],
    /// A table of random number for each special patch.
    zobrist_special_patches_table: [u64; PatchManager::AMOUNT_OF_SPECIAL_PATCHES as usize],
    /// A random number if it is player 2's turn.
    zobrist_player_2_to_move: u64,
    /// A table of random numbers for each tile on the board for player 1.
    zobrist_player_1_quilt_board_table: [u64; QuiltBoard::TILES as usize],
    /// A table of random numbers for each position on the time board for player 1.
    zobrist_player_1_position_table: [u64; TimeBoard::MAX_POSITION as usize + 1], // +1 to include 0 and the maximum position
    /// A table of random numbers for each button balance for player 1.
    zobrist_player_1_button_balance_table: [u64; Self::MAX_BUTTON_BALANCE + 1], // also +1
    /// A table of random numbers for each button income for player 1.
    zobrist_player_1_button_income_table: [u64; Self::MAX_BUTTON_INCOME + 1], // also +1
    /// A table of random numbers for each tile on the board for player 2.
    zobrist_player_2_quilt_board_table: [u64; QuiltBoard::TILES as usize],
    /// A table of random numbers for each position on the time board for player 2.
    zobrist_player_2_position_table: [u64; TimeBoard::MAX_POSITION as usize + 1], // also +1
    /// A table of random numbers for each button balance for player 2.
    zobrist_player_2_button_balance_table: [u64; Self::MAX_BUTTON_BALANCE + 1], // also +1
    /// A table of random numbers for each button income for player 2.
    zobrist_player_2_button_income_table: [u64; Self::MAX_BUTTON_INCOME + 1], // also +1
}

impl ZobristHash {
    /// The amount of patches the zobrist hash uses to hash from all the available patches.
    ///
    /// Technically 3 this is incorrect as this can result in the same hash for different games
    /// when the patches after the first 3 patches are different.
    /// This is counted as a optimization that we have to live with.
    ///
    /// TODO: Test how this behaves if we hash more of the patches as this will result in a more accurate hash.
    /// 3 is the minimum possible as otherwise the best actions could be incorrect
    pub const AMOUNT_OF_PATCHES: usize =
        PatchManager::AMOUNT_OF_STARTING_PATCHES as usize + PatchManager::AMOUNT_OF_NON_STARTING_PATCHES as usize;

    /// The maximum button balance a player can have is bounded by the game.
    ///
    /// * A player has `5` buttons at the start of the game.
    /// * There are only `9` button income triggers that can yield a maximum amount
    ///   of `32` buttons each (see `MAX_BUTTON_INCOME` estimate below).
    /// * The player can get `1` button income for every tile he walks on the
    ///   time track with the walking action. There are `54` tiles on the time track.
    /// * The only other income source are the `7` buttons from a full quilt board.
    ///
    /// Because of this the maximum button balance a player can have is bounded
    /// by `5 + 9 · 32 + 7 + 53 · 1 = 353`. This is a upper bound and not the
    /// actual maximum because of the same reason as the `MAX_BUTTON_INCOME`
    /// estimate below. Furthermore the player can only choose between the
    /// walking action and the action to place a tile. Therefore he cannot get
    /// both at the same time. It would probably be possible to lower the bound
    /// to `5 + 9 · 32 + 7 + 54 · 1 = 300` (remove the walking actions) and
    /// still be correct. But to be safe the bound is kept at `353`.
    pub const MAX_BUTTON_BALANCE: usize = PlayerState::STARTING_BUTTON_BALANCE as usize
        + TimeBoard::AMOUNT_OF_BUTTON_INCOME_TRIGGERS * Self::MAX_BUTTON_INCOME
        + QuiltBoard::FULL_BOARD_BUTTON_INCOME as usize
        + TimeBoard::MAX_POSITION as usize;

    /// The maximum amount of button income a player can have is bounded
    /// by the number of tiles in the quilt board.
    ///
    /// The highest possible upper bound that is realistic would therefore be `81`
    /// (The amount of tiles in the quilt board). But this would require that
    /// all patches that are layed out have at least the same button income as
    /// the amount of tiles they cover. This is not true for any patch in the
    /// game.
    ///
    /// Therefore variable uses a more conservative upper bound of `32`.
    /// For this the patches were ordered by the percentage of button income in
    /// relation to the amount of tiles they cover. Then the first patches were
    /// chosen until the amount of tiles covered was `>= 81`. With this a
    /// maximum button income of 33 as upper bound was found.
    ///
    /// Here is the list of all patches and the patches that were chosen ordered
    /// by the percentage of button income in relation to the amount of tiles:
    ///
    /// ```txt
    /// index:  4, tiles: 4, buttons: 3, percentage: 0.75
    /// index:  1, tiles: 5, buttons: 3, percentage: 0.6
    /// index:  3, tiles: 6, buttons: 3, percentage: 0.5
    /// index:  9, tiles: 4, buttons: 2, percentage: 0.5
    /// index: 12, tiles: 6, buttons: 3, percentage: 0.5
    /// index: 14, tiles: 4, buttons: 2, percentage: 0.5
    /// index: 17, tiles: 5, buttons: 2, percentage: 0.4
    /// index: 18, tiles: 5, buttons: 2, percentage: 0.4
    /// index: 29, tiles: 5, buttons: 2, percentage: 0.4
    /// index: 13, tiles: 6, buttons: 2, percentage: 0.3333333333333333
    /// index: 15, tiles: 6, buttons: 2, percentage: 0.3333333333333333
    /// index: 30, tiles: 6, buttons: 2, percentage: 0.3333333333333333
    /// index: 19, tiles: 4, buttons: 1, percentage: 0.25
    /// index: 26, tiles: 4, buttons: 1, percentage: 0.25
    /// index: 28, tiles: 4, buttons: 1, percentage: 0.25
    /// index: 10, tiles: 5, buttons: 1, percentage: 0.2
    /// index: 27, tiles: 5, buttons: 1, percentage: 0.2
    /// --------------- CUTOFF AFTER 84 >= 81 TILES COVERED ---------------
    /// index: 31, tiles: 5, buttons: 1, percentage: 0.2
    /// index: 16, tiles: 6, buttons: 1, percentage: 0.16666666666666666
    /// index: 32, tiles: 6, buttons: 1, percentage: 0.16666666666666666
    /// index: 20, tiles: 7, buttons: 1, percentage: 0.14285714285714285
    /// index:  2, tiles: 8, buttons: 1, percentage: 0.125
    /// index:  0, tiles: 2, buttons: 0, percentage: 0
    /// index:  5, tiles: 6, buttons: 0, percentage: 0
    /// index:  6, tiles: 6, buttons: 0, percentage: 0
    /// index:  7, tiles: 7, buttons: 0, percentage: 0
    /// index:  8, tiles: 5, buttons: 0, percentage: 0
    /// index: 11, tiles: 6, buttons: 0, percentage: 0
    /// index: 21, tiles: 3, buttons: 0, percentage: 0
    /// index: 22, tiles: 5, buttons: 0, percentage: 0
    /// index: 23, tiles: 3, buttons: 0, percentage: 0
    /// index: 24, tiles: 4, buttons: 0, percentage: 0
    /// index: 25, tiles: 3, buttons: 0, percentage: 0
    /// ```
    ///
    /// But this is not the least upper bound (supremum) as the tiles covered
    /// are `84` in the end and not `81`. The actual supremum is a button
    /// income of `32`. This is the case because the most one can cover below
    /// the limit of 81 tiles is reached is only a button income of `32`.
    /// Then at least 79 tiles are covered and all patches with 2 tiles or less
    /// do not have any button income. To show that this is actually not only
    /// the supremum but a reachable maximum a quilt board has to be constructed
    /// that has a button income of 32. This is done in the following:
    ///
    /// ```txt
    /// 28 28 28 28 10 10 10 XX 13
    /// 19 19 19 10 10 13 13 13 13
    /// 19 04 18 18 18 18 12 12 13
    /// 04 04 18 XX 12 12 12 12 14
    /// 04 30 29 29 29 17 14 14 14
    /// 30 30 30 29 17 17 17 03 03
    /// 30 26 30 29 01 17 03 03 03
    /// 26 26 15 15 01 01 03 09 09
    /// 26 15 15 15 15 01 01 09 09
    ///
    /// where the tiles covered with XX are still free and all the other tiles
    /// have the id of the patch that covers them.
    /// ```
    ///
    /// This quilt board has a button income of 32, has a time cost of 66 and
    /// covers at least 79 tiles but can be filled up with two special patches
    /// to cover the full quilt board. The board has a button income to
    /// percentage ratio of `32 / 81 ≈ 39,5%` which is the highest possible.
    /// While this is a maximum that can be created on the quilt board, it is
    /// not achievable in the game as the time cost of 66 is greater than the
    /// allowed time cost of 54.
    ///
    /// TODO: improve the bound even more by only allowing patches that fall
    /// within the time cost limit of 54.
    pub const MAX_BUTTON_INCOME: usize = 32;

    /// Creates a new Zobrist hash.
    ///
    /// This is done by generating random numbers for each part of the game state.
    /// Including:
    /// * The patches that are available (at least the first 3 to guarantee correct actions)
    /// * The quilt boards of both players
    /// * The button balance of both players
    /// * The button income of both players
    /// * The player whose turn it is
    ///
    /// The random numbers are generated using the [`rand`] crate.
    ///
    /// # Returns
    ///
    /// A new Zobrist hash struct.
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        // TODO: maybe we can reduce the amount of memory required by using 2 rank 1 vectors and then using the dot product to create a matrix on the fly?
        let mut zobrist_patches_table =
            [0; (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize) * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize)];
        let mut zobrist_special_patches_table = [0; PatchManager::AMOUNT_OF_SPECIAL_PATCHES as usize];
        let mut zobrist_player_1_quilt_board_table = [0; QuiltBoard::TILES as usize];
        let mut zobrist_player_1_position_table = [0; TimeBoard::MAX_POSITION as usize + 1];
        let mut zobrist_player_1_button_balance_table = [0; Self::MAX_BUTTON_BALANCE + 1];
        let mut zobrist_player_1_button_income_table = [0; Self::MAX_BUTTON_INCOME + 1];
        let mut zobrist_player_2_quilt_board_table = [0; QuiltBoard::TILES as usize];
        let mut zobrist_player_2_position_table = [0; TimeBoard::MAX_POSITION as usize + 1];
        let mut zobrist_player_2_button_balance_table = [0; Self::MAX_BUTTON_BALANCE + 1];
        let mut zobrist_player_2_button_income_table = [0; Self::MAX_BUTTON_INCOME + 1];

        for i in
            0..((PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize) * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize))
        {
            zobrist_patches_table[i] = rand::random();
        }

        for i in 0..PatchManager::AMOUNT_OF_SPECIAL_PATCHES as usize {
            zobrist_special_patches_table[i] = rand::random();
        }

        for i in 0..QuiltBoard::TILES as usize {
            zobrist_player_1_quilt_board_table[i] = rand::random();
            zobrist_player_2_quilt_board_table[i] = rand::random();
        }

        for i in 0..TimeBoard::MAX_POSITION as usize + 1 {
            zobrist_player_1_position_table[i] = rand::random();
            zobrist_player_2_position_table[i] = rand::random();
        }

        for i in 0..=Self::MAX_BUTTON_BALANCE {
            zobrist_player_1_button_balance_table[i] = rand::random();
            zobrist_player_2_button_balance_table[i] = rand::random();
        }

        for i in 0..=Self::MAX_BUTTON_INCOME {
            zobrist_player_1_button_income_table[i] = rand::random();
            zobrist_player_2_button_income_table[i] = rand::random();
        }

        Self {
            zobrist_player_2_to_move: rand::random(),
            zobrist_patches_table,
            zobrist_special_patches_table,
            zobrist_player_1_quilt_board_table,
            zobrist_player_1_position_table,
            zobrist_player_1_button_balance_table,
            zobrist_player_1_button_income_table,
            zobrist_player_2_quilt_board_table,
            zobrist_player_2_position_table,
            zobrist_player_2_button_balance_table,
            zobrist_player_2_button_income_table,
        }
    }

    /// Hashes the given game state.
    /// This is done by xoring the random numbers of the zobrist hash with the current game state.
    /// This is done for every part of the game state.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to hash.
    ///
    /// # Returns
    ///
    /// The hash of the given game state.
    pub fn hash(&self, game: &Patchwork) -> u64 {
        let mut hash = 0;

        // Hash the next patches
        // cannot hash more patches than there are
        for position_index in 0..(Self::AMOUNT_OF_PATCHES.min(game.patches.len())) {
            let patch_index = game.patches[position_index].id as usize;

            let index = patch_index * (PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize) + position_index;

            hash ^= self.zobrist_patches_table[index];
        }

        // Hash special patches on board
        for (index, special_patch_index) in [26, 32, 38, 44, 50].iter().enumerate() {
            if game.time_board.is_special_patch_at(*special_patch_index) {
                hash ^= self.zobrist_special_patches_table[index];
            }
        }

        // Hash the player turn
        if game.is_player_2() {
            hash ^= self.zobrist_player_2_to_move;
        }

        // Hash player quilt boards
        for row in 0..QuiltBoard::ROWS {
            for column in 0..QuiltBoard::COLUMNS {
                let index = row * QuiltBoard::COLUMNS + column;

                if game.player_1.quilt_board.get_at(index) {
                    hash ^= self.zobrist_player_1_quilt_board_table[index as usize];
                }

                if game.player_2.quilt_board.get_at(index) {
                    hash ^= self.zobrist_player_2_quilt_board_table[index as usize];
                }
            }
        }

        // Hash position, button income and button balance of both players
        let player_1 = &game.player_1;
        let player_2 = &game.player_2;
        hash ^= self.zobrist_player_1_position_table[player_1.get_position() as usize];
        hash ^= self.zobrist_player_1_button_balance_table[player_1.button_balance as usize];
        hash ^= self.zobrist_player_1_button_income_table[player_1.quilt_board.button_income as usize];
        hash ^= self.zobrist_player_2_position_table[player_2.get_position() as usize];
        hash ^= self.zobrist_player_2_button_balance_table[player_2.button_balance as usize];
        hash ^= self.zobrist_player_2_button_income_table[player_2.quilt_board.button_income as usize];

        hash
    }
}

impl Default for ZobristHash {
    fn default() -> Self {
        Self::new()
    }
}
