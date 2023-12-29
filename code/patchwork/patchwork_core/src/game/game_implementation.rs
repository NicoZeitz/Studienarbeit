use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::{
    ActionId, GameOptions, Patch, PatchManager, Patchwork, PatchworkError, PlayerState, QuiltBoard, TimeBoard, TurnType,
};

/// The game logic for Patchwork.
impl Patchwork {
    // ────────────────────────────────────────────────── START GAME ───────────────────────────────────────────────────

    /// Gets the initial state of the game.
    ///
    /// # Arguments
    ///
    /// * `options` - The options for the game.
    ///
    /// # Returns
    ///
    /// The initial state of the game.
    pub fn get_initial_state(options: Option<GameOptions>) -> Self {
        // 1. Each player takes a quilt board, a time token and 5 buttons
        //    (as currency). Keep the remaining buttons on the table close at
        //    hand.
        let player_1 = PlayerState::default();
        let player_2 = PlayerState::default();

        // 2. Place the central time board in the middle of the table.

        // 3. Place your time tokens on the starting space of the
        //    time board. The player who last used a needle begins
        let time_board = TimeBoard::default();
        let current_player_flag = Patchwork::PLAYER_1;

        // 4. Place the (regular) patches in a circle or oval around the time
        //     board.

        // 5. Locate the smallest patch, i.e. the patch of size 1x2, and place
        //    the neutral token between this patch and the next patch in
        //    clockwise order.
        let patches = PatchManager::generate_patches(options.map(|o| o.seed));

        // # 6. Lay out the special tile

        // # 7. Place the special patches on the marked spaces of the time board

        // # 8. Now you are ready to go!
        Patchwork {
            patches,
            time_board,
            player_1,
            player_2,
            current_player_flag,
            turn_type: TurnType::Normal,
        }
    }

    // ───────────────────────────────────────────────── VALID ACTIONS ─────────────────────────────────────────────────

    /// Gets the valid actions for the current player in the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// The valid actions for the current player in the given state.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the number of valid actions.
    pub fn get_valid_actions(&self) -> Vec<ActionId> {
        // Phantom Actions - the current player is not really allowed to take a turn
        if matches!(self.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom) {
            return vec![ActionId::phantom()];
        }

        // Course of Play
        //
        // In this game, you do not necessarily alternate between turns. The
        // player whose time token is the furthest behind on the time board takes
        // his turn. This may result in a player taking multiple turns in a row
        // before his opponent can take one.
        // If both time tokens are on the same space, the player whose token is
        // on top goes first.

        // Placing a Special Patch is a special action
        if let TurnType::SpecialPatchPlacement = self.turn_type {
            return self.current_player().quilt_board.get_valid_actions_for_special_patch();
        }

        // On your turn, you carry out one of the following actions:
        let mut valid_actions: Vec<ActionId> = vec![
            // A: Advance and Receive Buttons
            ActionId::walking(self.current_player().position),
        ];

        // B: Take and Place a Patch
        valid_actions.append(&mut self.get_take_and_place_a_patch_actions());

        valid_actions
    }

    /// Gets a random action for the current player in the given state.
    ///
    /// # Returns
    ///
    /// A random action for the current player in the given state.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the number of valid actions.
    pub fn get_random_action(&self) -> ActionId {
        // PERF: more efficient implementation
        let mut valid_actions = self.get_valid_actions();
        let random_index = rand::random::<usize>() % valid_actions.len();
        valid_actions.remove(random_index)
    }

    /// Gets a random action for the current player in the given state. This
    /// function is deterministic and will always return the same action for the
    /// same state and seed.
    ///
    /// # Arguments
    ///
    /// * `seed` - The seed to use for the random number generator.
    ///
    /// # Returns
    ///
    /// A random action for the current player in the given state.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the number of valid actions.
    pub fn get_seeded_random_action(&self, seed: u64) -> ActionId {
        let mut random = Xoshiro256PlusPlus::seed_from_u64(seed);
        let mut valid_actions = self.get_valid_actions();
        let random_index = random.gen::<usize>() % valid_actions.len();
        valid_actions.remove(random_index)
    }

