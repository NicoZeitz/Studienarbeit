use std::{cmp::Ordering, fmt::Display};

pub use crate::game::*;
use crate::{Patch, PatchManager, PlayerState, QuiltBoard, Termination, TerminationType, TimeBoard};

/// Represents the type of turn that is currently being played.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TurnType {
    /// A normal turn.
    Normal,
    /// A turn where the player has to place a special patch.
    SpecialPatchPlacement,
    /// A turn that was created because a player switch was forced.
    /// The only available action is to take a phantom action.
    NormalPhantom,
    /// A turn that was created because a player switch was forced while a special patch was being placed.
    /// The only available action is to take a phantom action.
    SpecialPhantom,
}

/// Different flags for the status of the game.
#[rustfmt::skip]
pub mod status_enum {
    /// The first player.
    pub const PLAYER_1: u8 = 0b0000_0001; // 1
    /// The second player.
    pub const PLAYER_2: u8 = 0b0000_0010; // 2
    /// If the first player has the special tile (e.g. the 7x7 tile).
    pub const PLAYER_1_HAS_SPECIAL_TILE: u8 = 0b0000_0100; // 4
    /// If the second player has the special tile (e.g. the 7x7 tile).
    pub const PLAYER_2_HAS_SPECIAL_TILE: u8 = 0b0000_1000; // 8
    /// If the first player was first to reach the goal.
    pub const PLAYER_1_FIRST_AT_END: u8 = 0b0001_0000; // 16
    /// If the second player was first to reach the goal.
    pub const PLAYER_2_FIRST_AT_END: u8 = 0b0010_0000; // 32

    /// The flags for both players combined.
    pub const BOTH_PLAYERS: u8 = PLAYER_1 | PLAYER_2; // 3
    /// The flags for both players having the special tile combined.
    #[allow(dead_code)]
    pub const BOTH_PLAYERS_HAVE_SPECIAL_TILE: u8 = PLAYER_1_HAS_SPECIAL_TILE | PLAYER_2_HAS_SPECIAL_TILE; // 12
    /// The flags for both players being first to reach the goal combined.
    #[allow(dead_code)]
    pub const BOTH_PLAYERS_FIRST_AT_END: u8 = PLAYER_1_FIRST_AT_END | PLAYER_2_FIRST_AT_END; // 48
}

/// Represents the full state of the patchwork board game.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Patchwork {
    /// The patches that are available to be purchased.
    #[serde(serialize_with = "serialize_patches", deserialize_with = "deserialize_patches")]
    pub patches: Vec<&'static Patch>,
    /// The time board, which is a 9x9 grid of tiles.
    pub time_board: TimeBoard,
    /// The first player in the game.
    pub player_1: PlayerState,
    /// The second player in the game.
    pub player_2: PlayerState,
    /// The type of turn that is currently being played.
    pub turn_type: TurnType,
    /// Different flags for the current status of the game.
    ///
    /// Consists of
    /// * The current player
    /// * Whether a player has the special tile
    /// * Whether a player was first to reach the end
    ///
    /// It is illegal to have both players be the current player.
    /// It is illegal to have both players have the special tile.
    /// It is illegal to have both players be first to reach the end.
    pub(crate) status_flags: u8,
}

// Impl block for different getters and setters
impl Patchwork {
    /// Gets the player with the given flag.
    #[inline]
    pub const fn get_player(&self, player: u8) -> &PlayerState {
        if (player & status_enum::PLAYER_1) > 0 {
            &self.player_1
        } else {
            &self.player_2
        }
    }

    // Returns if the current player is player 1.
    #[inline]
    pub const fn is_player_1(&self) -> bool {
        (self.status_flags & status_enum::PLAYER_1) > 0
    }

    // Returns if the given player is player 1.
    #[inline]
    pub const fn is_flag_player_1(player_flag: u8) -> bool {
        player_flag == status_enum::PLAYER_1
    }

    // Returns if the current player is player 2.
    #[inline]
    pub const fn is_player_2(&self) -> bool {
        (self.status_flags & status_enum::PLAYER_2) > 0
    }

    // Returns if the given player is player 2.
    #[inline]
    pub const fn is_flag_player_2(player_flag: u8) -> bool {
        player_flag == status_enum::PLAYER_2
    }

    /// Returns the flag for player 1.
    #[inline]
    pub const fn get_player_1_flag() -> u8 {
        status_enum::PLAYER_1
    }

    /// Returns the flag for player 2.
    #[inline]
    pub const fn get_player_2_flag() -> u8 {
        status_enum::PLAYER_2
    }

    /// Returns the current player.
    #[inline]
    pub fn current_player(&self) -> &PlayerState {
        if self.is_player_1() {
            &self.player_1
        } else {
            &self.player_2
        }
    }

    /// Returns a mutable reference to the current player.
    #[inline]
    pub fn current_player_mut(&mut self) -> &mut PlayerState {
        if self.is_player_1() {
            &mut self.player_1
        } else {
            &mut self.player_2
        }
    }

