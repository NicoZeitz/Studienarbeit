use std::fmt::Display;

use crate::QuiltBoard;

/// Represents the state of a player in the game Patchwork.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerState {
    /// THe position of the player on the time board.
    pub position: usize,
    /// The amount of buttons the player has.
    pub button_balance: isize,
    /// The quilt board of the player.
    pub quilt_board: QuiltBoard,
}

impl PlayerState {
    pub const STARTING_BUTTON_BALANCE: isize = 5;

    /// Creates a new [`PlayerState`] with the default values.
    pub fn new() -> PlayerState {
        PlayerState {
            position: 0,
            button_balance: PlayerState::STARTING_BUTTON_BALANCE,
            quilt_board: QuiltBoard::new(),
        }
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
            // TODO: can we get the player name here without duplicating the string?
            self.button_balance,
            self.quilt_board
        )
    }
}
