use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{Notation, PatchManager, Patchwork, PatchworkError, PlayerState, QuiltBoard, TimeBoard, TurnType};

lazy_static! {
    static ref STATE_REGEX: Regex = Regex::new(
        r"^(?P<player_1_quilt_board>(?:[A-Fa-f0-9]){21})/(?P<player_1_income>-?\d+)/(?P<player_1_button_income>\d+)/(?P<player_1_position>\d+) (?P<player_2_quilt_board>(?:0|1){81})/(?P<player_2_income>-?\d+)/(?P<player_2_button_income>\d+)/(?P<player_2_position>\d+) (?P<current_player>[01]) (?P<patches>(?:(?:\d+/)*\d+)|-) (?P<special_patch_placement_move>(?:-|26|32|38|44|50))$",
    ).unwrap();
}

impl Notation for Patchwork {
    /// Saves the state of the game as a string.
    /// The state can be loaded again with `load_state`.
    ///
    /// # Returns
    ///
    /// The state of the game as a string.
    ///
    /// # State Representation
    ///
    /// The state representation is partially inspired by Forsyth-Edwards Notation (FEN)
    /// The state consists of 4 parts each separated by a space
    /// 1. All Information about player 1 separated by a slash
    ///     a. The quilt board a 21 character long hexadecimal string
    ///     b. The button balance
    ///     c. The button income
    ///     d. The position on the time board
    /// 2. All Information about player 2 stored the same way as player 1
    /// 3. The current player - '0' for player 1 and '1' for player 2
    /// 4. The patches still left to take - A list of patch ids separated by
    ///    a slash starting from the first patch the current player can take
    ///    or '-' if no patches are left
    /// 5. If the current move is a special patch placement move and the fitting index (one of '26', '32', '38', '44' or '50')
    ///    or '-' if the special patch placement move is not active
    ///
    /// # Example
    ///
    /// ```
    /// // The state of an example starting game
    /// let state = Patchwork::load_from_notation("000000000000000000000/5/0/0 000000000000000000000/5/0/0 0 1/2/3/4/5/6/7/8/9/10/11/12/13/14/15/16/17/18/19/20/21/22/23/24/25/26/27/28/29/30/31/32/0 -");
    /// let notation = state.save_to_notation().unwrap();
    /// ```
    // TODO: allow better notation with B for button balance, I for income and so on
    fn save_to_notation(&self) -> Result<String, PatchworkError> {
        // TODO: uncomment
        // if matches!(self.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom(_)) {
        //     return Err(PatchworkError::InvalidNotationError {
        //         notation: "".to_string(),
        //         reason: "[Patchwork][save_to_notation] Cannot save phantom state!",
        //     });
        // }

        let mut state = String::new();

        // 1. All Information about player 1 separated by a slash
        //     a. The quilt board a 81 character string of 0 and 1s
        //     b. The button balance
        //     c. The button income
        //     d. The position on the time board
        state.push_str(
            format!(
                "{:021X}/{:?}/{:?}/{:?} ",
                self.player_1.quilt_board.tiles,
                self.player_1.button_balance,
                self.player_1.quilt_board.button_income,
                self.player_1.position,
            )
            .as_str(),
        );

        // 2. All Information about player 2 stored the same way as player 1
        state.push_str(
            format!(
                "{:021X}/{:?}/{:?}/{:?} ",
                self.player_2.quilt_board.tiles,
                self.player_2.button_balance,
                self.player_2.quilt_board.button_income,
                self.player_2.position,
            )
            .as_str(),
        );

        // 3. The current player - '0' for player 1 and '1' for player 2
        let flag = if self.is_player_1() { 0 } else { 1 };
        state.push_str(format!("{:?} ", flag).as_str());

        // 4. The patches still left to take - A list of patch ids separated by
        //    a slash starting from the first patch the current player can take
        //    or '-' if no patches are left
        if self.patches.is_empty() {
            state.push_str("- ")
        } else {
            state.push_str(
                format!(
                    "{} ",
                    self.patches
                        .iter()
                        .map(|patch| format!("{:?}", patch.id))
                        .collect::<Vec<String>>()
                        .join("/")
                )
                .as_str(),
            );
        }

        // 5. If the current move is a special patch placement move and the fitting index (one of '26', '32', '38', '44' or '50')
        //    or '-' if the special patch placement move is not active
        if let TurnType::SpecialPatchPlacement(index) = self.turn_type {
            state.push_str(format!("{:?}", index).as_str());
        } else {
            state.push('-');
        }

        Ok(state)
    }