    /// Returns the other player.
    #[inline]
    pub const fn other_player(&self) -> &PlayerState {
        if self.is_player_1() {
            &self.player_2
        } else {
            &self.player_1
        }
    }

    /// Returns a mutable reference to the other player.
    #[inline]
    pub fn other_player_mut(&mut self) -> &mut PlayerState {
        if self.is_player_1() {
            &mut self.player_2
        } else {
            &mut self.player_1
        }
    }

    /// Switches the current player.
    #[inline]
    pub fn switch_player(&mut self) {
        if self.is_player_1() {
            self.status_flags &= !status_enum::PLAYER_1;
            self.status_flags |= status_enum::PLAYER_2;
        } else {
            self.status_flags &= !status_enum::PLAYER_2;
            self.status_flags |= status_enum::PLAYER_1;
        }
    }

    /// Returns if the special tile condition has already been reached by either player.
    ///
    /// # Returns
    ///
    /// If the special tile condition has already been reached by either player.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn is_special_tile_condition_reached(&self) -> bool {
        self.status_flags & (status_enum::PLAYER_1_HAS_SPECIAL_TILE | status_enum::PLAYER_2_HAS_SPECIAL_TILE) > 0
    }

    /// Returns if the special tile condition has already been reached by player 1.
    ///
    /// # Returns
    ///
    /// If the special tile condition has already been reached by player 1.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn is_special_tile_condition_reached_by_player_1(&self) -> bool {
        self.status_flags & status_enum::PLAYER_1_HAS_SPECIAL_TILE > 0
    }

    /// Returns if the special tile condition has already been reached by player 2.
    ///
    /// # Returns
    ///
    /// If the special tile condition has already been reached by player 2.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn is_special_tile_condition_reached_by_player_2(&self) -> bool {
        self.status_flags & status_enum::PLAYER_2_HAS_SPECIAL_TILE > 0
    }

    /// Sets the special tile condition for the given player.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player to set the special tile condition for.
    ///
    /// # Undefined Behavior
    ///
    /// If the special tile condition has already been reached. This will panic
    /// in debug mode.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn set_special_tile_condition(&mut self, player_flag: u8) {
        debug_assert!(
            !self.is_special_tile_condition_reached(),
            "[Patchwork::set_special_tile_condition] Special tile condition has already been reached"
        );

        if Self::is_flag_player_1(player_flag) {
            self.status_flags |= status_enum::PLAYER_1_HAS_SPECIAL_TILE;
        } else {
            self.status_flags |= status_enum::PLAYER_2_HAS_SPECIAL_TILE;
        }
    }

    /// Unset the special tile condition for the given player.
    ///
    /// Does nothing if the special tile condition has not been reached.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player to unset the special tile condition for.
    pub fn unset_special_tile_condition(&mut self, player_flag: u8) {
        if Self::is_flag_player_1(player_flag) {
            self.status_flags &= !status_enum::PLAYER_1_HAS_SPECIAL_TILE;
        } else {
            self.status_flags &= !status_enum::PLAYER_2_HAS_SPECIAL_TILE;
        }
    }

    /// Returns if the goal has already been reached by either player.
    ///
    /// # Returns
    ///
    /// If the goal has already been reached by either player.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn is_goal_reached(&self) -> bool {
        self.status_flags & (status_enum::PLAYER_1_FIRST_AT_END | status_enum::PLAYER_2_FIRST_AT_END) > 0
    }

    /// Returns if the goal has already been reached by player 1.
    ///
    /// # Returns
    ///
    /// If the goal has already been reached by player 1.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn player_1_was_first_to_reach_goal(&self) -> bool {
        self.status_flags & status_enum::PLAYER_1_FIRST_AT_END > 0
    }

    /// Returns if the goal has already been reached by player 2.
    ///
    /// # Returns
    ///
    /// If the goal has already been reached by player 2.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub const fn player_2_was_first_to_reach_goal(&self) -> bool {
        self.status_flags & status_enum::PLAYER_2_FIRST_AT_END > 0
    }

    /// Sets the goal reached for the given player.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player to set the goal reached for.
    ///
    /// # Undefined Behavior
    ///
    /// If the goal has already been reached by either player. This will panic
    /// in debug mode.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn set_goal_reached(&mut self, player_flag: u8) {
        debug_assert!(
            !self.is_goal_reached(),
            "[Patchwork::set_goal_reached] Goal has already been reached by either player"
        );

        if Self::is_flag_player_1(player_flag) {
            self.status_flags |= status_enum::PLAYER_1_FIRST_AT_END;
        } else {
            self.status_flags |= status_enum::PLAYER_2_FIRST_AT_END;
        }
    }

    /// Unset the goal reached for the given player.
    ///
    /// Does nothing if the goal has not been reached.
    ///
    /// # Arguments
    ///
    /// * `player_flag` - The player to unset the goal reached for.
    pub fn unset_goal_reached(&mut self, player_flag: u8) {
        debug_assert!(player_flag >> 2 == 0, "[Patchwork::unset_goal_reached] The given parameters are likely a patchwork status flags and not the player flags: {player_flag:b}");

        if Self::is_flag_player_1(player_flag) {
            self.status_flags &= !status_enum::PLAYER_1_FIRST_AT_END;
        } else {
            self.status_flags &= !status_enum::PLAYER_2_FIRST_AT_END;
        }
    }

    /// Gets the score of the given player.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    /// * `player_flag` - The player to get the score for.
    ///
    /// # Returns
    ///
    /// The score of the given player.
    pub const fn get_score(&self, player_flag: u8) -> i32 {
        let player = &self.get_player(player_flag);

        let mut score = player.quilt_board.score() + player.button_balance;

        if (Self::is_flag_player_1(player_flag) && (self.status_flags & status_enum::PLAYER_1_HAS_SPECIAL_TILE) > 0)
            || (Self::is_flag_player_2(player_flag) && (self.status_flags & status_enum::PLAYER_2_HAS_SPECIAL_TILE) > 0)
        {
            score += QuiltBoard::FULL_BOARD_BUTTON_INCOME;
        }

        score
    }

    /// Gets the termination result of the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// The termination result of the game associated with the given state.
    pub fn get_termination_result(&self) -> Termination {
        let player_1_score = self.get_score(status_enum::PLAYER_1);
        let player_2_score = self.get_score(status_enum::PLAYER_2);

        let termination = match player_1_score.cmp(&player_2_score) {
            Ordering::Less => TerminationType::Player2Won,
            Ordering::Greater => TerminationType::Player1Won,
            Ordering::Equal => {
                if (self.status_flags & status_enum::PLAYER_1_FIRST_AT_END) > 0 {
                    TerminationType::Player1Won
                } else if (self.status_flags & status_enum::PLAYER_2_FIRST_AT_END) > 0 {
                    TerminationType::Player2Won
                } else {
                    panic!("[Patchwork::get_termination_result] Both players have the same score and neither was first to reach the end")
                }
            }
        };

        Termination {
            termination,
            player_1_score,
            player_2_score,
        }
    }
}

