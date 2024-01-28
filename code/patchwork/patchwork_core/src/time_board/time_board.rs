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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TimeBoard {
    /// The tiles of the time board.
    #[serde(with = "serde_bytes")]
    pub tiles: [u8; 54],
}

impl Default for TimeBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeBoard {
    /// The maximum position on the time board.
    pub const MAX_POSITION: u8 = 53;

    /// The amount of special patches on the time board.
    pub const AMOUNT_OF_SPECIAL_PATCHES: usize = 5;

    /// The amount of button income triggers on the time board.
    pub const AMOUNT_OF_BUTTON_INCOME_TRIGGERS: usize = 9;

    /// The position of the first special patch.
    pub const FIRST_SPECIAL_PATCH_POSITION: u8 = 26;
    /// The position of the last special patch.
    pub const SECOND_SPECIAL_PATCH_POSITION: u8 = 50;

    /// The position of the first button income trigger.
    pub const FIRST_BUTTON_INCOME_TRIGGER_POSITION: u8 = 5;
    /// The position of the last button income trigger.
    pub const LAST_BUTTON_INCOME_TRIGGER_POSITION: u8 = 53;

    /// Creates a new time board.
    pub const fn new() -> TimeBoard {
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

    // ──────────────────────────────────────────────── SPECIAL PATCHES ────────────────────────────────────────────────

    /// Sets the special patch at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to set the special patch at.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub fn set_special_patch(&mut self, index: u8) {
        let clamped_index: usize = (index as usize).min(TimeBoard::MAX_POSITION as usize);
        self.tiles[clamped_index] |= entities_enum::SPECIAL_PATCH;
    }

