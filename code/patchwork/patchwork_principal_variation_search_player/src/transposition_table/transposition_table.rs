use std::sync::atomic::AtomicUsize;

use patchwork_core::{ActionId, PatchManager, Patchwork, QuiltBoard};

use crate::{Entry, EvaluationType, Size, ZobristHash};

use super::TranspositionTableDiagnostics;

/// A transposition table for storing evaluations of positions.
///
/// This is used to store evaluations of positions so that they do not have to
/// be recalculated. This is especially useful for positions that are reached
/// multiple times during the search.
///
/// The table uses [Lockless Hashing](https://www.chessprogramming.org/Shared_Hash_Table#Lock-less)
#[derive(Debug)]
pub(crate) struct TranspositionTable {
    pub(crate) entries: Vec<Entry>,
    pub(crate) zobrist_hash: ZobristHash,
    pub(crate) current_age: AtomicUsize,
    pub(crate) diagnostics: TranspositionTableDiagnostics,
}

impl TranspositionTable {
    /// Creates a new transposition table.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the transposition table.
    ///
    /// # Returns
    ///
    /// * `TranspositionTable` - The created transposition table.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the size of the transposition table as all entries
    /// are initialized.
    pub(crate) fn new(size: Size) -> Self {
        let size = match size {
            Size::B(size) => size as usize,
            Size::KB(size) => size as usize * 1024,
            Size::MB(size) => size as usize * 1024 * 1024,
            Size::GB(size) => size as usize * 1024 * 1024 * 1024,
            Size::KiB(size) => size as usize * 1000,
            Size::MiB(size) => size as usize * 1000 * 1000,
            Size::GiB(size) => size as usize * 1000 * 1000 * 1000,
        };
        let entries = size / std::mem::size_of::<Entry>();

        Self {
            entries: vec![Entry::default(); entries],
            zobrist_hash: ZobristHash::new(),
            current_age: AtomicUsize::new(0),
            diagnostics: TranspositionTableDiagnostics::new(entries),
        }
    }

    /// Gets the size of the transposition table in bytes.
    ///
    /// # Returns
    ///
    /// * `usize` - The size of the transposition table in bytes.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    #[allow(dead_code)] // TODO: move transposition table to own
    pub(crate) fn size(&self) -> usize {
        debug_assert_eq!(
            self.entries.len() * std::mem::size_of::<Entry>(),
            self.diagnostics.capacity.load(std::sync::atomic::Ordering::SeqCst),
        );
        std::mem::size_of::<Entry>() * self.entries.len()
    }