impl Display for Patchwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Current player is {}", if self.is_player_1() { "1" } else { "2" })?;
        if matches!(self.turn_type, TurnType::SpecialPatchPlacement) {
            write!(f, " (special patch placement move)",)?;
        }
        write!(f, "\n\n")?;

        let player_1_string = format!("{}", self.player_1);
        let player_2_string = format!("{}", self.player_2);

        let player_1_lines = player_1_string.split('\n');
        let mut player_2_lines = player_2_string.split('\n');

        let max_length = player_1_lines
            .clone()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        // pad each line in player 1 to the same length
        for (player_1_line, player_2_line) in player_1_lines.zip(&mut player_2_lines) {
            write!(f, "{}", player_1_line)?;
            write!(f, "{}", " ".repeat(max_length - player_1_line.chars().count()))?;
            writeln!(f, " │ {}", player_2_line)?;
        }

        write!(f, "\nTime board:\n{}\n", self.time_board)?;
        writeln!(f, "Next 6 patches (can only take first 3):")?;

        // only take first 6 patches
        let patch_strings = self.patches.iter().take(6).map(|patch| format!("{}", patch));

        let patch_strings_lines = patch_strings
            .into_iter()
            .map(|patch_string| {
                patch_string
                    .split('\n')
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let max_amount_of_lines = patch_strings_lines.iter().map(|lines| lines.len()).max().unwrap_or(0);

        // pad each patch with newlines on top to max length
        let patch_strings_lines = patch_strings_lines
            .into_iter()
            .map(|mut lines| {
                while lines.len() < max_amount_of_lines {
                    lines.insert(0, "".to_string());
                }
                lines
            })
            .collect::<Vec<_>>();

        // pad each line in each patch to the same length
        let patch_strings_lines = patch_strings_lines
            .into_iter()
            .map(|lines| {
                let max_length = lines
                    .clone()
                    .into_iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0);

                lines
                    .into_iter()
                    .map(|line| format!("{}{}", line, " ".repeat(max_length - line.chars().count())))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // transpose vec
        let patch_strings_lines = (0..max_amount_of_lines)
            .map(|i| {
                patch_strings_lines
                    .clone()
                    .into_iter()
                    .map(|lines| lines[i].clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for patch_strings_lines in patch_strings_lines {
            for line in patch_strings_lines {
                write!(f, "{}    ", line)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

fn serialize_patches<S>(patches: &[&'static Patch], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let patches = patches.iter().map(|patch| patch.id).collect::<Vec<_>>();
    serde_bytes::serialize(&patches, serializer)
}

fn deserialize_patches<'de, D>(deserializer: D) -> Result<Vec<&'static Patch>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let patches: Vec<u8> = serde_bytes::deserialize(deserializer)?;
    Ok(patches.into_iter().map(PatchManager::get_patch).collect::<Vec<_>>())
}
