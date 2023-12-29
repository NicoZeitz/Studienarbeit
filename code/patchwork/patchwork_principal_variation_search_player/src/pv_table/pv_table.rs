use patchwork_core::Action;

use crate::PVSPlayer;

/// Principal Variation Table
///
/// A table that stores the principal variation of a game state.
/// The principal variations are stored as a [Triangular PV-Table](https://www.chessprogramming.org/Triangular_PV-Table)
///
/// The size of the table is `(MAX_DEPTH + 1) * MAX_DEPTH / 2`.
///
/// The current `MAX_DEPTH` is `256`. With this the table has `32896` entries.
/// Each entry is 112 bytes big. This means the table has a size of `≈3.51 MB`.
pub(crate) struct PVTable {
    /// A triangular table of actions.
    ///
    /// The maximum size is the [Triangular number](https://en.wikipedia.org/wiki/Triangular_number)
    #[allow(unused)] // FEATURE:PV_TABLE
    table: [Option<Action>; (PVSPlayer::MAX_DEPTH + 1) * PVSPlayer::MAX_DEPTH / 2],
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            table: array_init::array_init(|_| None),
        }
    }
}