    /// Probes the transposition table for an evaluation.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to probe the transposition table for.
    /// * `alpha` - The alpha value of the search.
    /// * `beta` - The beta value of the search.
    /// * `depth` - The depth of the search.
    ///
    /// # Returns
    ///
    /// * `Some((ActionId, i32))` - The evaluation if it is found.
    /// * `None` - If no evaluation is found.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    pub(crate) fn probe_hash_entry(
        &self,
        game: &Patchwork,
        alpha: i32,
        beta: i32,
        depth: usize,
    ) -> Option<(ActionId, i32)> {
        self.diagnostics.increment_accesses();

        let hash = self.zobrist_hash.hash(game);
        let index = (hash % self.entries.len() as u64) as usize;

        let data = self.entries[index].data;

        // If key and data were written simultaneously by different search instances with different keys
        // this will result in a mismatch of the comparison, except the rare case of
        // (key collisions / type-1 errors](https://www.chessprogramming.org/Transposition_Table#KeyCollisions)
        let test_key = hash ^ data;
        if self.entries[index].key != test_key {
            self.diagnostics.increment_misses();
            return None;
        }

        let (table_depth, table_evaluation, table_evaluation_type, table_action) = Entry::unpack_data(data);

        // Only use stored evaluation if it has been searched to at least the
        // same depth as would be searched now
        if table_depth < depth {
            self.diagnostics.increment_misses();
            return None;
        }

        match table_evaluation_type {
            EvaluationType::Exact => Some((table_action, table_evaluation)),
            EvaluationType::UpperBound => {
                if table_evaluation <= alpha {
                    Some((table_action, alpha))
                } else {
                    self.diagnostics.increment_misses();
                    None
                }
            }
            EvaluationType::LowerBound => {
                if table_evaluation >= beta {
                    Some((table_action, beta))
                } else {
                    self.diagnostics.increment_misses();
                    None
                }
            }
        }
    }

    /// Stores an evaluation in the transposition table. Furthermore all
    /// symmetries of the game state are stored as well.
    ///
    /// This function can be used for symmetry reduction.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to store the evaluation for.
    /// * `depth` - The depth of the evaluation.
    /// * `evaluation` - The evaluation to store.
    /// * `evaluation_type` - The type of evaluation to store.
    /// * `action` - The best action to take from the game state.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `𝑚` is the amount of symmetries for the game state (bounded by 64) and
    /// `𝑛` is the amount of transformations of the patch the action is for (bounded by 448).
    pub(crate) fn store_evaluation_with_symmetries(
        &mut self,
        game: &Patchwork,
        depth: usize,
        evaluation: i32,
        evaluation_type: EvaluationType,
        action: ActionId,
    ) {
        // Symmetries that are possible for (special) patch placement:
        // - Rotate/Flip player 1 quilt board in all 8 directions
        // - Rotate/Flip player 2 quilt board in all 8 directions
        // => 64 symmetries in total

        let mut game_to_store = game.clone();

        for (player_1_rotation, player_1_flip, player_2_rotation, player_2_flip) in
            itertools::iproduct!(0..=3, 0..=1, 0..=3, 0..=1)
        {
            game_to_store.player_1.quilt_board.tiles = QuiltBoard::flip_horizontally_then_rotate_tiles(
                game.player_1.quilt_board.tiles,
                player_1_rotation,
                player_1_flip == 1,
            );
            game_to_store.player_2.quilt_board.tiles = QuiltBoard::flip_horizontally_then_rotate_tiles(
                game.player_2.quilt_board.tiles,
                player_2_rotation,
                player_2_flip == 1,
            );

            let action_to_store = get_action_to_store(
                &game_to_store,
                action,
                player_1_rotation,
                player_1_flip,
                player_2_rotation,
                player_2_flip,
            );

            if let Some(action_to_store) = action_to_store {
                self.store_evaluation(&game_to_store, depth, evaluation, evaluation_type, action_to_store);
            }
        }
    }

    /// Stores an evaluation in the transposition table.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to store the evaluation for.
    /// * `depth` - The depth of the evaluation.
    /// * `evaluation` - The evaluation to store.
    /// * `evaluation_type` - The type of evaluation to store.
    /// * `action` - The best action to take from the game state.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    #[allow(clippy::if_same_then_else)]
    pub(crate) fn store_evaluation(
        &mut self,
        game: &Patchwork,
        depth: usize,
        evaluation: i32,
        evaluation_type: EvaluationType,
        action: ActionId,
    ) {
        let hash = self.zobrist_hash.hash(game);

        let index = (hash % self.entries.len() as u64) as usize;

        let replace = if self.entries[index].key == 0 {
            // first entry in the key bucket

            self.diagnostics.increment_entries();
            true
        } else if self.entries[index].age < self.current_age.load(std::sync::atomic::Ordering::SeqCst) {
            // override older entries

            self.diagnostics.increment_overwrites();
            true
        } else if Entry::get_depth(self.entries[index].data) <= depth {
            // override entries with smaller depth

            self.diagnostics.increment_overwrites();
            true
        } else {
            false
        };

        if !replace {
            return;
        }

        // TODO: Mate = game end store here independent of amount it too to get to mate, normally mate is stored as big number/big negative number -/+ the amount of moves it takes to get to mate
        // if(score > IS_MATE) score += pos->ply;
        // else if(score < -IS_MATE) score -= pos->ply;

        let data = Entry::pack_data(depth, evaluation, evaluation_type, action);
        let key = hash ^ data;

        self.entries[index] = Entry {
            key,
            data,
            age: self.current_age.load(std::sync::atomic::Ordering::SeqCst),
        };
    }

    /// Increments the age of the transposition table.
    ///
    /// This is used to invalidate old entries in the transposition table.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    pub(crate) fn increment_age(&mut self) {
        self.current_age.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Gets the principal variation line from the transposition table.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to get the PV line for.
    /// * `depth` - The depth of the PV line to get.
    ///
    /// # Returns
    ///
    /// * `Vec<ActionId>` - The PV line.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the depth of the PV line.
    pub(crate) fn get_pv_line(&self, game: &Patchwork, depth: usize) -> Vec<ActionId> {
        let mut pv_line = Vec::with_capacity(depth);

        let mut current_game = game.clone();

        for _ in 0..depth {
            if let Some(action) = self.probe_pv_move(&current_game) {
                let action_2 = action;
                let game_2 = current_game.clone();

                let result = current_game.do_action(action, true);
                if result.is_err() {
                    let hash = self.zobrist_hash.hash(&game_2);
                    let index = (hash % self.entries.len() as u64) as usize;
                    let data = self.entries[index].data;
                    let (table_depth, table_evaluation, table_evaluation_type, _) = Entry::unpack_data(data);

                    // TODO: remove prints
                    println!("──────────────────────────────────────────────────────────────────────────────");
                    println!("game: {:?}", game_2);
                    println!("action: {:?}", action_2);
                    println!("table_depth: {:?}", table_depth);
                    println!("table_evaluation: {:?}", table_evaluation);
                    println!("table_evaluation_type: {:?}", table_evaluation_type);
                    unreachable!("[TranspositionTable::get_pv_line] PV action is invalid");
                }
                pv_line.push(action);
            } else {
                break;
            }
        }

        pv_line
    }

    /// Probes the transposition table for a PV move.
    /// Returns the PV move if it is found.
    /// Returns None if no PV move is found.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state to probe the transposition table for.
    ///
    /// # Returns
    ///
    /// * `Some(ActionId)` - The PV move if it is found.
    /// * `None` - If no PV move is found.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    pub(crate) fn probe_pv_move(&self, game: &Patchwork) -> Option<ActionId> {
        let hash = self.zobrist_hash.hash(game);
        let index = (hash % self.entries.len() as u64) as usize;
        let data = self.entries[index].data;
        let test_key = hash ^ data;

        if self.entries[index].key != test_key {
            return None;
        }

        Some(Entry::get_action_id(data))
    }

    /// Clears the transposition table.
    ///
    /// This is used to clear the transposition table between games.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    #[allow(dead_code)] // TODO: move transposition table to own package
    pub(crate) fn clear(&mut self) {
        self.entries = vec![Entry::default(); self.entries.len()];
        self.current_age.store(0, std::sync::atomic::Ordering::SeqCst);

        self.diagnostics.reset_diagnostics();
    }

    /// Resets the diagnostics of the transposition table for a new search.
    ///
    /// This is used to reset the diagnostics between searches.
    ///
    /// # Complexity
    ///
    /// `𝒪(1)`
    pub(crate) fn reset_diagnostics(&mut self) {
        let entries = self
            .diagnostics
            .entries
            .load(TranspositionTableDiagnostics::LOAD_ORDERING);
        self.diagnostics.reset_diagnostics();
        self.diagnostics
            .entries
            .store(entries, TranspositionTableDiagnostics::STORE_ORDERING);
    }
}