    /// Plays a random rollout of the game from the given state to the end and
    /// returns the resulting state as well as the last action that was taken.
    ///
    /// # Returns
    ///
    /// The resulting terminal state as well as the last action that was taken.
    /// None if the node is already terminal.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑚 · 𝑛)` where `n` is the number of valid actions every turn and `𝑚` is the amount of
    /// actions that are taken until the game is terminated.
    pub fn random_rollout(&self) -> Self {
        let mut state = self.clone();

        while !state.is_terminated() {
            let action = state.get_random_action();
            // no need to switch players every turn
            // EXPECT: ACTIONS ARE ALL VALID SO NO ERRORS CAN OCCUR
            state
                .do_action(action, false)
                .expect("[Patchwork::random_rollout] Action was not valid");
        }

        state
    }

    // ────────────────────────────────────────────── DO AND UNDO ACTIONS ──────────────────────────────────────────────

    /// Mutates the current game state by taking an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to take.
    /// * `force_player_switch` - Whether the player switch should be forced. This will result in
    /// phantom actions if the other player is not actually allowed to take a turn.
    ///
    /// # Returns
    ///
    /// Whether the action was successfully taken.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// This function has undefined when a null action is given.
    /// This will panic in debug mode
    #[allow(unused_variables)]
    pub fn do_action(&mut self, action: ActionId, force_player_switch: bool) -> Result<(), PatchworkError> {
        debug_assert!(!action.is_null(), "[Patchwork::do_action] Expected non-null action");

        // IF phantom action
        if action.is_phantom() {
            // IF the player is not switched this is a no-op
            if !force_player_switch {
                return Ok(());
            }

            // IF the player is switched we need to switch back the turn type to the previous state
            return match self.turn_type {
                TurnType::NormalPhantom => {
                    self.turn_type = TurnType::Normal;
                    self.switch_player();
                    Ok(())
                }
                TurnType::SpecialPhantom => {
                    self.turn_type = TurnType::SpecialPatchPlacement;
                    self.switch_player();
                    Ok(())
                }
                _ => Err(PatchworkError::InvalidActionError {
                    reason: "[Patchwork::do_action] Did not expect phantom action",
                    action,
                    state: Box::new(self.clone()),
                }),
            };
        }

        // IF special patch placement
        if action.is_special_patch_placement() {
            return if matches!(self.turn_type, TurnType::SpecialPatchPlacement) {
                //   1. place patch
                //      a) if the board is full the current player get +7 points
                //   2. switch player
                //   3. clear special patch
                //   4. reset the turn type to normal

                let current_player_position = self.current_player().position;

                let special_patch_index = self
                    .time_board
                    .get_special_patch_before_position(current_player_position)
                    .expect(
                        "[Patchwork::do_action] Expected special patch to be placed before current player position",
                    );

                let current_player = self.current_player_mut();
                current_player.quilt_board.do_action(action);
                if current_player.quilt_board.is_full() {
                    current_player.button_balance += QuiltBoard::FULL_BOARD_BUTTON_INCOME;
                }

                self.switch_player();

                self.time_board.unset_special_patch(special_patch_index);

                self.turn_type = TurnType::Normal;
                Ok(())
            } else {
                Err(PatchworkError::InvalidActionError {
                    reason: "[Patchwork::do_action] Did not expect special patch placement action",
                    action,
                    state: Box::new(self.clone()),
                })
            };
        }

        debug_assert!(matches!(self.turn_type, TurnType::Normal));

        let now_other_player_position = self.other_player().position;
        let now_current_player_position = self.current_player().position;
        let time_cost;

        if action.is_walking() {
            // IF walking
            //   1. add +1 to current player button balance for every tile walked over

            #[cfg(debug_assertions)]
            if now_current_player_position != action.get_starting_index() {
                let starting_index = action.get_starting_index();
                println!("{}", self);
                println!("State:\n{:?}", self);
                println!("Action: \n{:?}", action);
                println!(
                    "Starting Index {} of Walking action does not match current player position {}",
                    starting_index, now_current_player_position
                );
                debug_assert_eq!(now_current_player_position, starting_index);
            }

            let current_player = self.current_player_mut();
            time_cost = now_other_player_position - now_current_player_position + 1;

            let button_income =
                now_other_player_position.min(TimeBoard::MAX_POSITION) as i32 - now_current_player_position as i32;
            if now_current_player_position + time_cost > TimeBoard::MAX_POSITION {
                current_player.button_balance += button_income;
            } else {
                current_player.button_balance += button_income + 1;
            }
        } else {
            debug_assert!(
                action.is_patch_placement(),
                "[Patchwork::do_action] Expected patch placement action"
            );

            let patch = PatchManager::get_patch(action.get_patch_id());
            let patch_index = action.get_patch_index() as usize;

            // IF patch placement
            //  1. rollover first patches and remove patch from available patches
            //  2. subtract button cost from current player button balance
            //  3. place patch
            //      a) if the board is full the current player get +7 points
            let len = self.patches.len();
            self.patches.rotate_left(patch_index + 1);
            self.patches.remove(len - 1);

            let current_player = self.current_player_mut();

            current_player.button_balance -= patch.button_cost as i32;

            current_player.quilt_board.do_action(action);
            if current_player.quilt_board.is_full() {
                current_player.button_balance += QuiltBoard::FULL_BOARD_BUTTON_INCOME;
            }

            time_cost = patch.time_cost;
        }

        // 4. move player by time_cost
        let next_current_player_position;
        {
            let current_player = self.current_player_mut();
            current_player.position += time_cost; // allow more than max time board position to allow for undo action
            next_current_player_position = current_player.position.min(TimeBoard::MAX_POSITION);
        }

        self.time_board.move_player_position(
            self.current_player_flag,
            now_current_player_position,
            next_current_player_position,
        );
        let walking_range = (now_current_player_position as usize + 1)
            ..(next_current_player_position.min(TimeBoard::MAX_POSITION) as usize + 1);

        // 5. test if player moved over button income trigger (only a single one possible) and add button income
        {
            let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range.clone()) as i32;
            let current_player = self.current_player_mut();
            let button_income = current_player.quilt_board.button_income as i32;
            current_player.button_balance += button_income_trigger * button_income;
        }

