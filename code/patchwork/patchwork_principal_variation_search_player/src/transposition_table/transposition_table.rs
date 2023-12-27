use std::sync::atomic::AtomicUsize;

use patchwork_core::{Action, ActionPayload, PatchManager, Patchwork, QuiltBoard};

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
    /// * `Some((Action, isize))` - The evaluation if it is found.
    /// * `None` - If no evaluation is found.
    pub(crate) fn probe_hash_entry(
        &self,
        game: &Patchwork,
        alpha: isize,
        beta: isize,
        depth: usize,
    ) -> Option<(Action, isize)> {
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

        let (table_depth, table_evaluation, table_evaluation_type, table_action) =
            Entry::unpack_data(data, self.entries[index].extra_data)?;

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
    pub(crate) fn store_evaluation_with_symmetries(
        &mut self,
        game: &Patchwork,
        depth: usize,
        evaluation: isize,
        evaluation_type: EvaluationType,
        action: &Action,
    ) {
        // // no symmetry reduction for null and walking actions possible --> WRONG
        // if action.is_null() || action.is_walking() {
        //     self.store_evaluation(game, depth, evaluation, evaluation_type, action);
        //     return;
        // }

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
                self.store_evaluation(&game_to_store, depth, evaluation, evaluation_type, &action_to_store);
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
    #[allow(clippy::if_same_then_else)]
    pub(crate) fn store_evaluation(
        &mut self,
        game: &Patchwork,
        depth: usize,
        evaluation: isize,
        evaluation_type: EvaluationType,
        action: &Action,
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
        // if(score > ISMATE) score += pos->ply;
        // else if(score < -ISMATE) score -= pos->ply;

        let (data, extra_data) = Entry::pack_data(depth, evaluation, evaluation_type, action.clone());
        let key = hash ^ data;

        self.entries[index] = Entry {
            key,
            data,
            age: self.current_age.load(std::sync::atomic::Ordering::SeqCst),
            extra_data,
        };
    }

    /// Increments the age of the transposition table.
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
    /// * `Vec<Action>` - The PV line.
    pub(crate) fn get_pv_line(&self, game: &Patchwork, depth: usize) -> Vec<Action> {
        let mut pv_line = Vec::with_capacity(depth);

        let mut current_game = game.clone();

        for _ in 0..depth {
            if let Some(action) = self.probe_pv_move(&current_game) {
                let result = current_game.do_action(&action, true);
                if result.is_err() {
                    unreachable!("[TranspositionTable][get_pv_line] PV action is invalid");
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
    /// * `Some(Action)` - The PV move if it is found.
    /// * `None` - If no PV move is found.
    pub(crate) fn probe_pv_move(&self, game: &Patchwork) -> Option<Action> {
        let hash = self.zobrist_hash.hash(game);
        let index = (hash % self.entries.len() as u64) as usize;
        let data = self.entries[index].data;
        let test_key = hash ^ data;

        if self.entries[index].key != test_key {
            return None;
        }

        let table_action = Entry::get_action(data, self.entries[index].extra_data)?;
        Some(table_action)
    }

    /// Clears the transposition table.
    ///
    /// This is used to clear the transposition table between games.
    pub(crate) fn clear(&mut self) {
        self.entries = vec![Entry::default(); self.entries.len()];
        self.current_age.store(0, std::sync::atomic::Ordering::SeqCst);
        self.diagnostics.reset_diagnostics();
    }

    /// Resets the diagnostics of the transposition table for a new search.
    ///
    /// This is used to reset the diagnostics between searches.
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
/// * `Some(Action)` - The action to store.
fn get_action_to_store(
    game: &Patchwork,
    action: &Action,
    player_1_rotation: usize,
    player_1_flip: usize,
    player_2_rotation: usize,
    player_2_flip: usize,
) -> Option<Action> {
    // handle default symmetries
    if (player_1_rotation == 0 && player_1_flip == 0 && game.is_player_1())
        || (player_2_rotation == 0 && player_2_flip == 0 && game.is_player_2())
    {
        return Some(action.clone());
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

    match action.payload {
        // no need to modify the action as no tiles are placed
        ActionPayload::Null => Some(action.clone()),
        // no need to modify the action as no tiles are placed
        ActionPayload::Walking { .. } => Some(action.clone()),
        ActionPayload::PatchPlacement {
            patch,
            patch_index,
            patch_rotation,
            patch_orientation,
            row,
            column,
            starting_index,
            next_quilt_board,
            previous_quilt_board,
        } => {
            let (row, column) = QuiltBoard::flip_horizontally_then_rotate_row_and_column(row, column, rotation, flip);
            if let Some((patch_rotation, patch_orientation)) = apply_patch_rotation(
                patch.id,
                row,
                column,
                patch_rotation as u8,
                patch_orientation != 0,
                rotation as u8,
                flip,
            ) {
                Some(Action::new(ActionPayload::PatchPlacement {
                    patch,
                    patch_index,
                    patch_rotation: patch_rotation as usize,
                    patch_orientation: if patch_orientation { 1 } else { 0 },
                    row,
                    column,
                    starting_index,
                    next_quilt_board: next_quilt_board.map(|next_quilt_board| {
                        QuiltBoard::flip_horizontally_then_rotate_tiles(next_quilt_board, rotation, flip)
                    }),
                    previous_quilt_board: previous_quilt_board.map(|previous_quilt_board| {
                        QuiltBoard::flip_horizontally_then_rotate_tiles(previous_quilt_board, rotation, flip)
                    }),
                }))
            } else {
                None
            }
        }
        ActionPayload::SpecialPatchPlacement {
            patch_id,
            row,
            column,
            next_quilt_board,
            previous_quilt_board,
        } => {
            let (row, column) = QuiltBoard::flip_horizontally_then_rotate_row_and_column(row, column, rotation, flip);
            Some(Action::new(ActionPayload::SpecialPatchPlacement {
                patch_id,
                row,
                column,
                next_quilt_board: next_quilt_board.map(|next_quilt_board| {
                    QuiltBoard::flip_horizontally_then_rotate_tiles(next_quilt_board, rotation, flip)
                }),
                previous_quilt_board: previous_quilt_board.map(|previous_quilt_board| {
                    QuiltBoard::flip_horizontally_then_rotate_tiles(previous_quilt_board, rotation, flip)
                }),
            }))
        }
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
/// * `Some((u8, bool))` - The patch rotation and orientation after applying the symmetry.
/// * `None` - If the new patch rotation and orientation are not in the transformation list of the patch.
fn apply_patch_rotation(
    patch_id: usize,
    row: usize,
    column: usize,
    patch_rotation: u8,
    patch_orientation: bool,
    applied_rotation: u8,
    applied_orientation: bool,
) -> Option<(u8, bool)> {
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

    if PatchManager::get_instance()
        .get_transformations(patch_id)
        .iter()
        .any(|transformation| {
            transformation.row == row
                && transformation.column == column
                && transformation.rotation_flag() == patch_rotation
                && transformation.flipped() == patch_orientation
        })
    {
        Some((patch_rotation, patch_orientation))
    } else {
        None
    }
}