/// Gets the action to store in the transposition table.
/// This is used to store the action with all symmetries.
///
/// # Arguments
///
/// * `game` - The game state to get the action to store for.
/// * `action` - The action to store.
/// * `player_1_rotation` - The rotation of player 1.
/// * `player_1_flip` - The flip of player 1.
/// * `player_2_rotation` - The rotation of player 2.
/// * `player_2_flip` - The flip of player 2.
///
/// # Returns
///
/// * `Some(ActionId)` - The action to store.
///
/// # Complexity
///
/// `𝒪(𝑛)` where `n` is the amount of transformation for the patch (bounded by 448).
fn get_action_to_store(
    game: &Patchwork,
    action: ActionId,
    player_1_rotation: u8,
    player_1_flip: u8,
    player_2_rotation: u8,
    player_2_flip: u8,
) -> Option<ActionId> {
    // handle default symmetries
    if (player_1_rotation == 0 && player_1_flip == 0 && game.is_player_1())
        || (player_2_rotation == 0 && player_2_flip == 0 && game.is_player_2())
    {
        return Some(action);
    }

    let rotation = if game.is_player_1() {
        player_1_rotation
    } else {
        player_2_rotation
    };
    let flip = (if game.is_player_1() {
        player_1_flip
    } else {
        player_2_flip
    }) != 0;

    // no need to modify the action as no tiles are placed
    if action.is_walking() || action.is_phantom() || action.is_null() {
        return Some(action);
    }

    if action.is_patch_placement() {
        let previous_player_was_1 = action.get_previous_player_was_1();
        let patch_id = action.get_patch_id();
        let row = action.get_row();
        let column = action.get_column();
        let patch_index = action.get_patch_index();
        let patch_transformation_index = action.get_patch_transformation_index();
        let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);

        let (row, column) = QuiltBoard::flip_horizontally_then_rotate_row_and_column(row, column, rotation, flip);

        let Some(patch_transformation_index) = apply_patch_rotation(
            patch_id,
            row,
            column,
            transformation.rotation_flag(),
            transformation.flipped(),
            rotation,
            flip,
        ) else {
            return None;
        };

        Some(ActionId::patch_placement(
            patch_id,
            patch_index,
            patch_transformation_index,
            previous_player_was_1,
        ))
    } else {
        // special patch placement
        debug_assert!(action.is_special_patch_placement());

        let row = action.get_row();
        let column = action.get_column();

        let (row, column) = QuiltBoard::flip_horizontally_then_rotate_row_and_column(row, column, rotation, flip);
        let quilt_board_index = QuiltBoard::get_index(row, column);

        Some(ActionId::special_patch_placement(quilt_board_index))
    }
}