    /// Loads the state of the game from a string.
    /// The state can be saved with `save_state`.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game as a string.
    ///
    /// # Returns
    ///
    /// The state of the game or an error if the state is invalid.
    fn load_from_notation(state: &str) -> Result<Self, PatchworkError> {
        let error = PatchworkError::InvalidNotationError {
            notation: state.to_string(),
            reason: "[Patchwork][load_from_notation] Invalid notation!",
        };

        let captures = STATE_REGEX.captures(state).ok_or(error.clone())?;

        let player_1_quilt_board = captures
            .name("player_1_quilt_board")
            .and_then(|s| u128::from_str_radix(s.as_str(), 16).ok())
            .ok_or(error.clone())?;
        let player_1_income = captures
            .name("player_1_income")
            .and_then(|s| s.as_str().parse::<isize>().ok())
            .ok_or(error.clone())?;
        let player_1_button_income = captures
            .name("player_1_button_income")
            .and_then(|s| s.as_str().parse::<isize>().ok())
            .ok_or(error.clone())?;
        let player_1_position = captures
            .name("player_1_position")
            .and_then(|s| s.as_str().parse::<usize>().ok())
            .ok_or(error.clone())?;

        let player_2_quilt_board = captures
            .name("player_2_quilt_board")
            .and_then(|s| u128::from_str_radix(s.as_str(), 16).ok())
            .ok_or(error.clone())?;
        let player_2_income = captures
            .name("player_2_income")
            .and_then(|s| s.as_str().parse::<isize>().ok())
            .ok_or(error.clone())?;
        let player_2_button_income = captures
            .name("player_2_button_income")
            .and_then(|s| s.as_str().parse::<isize>().ok())
            .ok_or(error.clone())?;
        let player_2_position = captures
            .name("player_2_position")
            .and_then(|s| s.as_str().parse::<usize>().ok())
            .ok_or(error.clone())?;

        let current_player = captures
            .name("current_player")
            .and_then(|s| s.as_str().parse::<i8>().ok())
            .map(|i| {
                if i == 0 {
                    Patchwork::PLAYER_1
                } else {
                    Patchwork::PLAYER_2
                }
            })
            .ok_or(error.clone())?;

        let patches = captures
            .name("patches")
            .map(|s| {
                let indices = s
                    .as_str()
                    .split('/')
                    .flat_map(|s| s.parse::<usize>().ok())
                    .collect::<Vec<_>>();

                let unique: HashSet<_> = indices.iter().collect();
                if unique.len() != indices.len() {
                    return Err(error.clone());
                }

                if indices.iter().any(|i| *i >= PatchManager::get_instance().patches.len()) {
                    return Err(error.clone()); // TODO: more descriptive message
                }

                Ok(indices
                    .iter()
                    .map(|patch_id| &PatchManager::get_instance().patches[*patch_id])
                    .collect::<Vec<_>>())
            })
            .ok_or(error.clone())??;

        let special_patch_placement_move = captures.name("special_patch_placement_move").and_then(|s| {
            let str = s.as_str();
            if str == "-" {
                None
            } else {
                str.parse::<usize>().ok()
            }
        });

        // Validating the state
        if player_1_position > TimeBoard::MAX_POSITION || player_2_position > TimeBoard::MAX_POSITION {
            return Err(error);
        }

        let further_player_position = player_1_position.max(player_2_position);

        if let Some(index) = special_patch_placement_move {
            // special patch placement move can only be, when a player has passed the special patch
            if index > further_player_position {
                return Err(error);
            }

            // max time cost is 6 so we can only have a special patch placement move if the player is max. 5 steps away
            if index + 5 < further_player_position {
                return Err(error);
            }
        }

        let mut time_board = TimeBoard::default();
        time_board.set_player_position(Patchwork::PLAYER_1, 0, player_1_position);
        time_board.set_player_position(Patchwork::PLAYER_2, 0, player_2_position);
        time_board.clear_special_patches_until(further_player_position);

        // TODO: write a validate_state method

        Ok(Patchwork {
            patches,
            time_board,
            player_1: PlayerState {
                position: player_1_position,
                button_balance: player_1_income,
                quilt_board: QuiltBoard {
                    tiles: player_1_quilt_board,
                    button_income: player_1_button_income,
                },
            },
            player_2: PlayerState {
                position: player_2_position,
                button_balance: player_2_income,
                quilt_board: QuiltBoard {
                    tiles: player_2_quilt_board,
                    button_income: player_2_button_income,
                },
            },
            current_player_flag: current_player,
            turn_type: special_patch_placement_move
                .map(TurnType::SpecialPatchPlacement)
                .unwrap_or(TurnType::Normal),
        })
    }
}