        // 6. test if player moved over special patch (only a single one possible) and conditionally change the state
        if self.time_board.is_special_patches_in_range(walking_range) {
            // Test if special patch can even be placed
            if self.current_player().quilt_board.is_full() {
                // If not throw the special patch away and switch player
                self.switch_player();
                return Ok(());
            }

            if force_player_switch {
                self.turn_type = TurnType::SpecialPhantom;
                self.switch_player();
            } else {
                self.turn_type = TurnType::SpecialPatchPlacement;
            }

            return Ok(());
        }

        // test player position and optionally switch (always true if action.is_walking)
        if next_current_player_position > now_other_player_position {
            self.switch_player();
        } else if force_player_switch {
            self.turn_type = TurnType::NormalPhantom;
            self.switch_player();
        }

        Ok(())
    }

    /// Mutates the current game state by undoing an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to take.
    /// * `force_player_switch` - Whether the player switch should be forced. This will create/resolve phantom actions
    ///
    /// # Returns
    ///
    /// Whether the action was successfully taken.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    ///
    /// # Undefined Behavior
    ///
    /// This function has undefined if the game is in initial state or when a null action is given.
    /// This will panic in debug mode
    pub fn undo_action(&mut self, action: ActionId, force_player_switch: bool) -> Result<(), PatchworkError> {
        debug_assert!(!action.is_null(), "[Patchwork::undo_action] Expected non-null action");

        #[cfg(debug_assertions)]
        if self.current_player().position == 0 && self.other_player().position == 0 {
            return Err(PatchworkError::GameStateIsInitialError);
        }

        if action.is_phantom() {
            // IF the player is not switched this is a no-op
            if !force_player_switch {
                return Ok(());
            }

            return match self.turn_type {
                TurnType::Normal => {
                    self.turn_type = TurnType::NormalPhantom;
                    self.switch_player();
                    Ok(())
                }
                TurnType::SpecialPatchPlacement => {
                    self.turn_type = TurnType::SpecialPhantom;
                    self.switch_player();
                    Ok(())
                }
                _ => Err(PatchworkError::InvalidActionError {
                    reason: "[Patchwork::undo_action] Did not expect phantom action",
                    action,
                    state: Box::new(self.clone()),
                }),
            };
        }

        if action.is_walking() {
            //   1. subtract +1 from current player button balance for every tile walked over
            //   2. move player back by time_cost
            //   3. test if player moved over button income trigger (only a single one possible) and subtract button income
            //   4. switch player (as it is always the other player's turn after a walking action)

            if matches!(self.turn_type, TurnType::NormalPhantom) && !self.is_terminated() {
                return Err(PatchworkError::InvalidActionError {
                    reason: "[Patchwork::undo_action] Did not expect walking action",
                    action,
                    state: Box::new(self.clone()),
                });
            }

            if (self.current_player().position < TimeBoard::MAX_POSITION
                && !matches!(self.turn_type, TurnType::SpecialPatchPlacement))
                || force_player_switch
            {
                self.switch_player();
            }
            self.turn_type = TurnType::Normal;

            let previous_other_player_position = self.other_player().position.min(TimeBoard::MAX_POSITION) as usize;
            let starting_index = action.get_starting_index().min(TimeBoard::MAX_POSITION);

            let time_cost = if previous_other_player_position >= TimeBoard::MAX_POSITION as usize {
                previous_other_player_position as i32 - starting_index as i32
            } else {
                previous_other_player_position as i32 - starting_index as i32 + 1
            };

            let now_current_player_position;
            {
                let current_player = self.current_player_mut();
                now_current_player_position = current_player.position;
                current_player.button_balance -= time_cost;
                current_player.position = starting_index;
            }

            {
                let walking_range = (starting_index as usize + 1)
                    ..(now_current_player_position.min(TimeBoard::MAX_POSITION) as usize + 1);
                let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range) as i32;
                let current_player = self.current_player_mut();
                let button_income = current_player.quilt_board.button_income as i32;
                current_player.button_balance -= button_income_trigger * button_income;
            }

            self.time_board
                .move_player_position(self.current_player_flag, now_current_player_position, starting_index);

            return Ok(());
        }

        if action.is_patch_placement() {
            let patch_id = action.get_patch_id();
            let patch = PatchManager::get_patch(patch_id);
            let patch_index = action.get_patch_index() as usize;

            self.turn_type = TurnType::Normal;
            self.patches.push(patch);
            self.patches.rotate_right(patch_index + 1);

            // Player needs to be switched when:
            // 1. Force player switch
            // 2. Previous player is not the same as the current player
            let previous_player_1 = action.get_previous_player_was_1();
            let other_previous_player =
                self.is_player_1() && !previous_player_1 || self.is_player_2() && previous_player_1;

            if force_player_switch || other_previous_player {
                self.switch_player();
            }

            let previous_current_player_position = self.current_player().position - patch.time_cost;

            let now_current_player_position;
            {
                let current_player = self.current_player_mut();
                now_current_player_position = current_player.position;
                if current_player.quilt_board.is_full() {
                    current_player.button_balance -= QuiltBoard::FULL_BOARD_BUTTON_INCOME;
                }

                current_player.button_balance += patch.button_cost as i32;
                current_player.position = previous_current_player_position;
            }
            {
                let walking_range = (previous_current_player_position as usize + 1)
                    ..((previous_current_player_position + patch.time_cost).min(TimeBoard::MAX_POSITION) as usize + 1);
                let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range) as i32;
                let current_player = self.current_player_mut();
                let button_income = current_player.quilt_board.button_income as i32;
                current_player.button_balance -= button_income_trigger * button_income;
            }

            self.current_player_mut().quilt_board.undo_action(action);

            self.time_board.move_player_position(
                self.current_player_flag,
                now_current_player_position,
                previous_current_player_position,
            );

            return Ok(());
        }

        // special patch placement
        debug_assert!(
            action.is_special_patch_placement(),
            "[Patchwork::undo_action] Expected special patch placement action"
        );

        self.switch_player();

        let special_patch_index = self
            .time_board
            .get_special_patch_before_position(self.current_player().position)
            .expect("[Patchwork::undo_action] Expected special patch to be placed before current player position");

        self.turn_type = TurnType::SpecialPatchPlacement;
        self.time_board.set_special_patch(special_patch_index);

        if self.current_player().quilt_board.is_full() {
            self.current_player_mut().button_balance -= QuiltBoard::FULL_BOARD_BUTTON_INCOME;
        }

        self.current_player_mut().quilt_board.undo_action(action);

        Ok(())
    }

    // ─────────────────────────────────────────── DO AND UNDO NULL ACTIONS ────────────────────────────────────────────

    // TODO: null actions (get_valid_null_actions, do_null_action, undo_null_action)

    // ──────────────────────────────────────────────────── GETTERS ────────────────────────────────────────────────────

    /// Gets the current player.
    ///
    /// # Returns
    ///
    /// The current player. '1' for player 1 and '-1' for player 2.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn get_current_player(&self) -> i8 {
        self.current_player_flag
    }

    /// Gets whether the given state is terminated. This is true if both players are at the end of the time board.
    ///
    /// # Returns
    ///
    /// Whether the game associated with the given state is terminated.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline(always)]
    pub const fn is_terminated(&self) -> bool {
        let player_1_position = self.player_1.position;
        let player_2_position = self.player_2.position;

        player_1_position >= TimeBoard::MAX_POSITION && player_2_position >= TimeBoard::MAX_POSITION
    }

    // ─────────────────────────────────────────────── UTILITY FUNCTIONS ───────────────────────────────────────────────

    /// Get the valid moves for the action "Take and Place a Patch"
    ///
    /// # Arguments
    ///
    /// * `state` - the current state (will not be modified)
    ///
    /// # Returns
    ///
    /// a list of all valid next states
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `n` is the number of valid actions.
    #[inline]
    fn get_take_and_place_a_patch_actions(&self) -> Vec<ActionId> {
        return self
            .patches
            .iter()
            .take(PatchManager::MAX_AMOUNT_OF_CHOOSABLE_TILES as usize)
            .enumerate()
            .filter(|patch| self.can_player_take_patch(self.current_player(), patch.1))
            .flat_map(|(index, patch)| {
                self.current_player()
                    .quilt_board
                    .get_valid_actions_for_patch(patch, index as u8, self.is_player_1())
            })
            .collect();
    }

    /// Performance fastpath for checking if a player can take a patch
    /// and avoiding costly calculations.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    /// * `patch` - The patch to take.
    ///
    /// # Returns
    ///
    /// Whether the player can take the patch.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[inline]
    fn can_player_take_patch(&self, player: &PlayerState, patch: &Patch) -> bool {
        // player can only place pieces that they can afford
        if patch.button_cost as i32 > player.button_balance {
            return false;
        }

        // player can only place pieces that fit on their board (fastpath)
        if player.quilt_board.tiles_free() < patch.amount_tiles() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::Notation;
    use pretty_assertions::assert_eq;
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoshiro256PlusPlus;

    use super::*;

    const ITERATIONS: usize = 10_000;

    #[test]
    fn test_walking_action_at_start() {
        let mut state = Patchwork::get_initial_state(None);
        let old_state = state.clone();

        let action = ActionId::walking(0);

        state.do_action(action, false).unwrap();

        assert_eq!(
            state.player_1.position, 1,
            "Player 1 position changed wrong in do action"
        );
        assert_eq!(
            state.player_2.position, 0,
            "Player 2 position changed wrong in do action"
        );

        state.undo_action(action, false).unwrap();

        assert_eq!(
            state.current_player().position,
            0,
            "Current player position changed wrong in undo action"
        );
        assert_eq!(
            state.other_player().position,
            0,
            "Other player position changed in undo action"
        );
        assert_eq!(old_state, state, "Old State != Restored State");
    }

    #[test]
    fn test_undo_redo_actions_force_swap() {
        for i in 0..ITERATIONS {
            test_undo_redo_actions(true, i as u64);
        }
    }

    #[test]
    fn test_undo_redo_actions_normal_swap() {
        for i in 0..ITERATIONS {
            test_undo_redo_actions(false, i as u64);
        }
    }

    fn test_undo_redo_actions(force_swap: bool, seed: u64) {
        println!(
            "────────────── Testing undo/redo actions with force_swap = {}, seed = {} ──────────────",
            force_swap, seed
        );

        let mut state = Patchwork::get_initial_state(Some(GameOptions { seed }));

        let mut actions = VecDeque::new();
        let mut states = VecDeque::new();
        let mut random = Xoshiro256PlusPlus::seed_from_u64(seed);

        let mut iteration = 0;

        while !state.is_terminated() {
            let mut valid_actions: Vec<ActionId> = state.get_valid_actions();
            let random_index = random.gen::<usize>() % valid_actions.len();
            let action = valid_actions.remove(random_index);

            println!(
                "{: >2}: {} → {}",
                iteration,
                state.save_to_notation_with_phantom_state(true).unwrap(),
                action.save_to_notation().unwrap_or("Not Displayable".to_string())
            );

            let cloned_state = state.clone();

            state.do_action(action, force_swap).unwrap();

            actions.push_back(action);
            states.push_back(cloned_state);

            iteration += 1;
        }

        println!(
            "{: >2}: {} = GAME END",
            iteration,
            state.save_to_notation_with_phantom_state(true).unwrap(),
        );

        while let Some(action) = actions.pop_back() {
            let old_state = states.pop_back().unwrap();

            state
                .undo_action(action, force_swap)
                .map_err(|e| println!("{:?}", e))
                .unwrap();

            iteration -= 1;

            assert_eq!(
                old_state,
                state,
                "Old State != Restored State, Undo action {:?} failed at iteration {}",
                action.save_to_notation().unwrap_or("Not Displayable".to_string()),
                iteration
            );
        }
    }
}