/// Applies the patch rotation to the row and column.
///
/// # Arguments
///
/// * `patch_id` - The id of the patch.
/// * `row` - The row of the patch.
/// * `column` - The column of the patch.
/// * `patch_rotation` - The rotation of the patch.
/// * `patch_orientation` - The orientation of the patch.
/// * `applied_rotation` - The rotation that has been applied as a symmetry.
/// * `applied_orientation` - The orientation that has been applied as a symmetry.
///
/// # Returns
///
/// * `Some(u16)` - The patch transformation index after applying the symmetry.
/// * `None` - If the new patch rotation and orientation are not in the transformation list of the patch.
///
/// # Complexity
///
/// `𝒪(𝑛)` where `n` is the amount of transformation for the patch (bounded by 448).
fn apply_patch_rotation(
    patch_id: u8,
    row: u8,
    column: u8,
    patch_rotation: u8,
    patch_orientation: bool,
    applied_rotation: u8,
    applied_orientation: bool,
) -> Option<u16> {
    let mut applied_rotation = applied_rotation;

    // Cayley table for the Dihedral Group D₄ (row · column)
    // · │R⁰ R¹ R² R³ H  V  D  D'
    // ──┼───────────────────────
    // R⁰│R⁰ R¹ R² R³ H  V  D  D'
    // R¹│R¹ R² R³ R⁰ D' D  H  V
    // R²│R² R³ R⁰ R¹ V  H  D' D
    // R³│R³ R⁰ R¹ R² D  D' V  H
    // H │H  D  V  D' R⁰ R² R¹ R³
    // V │V  D' H  D  R² R⁰ R³ R¹
    // D │D  V  D' H  R³ R¹ R⁰ R²
    // D'│D' H  D  V  R¹ R³ R² R⁰

    // Flip and rotation operations are not commutative
    // correct the applied rotation to account for the flip
    if patch_orientation {
        applied_rotation = (4 - applied_rotation) % 4;
    }
    let patch_orientation = applied_orientation ^ patch_orientation;
    let patch_rotation = (patch_rotation + applied_rotation) % 4;

    PatchManager::get_transformations(patch_id)
        .iter()
        .position(|transformation| {
            transformation.row == row
                && transformation.column == column
                && transformation.rotation_flag() == patch_rotation
                && transformation.flipped() == patch_orientation
        })
        .map(|patch_transformation_index| patch_transformation_index as u16)
}
