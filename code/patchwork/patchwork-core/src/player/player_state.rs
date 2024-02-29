use std::fmt::Display;

use crate::{QuiltBoard, TimeBoard};

/// Represents the state of a player in the game Patchwork.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlayerState {
    /// The position of the player on the time board.
    ///
    /// This can be greater than [`TimeBoard::MAX_POSITION`] to allow for
    /// undo actions
    ///
    /// To get the correct position use the [`get_position`] method.
    pub(crate) position: u8,
    /// The amount of buttons the player has.
    pub button_balance: i32,
    /// The quilt board of the player.
    pub quilt_board: QuiltBoard,
}

impl PlayerState {
    /// The starting amount of buttons for a player.
    pub const STARTING_BUTTON_BALANCE: i32 = 5;

    /// Creates a new [`PlayerState`] with the default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            position: 0,
            button_balance: Self::STARTING_BUTTON_BALANCE,
            quilt_board: QuiltBoard::new(),
        }
    }

    /// Returns the position of the player on the time board.
    ///
    /// # Returns
    ///
    /// The position of the player on the time board.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    #[must_use]
    pub fn get_position(&self) -> u8 {
        self.position.min(TimeBoard::MAX_POSITION)
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Player (button balance: {}):\n{}",
            self.button_balance, self.quilt_board
        )
    }
}