#[cfg(test)]
mod history_tests {
    use std::{fs::OpenOptions, io::Write};

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    struct Game {
        pub turns: Vec<GameTurn>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    struct GameTurn {
        pub state: Patchwork,
        pub action: Option<ActionId>,
    }

    #[test]
    #[ignore]
    fn record_normal_games() {
        // values are chosen so that the files are smaller than 100.000 KiB
        record_games("normal.game.bin", false, 16_294);
    }

    #[test]
    #[ignore]
    fn record_force_swap_games() {
        record_games("force_swap.game.bin", true, 12_722);
    }

    fn record_games(file_name: &str, force_swap: bool, amount_of_games_to_capture: usize) {
        println!("────────────── Recording games to {} ──────────────", file_name);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("src/game/tests/history/{}", file_name))
            .unwrap();

        let mut games = Vec::<Game>::new();

        for i in 0..amount_of_games_to_capture {
            let mut game = Game {
                turns: Vec::<GameTurn>::new(),
            };

            let mut state = Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 }));
            let mut random = Xoshiro256PlusPlus::seed_from_u64(i as u64);

            while !state.is_terminated() {
                let mut valid_actions: Vec<ActionId> = state.get_valid_actions();
                let random_index = random.gen::<usize>() % valid_actions.len();
                let action = valid_actions.remove(random_index);

                game.turns.push(GameTurn {
                    state: state.clone(),
                    action: Some(action),
                });

                state.do_action(action, force_swap).unwrap();
            }

            game.turns.push(GameTurn {
                state: state.clone(),
                action: None,
            });

            games.push(game);
        }

        let encoded: Vec<u8> = bincode::serialize(&games).unwrap();
        file.write_all(&encoded).unwrap();
    }

    #[test]
    fn replay_normal_games() {
        replay_games("normal.game.bin", false);
    }

    #[test]
    fn replay_force_swap_games() {
        replay_games("force_swap.game.bin", true);
    }

    fn replay_games(file_name: &str, force_swap: bool) {
        println!("────────────── Replaying games from {} ──────────────", file_name);
        let file = OpenOptions::new()
            .read(true)
            .open(format!("src/game/tests/history/{}", file_name))
            .unwrap();

        let games: Vec<Game> = bincode::deserialize_from(file).unwrap();
        for (i, game) in games.iter().enumerate() {
            println!("────────────── Replaying game {} ──────────────", i);
            let mut state = Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 }));

            for (j, turn) in game.turns.iter().enumerate() {
                println!("────────────── Replaying turn {} ──────────────", j);
                assert_eq!(state, turn.state, "State does not match");
                if let Some(action) = turn.action {
                    let valid_actions = state.get_valid_actions();

                    assert!(
                        valid_actions.contains(&action),
                        "Action {:?} is not valid in state {:?}",
                        action,
                        state
                    );

                    state.do_action(action, force_swap).unwrap();
                }
            }
        }
    }
}