    /// Unset the special patch at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to unset the special patch at.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub fn unset_special_patch(&mut self, index: u8) {
        let clamped_index: usize = (index as usize).min(TimeBoard::MAX_POSITION as usize);
        self.tiles[clamped_index] &= !entities_enum::SPECIAL_PATCH;
    }

    /// Unset all special patches until the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to unset all special patches until.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the given index.
    pub fn unset_special_patches_until(&mut self, index: u8) {
        for tile in self.tiles.iter_mut().take(index as usize) {
            *tile &= !entities_enum::SPECIAL_PATCH;
        }
    }

    /// Checks if there is a special patch at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to check.
    ///
    /// # Returns
    ///
    /// * `true` - There is a special patch at the given index.
    /// * `false` - There is no special patch at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub const fn is_special_patch_at(&self, index: usize) -> bool {
        self.tiles[index] & entities_enum::SPECIAL_PATCH > 0
    }

    /// Checks if there is a special patch in the given range.
    ///
    /// # Remarks
    ///
    /// This function is faster than `get_amount_special_patches_in_range` if you only want to
    /// know if there is a button income trigger in the given range as it stops as soon as it finds
    /// one.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `true` - There is a special patch in the given range.
    /// * `false` - There is no special patch in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn is_special_patches_in_range(&self, range: Range<usize>) -> bool {
        for i in range {
            if self.tiles[i] & entities_enum::SPECIAL_PATCH > 0 {
                return true;
            }
        }
        false
    }

    /// Gets the amount of special patches  in the given range.
    ///
    /// # Remarks
    ///
    /// If you only want to know if there is a special patches in the given range, use
    /// `is_special_patches_in_range` as it is faster.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `usize` - The amount of special patches  in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn get_amount_special_patches_in_range(&self, range: Range<usize>) -> usize {
        let mut amount = 0;
        for i in range {
            if self.tiles[i] & entities_enum::SPECIAL_PATCH > 0 {
                amount += 1;
            }
        }
        amount
    }

    /// Checks if there is a special patch in the given range and if so returns the index of the
    /// first special patch in the given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The index of the first special patch in the given range.
    /// * `None` - There is no special patch in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    #[inline]
    pub fn get_single_special_patch_in_range(&self, range: Range<usize>) -> Option<u8> {
        let mut range = range;
        range
            .find(|&i| self.tiles[i] & entities_enum::SPECIAL_PATCH > 0)
            .map(|i| i as u8)
    }

    /// Gets all indices of special patches in the given range.
    /// Returns an empty vector if there are no special patches in the given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `Vec<usize>` - The indices of all special patches in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn get_all_special_patches_in_range(&self, range: Range<usize>) -> Vec<u8> {
        let mut result = vec![];
        for i in range {
            if self.tiles[i] & entities_enum::SPECIAL_PATCH > 0 {
                result.push(i as u8);
            }
        }
        result
    }

    /// Gets the position where the last special patch was/is before the given position.
    /// Returns `None` if there was no special patch before the given position.
    /// The special patch does not have to be on the time board anymore.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to check.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The position of the last special patch before the given position.
    /// * `None` - There was no special patch before the given position.
    ///
    /// # Examples
    ///
    /// ```txt
    ///       special
    ///        patch  position
    ///          ↓       ↓
    /// ┌-┬-┬-┬-┬-┬-┬-┬-┬-┬-┐
    /// │ │ │ │ │P│ │ │ │1│ │
    /// └-┴-┴-┴-┴-┴-┴-┴-┴-┴-┘
    /// ```
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn get_special_patch_before_position(&self, position: u8) -> Option<u8> {
        if position >= 50 {
            return Some(50);
        }
        if position >= 44 {
            return Some(44);
        }
        if position >= 38 {
            return Some(38);
        }
        if position >= 32 {
            return Some(32);
        }
        if position >= 26 {
            return Some(26);
        }
        None
    }

    // ──────────────────────────────────────────── BUTTON INCOME TRIGGERS ─────────────────────────────────────────────

    /// Checks if there is a button income trigger at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to check.
    ///
    /// # Returns
    ///
    /// * `true` - There is a button income trigger at the given index.
    /// * `false` - There is no button income trigger at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    pub const fn is_button_income_trigger_at(&self, index: usize) -> bool {
        self.tiles[index] & entities_enum::BUTTON_INCOME_TRIGGER > 0
    }

    /// Checks if there is a button income trigger in the given range.
    ///
    /// # Remarks
    ///
    /// This function is faster than `get_amount_button_income_trigger_in_range` if you only want to
    /// know if there is a button income trigger in the given range as it stops as soon as it finds
    /// one.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `true` - There is a button income trigger in the given range.
    /// * `false` - There is no button income trigger in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn is_button_income_trigger_in_range(&self, range: Range<usize>) -> bool {
        for i in range {
            if self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
                return true;
            }
        }
        false
    }

    /// Gets the amount of button income triggers in the given range.
    ///
    /// # Remarks
    ///
    /// If you only want to know if there is a button income trigger in the given range, use
    /// `is_button_income_trigger_in_range` as it is faster.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `usize` - The amount of button income triggers in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn get_amount_button_income_trigger_in_range(&self, range: Range<usize>) -> usize {
        let mut amount = 0;
        for i in range {
            if self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
                amount += 1;
            }
        }
        amount
    }

    /// Checks if there is a button income trigger in the given range and if so returns the index of
    /// the first button income trigger in the given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The index of the first button income trigger in the given range.
    /// * `None` - There is no button income trigger in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    #[inline]
    pub fn get_single_button_income_trigger_in_range(&self, range: Range<usize>) -> Option<u8> {
        let mut range = range;
        range
            .find(|&i| self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0)
            .map(|i| i as u8)
    }

    /// Gets all indices of button income triggers in the given range.
    /// Returns an empty vector if there are no button income triggers in the given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to check.
    ///
    /// # Returns
    ///
    /// * `Vec<usize>` - The indices of all button income triggers in the given range.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles in the given range.
    pub fn get_all_button_income_triggers_in_range(&self, range: Range<usize>) -> Vec<u8> {
        let mut result = vec![];
        for i in range {
            if self.tiles[i] & entities_enum::BUTTON_INCOME_TRIGGER > 0 {
                result.push(i as u8);
            }
        }
        result
    }

    /// Gets the position where the last button income trigger is before the given position.
    /// Returns `None` if there was no button income trigger before the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to check.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The position of the last button income trigger before the given position.
    /// * `None` - There was no button income trigger before the given position.
    ///
    /// # Examples
    ///
    /// ```txt
    ///    button
    ///    income
    ///    trigger  position
    ///      ↓         ↓
    /// ┌-┬-┬-┬-┬-┬-┬-┬-┬-┬-┐
    /// │ │ │B│ │ │ │ │1│ │ │
    /// └-┴-┴-┴-┴-┴-┴-┴-┴-┴-┘
    /// ```
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn get_button_income_trigger_before_position(&self, position: u8) -> Option<u8> {
        if position >= 53 {
            return Some(53);
        }
        if position >= 47 {
            return Some(47);
        }
        if position >= 41 {
            return Some(41);
        }
        if position >= 35 {
            return Some(35);
        }
        if position >= 29 {
            return Some(29);
        }
        if position >= 23 {
            return Some(23);
        }
        if position >= 17 {
            return Some(17);
        }
        if position >= 11 {
            return Some(11);
        }
        if position >= 5 {
            return Some(5);
        }
        None
    }

    // ──────────────────────────────────────────────── PLAYER POSITION ────────────────────────────────────────────────

    /// Gets the position of the given player.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player flag of the player to get the position of.
    ///
    /// # Returns
    ///
    /// * `usize` - The position of the given player.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles on the time board (usually 54).
    pub fn get_player_position(&self, player_flag: u8) -> u8 {
        debug_assert!(player_flag >> 2 == 0, "[TimeBoard::get_player_position] The given parameters are likely a patchwork status flags and not the player flags: {player_flag:b}");
        self.tiles
            .iter()
            .position(|&tile| tile & player_flag > 0)
            .expect("[TimeBoard::get_player_position] There is no player on the time board. This is a bug in the patchwork_core library.") as u8
    }

    /// Sets the position of the given player.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player flag of the player to set the position of.
    /// * `position` - The position to set the player to.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the amount of tiles on the time board (usually 54).
    pub fn set_player_position(&mut self, player_flag: u8, position: usize) {
        debug_assert!(player_flag >> 2 == 0, "[TimeBoard::set_player_position] The given parameters are likely a patchwork status flags and not the player flags: {player_flag:b}");
        let clamped_position = position.min(TimeBoard::MAX_POSITION as usize);
        self.tiles.iter_mut().for_each(|tile| *tile &= !player_flag);
        self.tiles[clamped_position] |= player_flag;
    }

    /// Moves the player from the old_position to the new_position.
    /// This function does not check if the given old position is valid.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player flag of the player to set the position of.
    /// * `old_position` - The old position of the player.
    /// * `new_position` - The new position of the player.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// This function is undefined behavior if the given old position is not valid.
    /// This will panic in debug mode.
    pub fn move_player_position(&mut self, player_flag: u8, old_position: u8, new_position: u8) {
        let old_position = old_position.min(TimeBoard::MAX_POSITION);

        debug_assert!(player_flag >> 2 == 0, "[TimeBoard::move_player_position] The given parameters are likely a patchwork status flags and not the player flags: {player_flag:b}");
        debug_assert_eq!(
            self.get_player_position(player_flag),
            old_position,
            "[TimeBoard::move_player_position] time_board.get_player_position({:?}) != {:?} (old_position): The given old position is not valid.",
            player_flag,
            old_position
        );

        // reset old position
        self.tiles[old_position as usize] ^= player_flag;

        // set new position
        let clamped_position = (new_position as usize).min(TimeBoard::MAX_POSITION as usize);
        self.tiles[clamped_position] |= player_flag;
    }
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
