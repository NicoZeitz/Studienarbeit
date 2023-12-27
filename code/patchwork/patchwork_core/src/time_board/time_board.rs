use crate::Patchwork;
use std::{fmt::Display, ops::Range};

/// The entities that can be on a tile of the time board.
pub mod entities_enum {
    /// The first player.
    pub const PLAYER_1: u8 = 0b0000_0001; // 1
    /// The second player.
    pub const PLAYER_2: u8 = 0b0000_0010; // 2
    /// A button income trigger.
    pub const BUTTON_INCOME_TRIGGER: u8 = 0b0000_0100; // 4
    /// A special patch.
    pub const SPECIAL_PATCH: u8 = 0b0000_1000; // 8
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeBoard {
    /// The tiles of the time board.
    tiles: [u8; 54],
}

impl Default for TimeBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeBoard {
    /// The maximum position on the time board.
    pub const MAX_POSITION: usize = 53;

    /// The amount of button income triggers on the time board.
    pub const AMOUNT_BUTTON_INCOME_TRIGGERS: usize = 9;

    /// Creates a new time board.
    pub fn new() -> TimeBoard {
        let mut tiles = [0; 54];

        tiles[0] = entities_enum::PLAYER_1 | entities_enum::PLAYER_2;

        tiles[5] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[11] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[17] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[23] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[29] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[35] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[41] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[47] = entities_enum::BUTTON_INCOME_TRIGGER;
        tiles[53] = entities_enum::BUTTON_INCOME_TRIGGER;

        tiles[26] = entities_enum::SPECIAL_PATCH;
        tiles[32] = entities_enum::SPECIAL_PATCH;
        tiles[38] = entities_enum::SPECIAL_PATCH;
        tiles[44] = entities_enum::SPECIAL_PATCH;
        tiles[50] = entities_enum::SPECIAL_PATCH;

        TimeBoard { tiles }
    }

    pub fn is_button_income_trigger_in_range(&self, range: Range<usize>) -> bool {
        for i in range {
            if self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
                return true;
            }
        }
        false
    }

    pub fn get_amount_button_income_trigger_in_range(&self, range: Range<usize>) -> usize {
        let mut amount = 0;
        for i in range {
            if self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
                amount += 1;
            }
        }
        amount
    }

    pub fn get_special_patch_in_range(&self, range: Range<usize>) -> Option<usize> {
        let mut range = range;
        range.find(|&i| self.tiles[i] & entities_enum::SPECIAL_PATCH > 0)
    }

    pub fn has_special_patch_at(&self, index: usize) -> bool {
        self.tiles[index] & entities_enum::SPECIAL_PATCH > 0
    }

    pub fn clear_special_patch(&mut self, index: usize) {
        let clamped_index = index.clamp(0, TimeBoard::MAX_POSITION);
        self.tiles[clamped_index] ^= entities_enum::SPECIAL_PATCH;
    }

    pub(crate) fn set_special_patch(&mut self, index: usize) {
        let clamped_index = index.clamp(0, TimeBoard::MAX_POSITION);
        self.tiles[clamped_index] |= entities_enum::SPECIAL_PATCH;
    }

    pub fn clear_special_patches_until(&mut self, index: usize) {
        for tile in self.tiles.iter_mut().take(index) {
            *tile &= !entities_enum::SPECIAL_PATCH;
        }
    }

    pub fn set_player_position(&mut self, player_flag: i8, old_position: usize, new_position: usize) {
        let player = if player_flag == Patchwork::PLAYER_1 {
            entities_enum::PLAYER_1
        } else {
            entities_enum::PLAYER_2
        };

        // reset old position
        self.tiles[old_position] ^= player;

        // set new position
        let clamped_position = new_position.clamp(0, TimeBoard::MAX_POSITION);
        self.tiles[clamped_position] |= player;
    }
}

fn get_str_for_tile(tile: u8) -> String {
    let mut result_str = "".to_string();

    if tile & entities_enum::PLAYER_1 > 0 {
        result_str += "1";
    }
    if tile & entities_enum::PLAYER_2 > 0 {
        result_str += "2";
    }
    if tile & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
        result_str += "B";
    } else if tile & entities_enum::SPECIAL_PATCH > 0 {
        result_str += "P";
    }

    if result_str.is_empty() {
        result_str = " ".to_string();
    }

    result_str
}

impl Display for TimeBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first_line = vec![];
        let mut second_line = vec![];
        let mut third_line = vec![];

        for field in self.tiles.iter() {
            let tile_str = get_str_for_tile(*field);
            first_line.push("-".repeat(tile_str.len()));
            third_line.push("-".repeat(tile_str.len()));
            second_line.push(tile_str);
        }

        writeln!(f, "┌{}┐", &first_line.join("┬"))?;
        writeln!(f, "│{}│", &second_line.join("│"))?;
        write!(f, "└{}┘", &third_line.join("┴"))
    }
}
