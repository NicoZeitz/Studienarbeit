use std::{cmp::Ordering, fmt::Display};

pub use crate::game::*;
use crate::{Patch, PlayerState, Termination, TerminationType, TimeBoard};

/// Represents the type of turn that is currently being played.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TurnType {
    /// A normal turn.
    Normal,
    /// A turn where the player has to place a special patch.
    SpecialPatchPlacement(usize),
    /// A turn that was created because a player switch was forced.
    /// The only available action is to take a null action.
    NormalPhantom,
    /// A turn that was created because a player switch was forced while a special patch was being placed.
    /// The only available action is to take a null action.
    SpecialPhantom(usize),
}

/// Represents the full state of the patchwork board game.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Patchwork {
    /// The patches that are available to be purchased.
    pub patches: Vec<&'static Patch>,
    /// The time board, which is a 9x9 grid of tiles.
    pub time_board: TimeBoard,
    /// The first player in the game.
    pub player_1: PlayerState,
    /// The second player in the game.
    pub player_2: PlayerState,
    /// The player whose turn it is. 1 for player 1, -1 for player 2.
    pub(crate) current_player_flag: i8,
    /// The type of turn that is currently being played.
    pub(crate) turn_type: TurnType,
}

// Impl block for different getters and setters
impl Patchwork {
    /// The flag for player 1.
    pub const PLAYER_1: i8 = 1;

    /// The flag for player 2.
    pub const PLAYER_2: i8 = -1;

    /// Gets the player with the given flag.
    #[inline]
    pub fn get_player(&self, player: i8) -> &PlayerState {
        if player == Self::PLAYER_1 {
            &self.player_1
        } else {
            &self.player_2
        }
    }

    // Returns if the current player is player 1.
    #[inline]
    pub fn is_player_1(&self) -> bool {
        self.current_player_flag == Patchwork::PLAYER_1
    }

    // Returns if the given player is player 1.
    #[inline]
    pub fn is_flag_player_1(&self, player_flag: i8) -> bool {
        player_flag == Patchwork::PLAYER_1
    }

    // Returns if the current player is player 2.
    #[inline]
    pub fn is_player_2(&self) -> bool {
        self.current_player_flag == Patchwork::PLAYER_2
    }

    // Returns if the given player is player 2.
    #[inline]
    pub fn is_flag_player_2(&self, player_flag: i8) -> bool {
        player_flag == Patchwork::PLAYER_2
    }

    /// Returns the flag for player 1.
    #[inline]
    pub fn get_player_1_flag(&self) -> i8 {
        Patchwork::PLAYER_1
    }

    /// Returns the flag for player 2.
    #[inline]
    pub fn get_player_2_flag(&self) -> i8 {
        Patchwork::PLAYER_2
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
    pub fn other_player(&self) -> &PlayerState {
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
        self.current_player_flag *= -1;
    }

    /// Gets the score of the given player.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    /// * `player` - The player to get the score for.
    ///
    /// # Returns
    ///
    /// The score of the given player.
    pub fn get_score(&self, player: i8) -> i32 {
        let player = &self.get_player(player);
        player.quilt_board.score() + player.button_balance
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
        let player_1_score = self.get_score(Patchwork::PLAYER_1);
        let player_2_score = self.get_score(Patchwork::PLAYER_2);

        let termination = match player_1_score.cmp(&player_2_score) {
            Ordering::Less => TerminationType::Player2Won,
            Ordering::Equal => TerminationType::Draw,
            Ordering::Greater => TerminationType::Player1Won,
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
        if let TurnType::SpecialPatchPlacement(special_patch_placement_action) = self.turn_type {
            write!(
                f,
                " (special patch placement move {})",
                special_patch_placement_action + 1
            )?;
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