#[cfg(feature = "performance_tests")]
#[allow(clippy::redundant_closure_call)]
#[allow(dead_code)]
#[allow(unused_macros)]
#[allow(unused_imports)]
mod performance_tests {

    use std::time::{Duration, Instant};

    use super::*;

    const ITERATIONS: usize = 1000;

    //             1 ns
    //         1_000 ns = 1 µs
    //     1_000_000 ns = 1 ms
    // 1_000_000_000 ns = 1 s

    const GET_INITIAL_STATE_THRESHOLD: u128 = 500_000;
    const GET_VALID_ACTIONS_THRESHOLD: u128 = 200_000;
    const GET_RANDOM_ACTION_THRESHOLD: u128 = 200_000;
    const DO_ACTION_THRESHOLD: u128 = 12_000;
    const UNDO_ACTION_THRESHOLD: u128 = 12_000;
    const CLONE_THRESHOLD: u128 = 12_000;

    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);

            // Find and cut the rest of the path
            match &name[..name.len() - 3].rfind(':') {
                Some(pos) => &name[pos + 1..name.len() - 3],
                None => &name[..name.len() - 3],
            }
        }};
    }

    macro_rules! run_function(
        ($iterations:expr, $threshold:expr, $setup:expr, $function:expr) => {
            let mut max = Duration::new(0, 0);
            let mut min = Duration::new(u64::MAX, 1_000_000_000 - 1);
            let mut sum = Duration::new(0, 0);

            for i in 0..$iterations {
                let setup = $setup(i);

                let start = Instant::now();

                $function(setup);

                let duration = start.elapsed();
                if duration.as_nanos() > $threshold {
                    panic!("PERFORMANCE REGRESSION: {} took: {:?}", function!(), duration);
                }

                if duration > max {
                    max = duration;
                }
                if duration < min {
                    min = duration;
                }
                sum += duration;
            }

            let avg = sum / ITERATIONS as u32;
            println!(
                "\n{:<40} ─ {:>9?} (max), {:>9?} (avg), {:>9?} (min) ─ that would be [{:>8}, {:>8}, {:>8}] calls per second",
                function!(),
                max,
                avg,
                min,
                (1.0 / max.as_secs_f64()).round() as i64,
                (1.0 / avg.as_secs_f64()).round() as i64,
                (1.0 / min.as_secs_f64()).round() as i64,
            );
        };
    );

    #[test]
    fn get_initial_state() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            GET_INITIAL_STATE_THRESHOLD,
            |i: usize| { Some(GameOptions { seed: i as u64 }) },
            |args| { Patchwork::get_initial_state(args) }
        );
    }

    #[test]
    fn get_valid_actions() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            GET_VALID_ACTIONS_THRESHOLD,
            |i| { Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 })) },
            |patchwork: Patchwork| { patchwork.get_valid_actions() }
        );
    }

    #[test]
    fn get_random_action() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            GET_RANDOM_ACTION_THRESHOLD,
            |i| { Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 })) },
            |patchwork: Patchwork| { patchwork.get_random_action() }
        );
    }

    #[test]
    #[allow(unused)]
    fn do_action() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            DO_ACTION_THRESHOLD,
            |i| {
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 }));
                for _ in 0..(i % 20) {
                    patchwork.do_action(patchwork.get_random_action(), false);
                }
                let action = patchwork.get_random_action();
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, ActionId)| {
                let mut patchwork = patchwork;
                patchwork.do_action(action, false)
            }
        );
    }

    #[test]
    #[allow(unused)]
    fn undo_action() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            UNDO_ACTION_THRESHOLD,
            |i| {
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 }));
                for _ in 0..(i % 20) {
                    patchwork.do_action(patchwork.get_random_action(), false);
                }
                let action = patchwork.get_random_action();
                patchwork.do_action(action, false);
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, ActionId)| {
                let mut patchwork = patchwork;
                patchwork.undo_action(action, false)
            }
        );
    }

    #[test]
    fn clone() {
        verify_performance_args();

        run_function!(
            ITERATIONS,
            CLONE_THRESHOLD,
            |i| { Patchwork::get_initial_state(Some(GameOptions { seed: i as u64 })) },
            |patchwork: Patchwork| { patchwork.clone() }
        );
    }

    fn verify_performance_args() {
        if cfg!(debug_assertions) {
            panic!("Performance tests should be run with --release");
        }
    }
}
