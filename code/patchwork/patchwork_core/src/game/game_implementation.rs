use crate::{
    Action, ActionPayload, GameOptions, Patch, PatchManager, Patchwork, PatchworkError, PlayerState, QuiltBoard,
    TimeBoard, TurnType,
};

/// The game logic for Patchwork.
impl Patchwork {
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
        let patches = PatchManager::get_instance().generate_patches(options.map(|o| o.seed));

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

    /// Gets the valid actions for the current player in the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// The valid actions for the current player in the given state.
    pub fn get_valid_actions(&self) -> Vec<Action> {
        // Null Actions - the current player is not really allowed to take a turn
        if matches!(self.turn_type, TurnType::NormalPhantom | TurnType::SpecialPhantom(_)) {
            return vec![Action::null()];
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
        if let TurnType::SpecialPatchPlacement(special_patch_index) = self.turn_type {
            let special_patch = PatchManager::get_instance().get_special_patch(special_patch_index);
            return self
                .current_player()
                .quilt_board
                .get_valid_actions_for_special_patch(special_patch);
        }

        // On your turn, you carry out one of the following actions:
        let mut valid_actions: Vec<Action> = vec![
            // A: Advance and Receive Buttons
            Action::walking(self.current_player().position),
        ];

        // B: Take and Place a Patch
        valid_actions.append(&mut self.get_take_and_place_a_patch_actions(self.current_player().position));

        valid_actions
    }

    /// Gets a random action for the current player in the given state.
    ///
    /// # Returns
    ///
    /// A random action for the current player in the given state.
    pub fn get_random_action(&self) -> Action {
        // TODO: more efficient implementation
        let mut valid_actions = self.get_valid_actions();
        let random_index = rand::random::<usize>() % valid_actions.len();
        valid_actions.remove(random_index)
    }

    /// Plays a random rollout of the game from the given state to the end and returns the resulting state as well as the last action that was taken.
    ///
    /// # Returns
    ///
    /// The resulting terminal state as well as the last action that was taken. None if the node is already terminal.
    pub fn random_rollout(&self) -> Self {
        let mut state = self.clone();

        while !state.is_terminated() {
            let action = state.get_random_action();
            // no need to switch players every turn, actions are all valid so no errors can occur
            state.do_action(&action, false).unwrap();
        }

        state
    }

    /// Gets the current player.
    ///
    /// # Returns
    ///
    /// The current player. '1' for player 1 and '-1' for player 2.
    pub fn get_current_player(&self) -> i8 {
        self.current_player_flag
    }

    /// Mutates the current game state by taking an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to take.
    /// * `force_player_switch` - Whether the player switch should be forced. This will result in null actions if the other player is not actually allowed to take a turn.
    ///
    /// # Returns
    ///
    /// Whether the action was successfully taken.
    pub fn do_action(&mut self, action: &Action, force_player_switch: bool) -> Result<(), PatchworkError> {
        // IF null action
        if let ActionPayload::Null = action.payload {
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
                TurnType::SpecialPhantom(index) => {
                    self.turn_type = TurnType::SpecialPatchPlacement(index);
                    self.switch_player();
                    Ok(())
                }
                _ => Err(PatchworkError::InvalidActionError {
                    reason: "Did not expect null action",
                    action: Box::new(action.clone()),
                }),
            };
        }

        // IF special patch placement
        if let ActionPayload::SpecialPatchPlacement {
            #[cfg(debug_assertions)]
            patch_id,
            ..
        } = action.payload
        {
            return if let TurnType::SpecialPatchPlacement(special_patch_index) = self.turn_type {
                #[cfg(debug_assertions)]
                let index = PatchManager::get_instance().get_position_from_special_patch_id(patch_id);
                #[cfg(debug_assertions)]
                debug_assert_eq!(special_patch_index, index);
                //   1. place patch
                //      a) if the board is full the current player get +7 points
                //   2. switch player
                //   3. clear special patch
                //   4. reset the turn type to normal

                let current_player = self.current_player_mut();
                current_player.quilt_board.do_action(action);
                if current_player.quilt_board.is_full() {
                    current_player.button_balance += 7;
                }
                self.switch_player();
                self.time_board.clear_special_patch(special_patch_index);
                self.turn_type = TurnType::Normal;
                Ok(())
            } else {
                Err(PatchworkError::InvalidActionError {
                    reason: "Did not expect special patch placement action",
                    action: Box::new(action.clone()),
                })
            };
        }

        debug_assert!(matches!(self.turn_type, TurnType::Normal));

        let now_other_player_position = self.other_player().position;
        let now_current_player_position = self.current_player().position;
        let mut time_cost = 0;

        match action.payload {
            // IF walking
            ActionPayload::Walking { starting_index } => {
                debug_assert!(now_current_player_position == starting_index);

                //   1. add +1 to current player button balance for every tile walked over
                let current_player = self.current_player_mut();
                time_cost = now_other_player_position - now_current_player_position + 1;
                current_player.button_balance += time_cost as i32;
            }
            // IF patch placement
            ActionPayload::PatchPlacement { patch, patch_index, .. } => {
                //  1. rollover first patches and remove patch from available patches
                //  2. subtract button cost from current player button balance
                //  3. place patch
                //      a) if the board is full the current player get +7 points
                self.patches.rotate_left(patch_index + 1);
                self.patches.remove(self.patches.len() - 1);

                let current_player = self.current_player_mut();
                current_player.button_balance -= patch.button_cost as i32;
                current_player.quilt_board.do_action(action);
                if current_player.quilt_board.is_full() {
                    current_player.button_balance += 7;
                }

                time_cost = patch.time_cost;
            }
            _ => {
                // Cases Warnings
                // unreachable!("[Patchwork][do_action] Null and Walking Actions are already handled above");
            }
        }

        // 4. move player by time_cost
        let next_current_player_position;
        {
            let current_player = self.current_player_mut();
            current_player.position = (current_player.position + time_cost).min(TimeBoard::MAX_POSITION);
            next_current_player_position = current_player.position;
        }

        self.time_board.set_player_position(
            self.current_player_flag,
            now_current_player_position,
            next_current_player_position,
        );
        let walking_range = (now_current_player_position + 1)..(next_current_player_position + 1);

        // 5. test if player moved over button income trigger (only a single one possible) and add button income
        {
            let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range.clone()) as i32;
            let current_player = self.current_player_mut();
            let button_income = current_player.quilt_board.button_income;
            current_player.button_balance += button_income_trigger * button_income;
        }

        // 6. test if player moved over special patch (only a single one possible) and conditionally change the state
        let special_patch = self.time_board.get_special_patch_in_range(walking_range);
        if let Some(special_patch_index) = special_patch {
            // Test if special patch can even be placed
            if self.current_player().quilt_board.is_full() {
                // If not throw the special patch away and switch player
                self.switch_player();
                return Ok(());
            }

            if force_player_switch {
                self.turn_type = TurnType::SpecialPhantom(special_patch_index);
                self.switch_player();
            } else {
                self.turn_type = TurnType::SpecialPatchPlacement(special_patch_index);
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

    pub fn undo_action(&mut self, action: &Action, force_player_switch: bool) -> Result<(), PatchworkError> {
        if self.current_player().position == 0 && self.other_player().position == 0 {
            return Err(PatchworkError::GameStateIsInitialError);
        }

        match action.payload {
            ActionPayload::Null => {
                // IF the player is not switched this is a no-op
                if !force_player_switch {
                    return Ok(());
                }

                match self.turn_type {
                    TurnType::Normal => {
                        self.turn_type = TurnType::NormalPhantom;
                        self.switch_player();
                        Ok(())
                    }
                    TurnType::SpecialPatchPlacement(index) => {
                        self.turn_type = TurnType::SpecialPhantom(index);
                        self.switch_player();
                        Ok(())
                    }
                    _ => Err(PatchworkError::InvalidActionError {
                        reason: "Did not expect null action",
                        action: Box::new(action.clone()),
                    }),
                }
            }
            // IF walking
            ActionPayload::Walking { starting_index } => {
                //   1. subtract +1 from current player button balance for every tile walked over
                //   2. move player back by time_cost
                //   3. test if player moved over button income trigger (only a single one possible) and subtract button income
                //   4. switch player (as it is always the other player's turn after a walking action)

                if matches!(self.turn_type, TurnType::NormalPhantom) && !self.is_terminated() {
                    return Err(PatchworkError::InvalidActionError {
                        reason: "Did not expect walking action",
                        action: Box::new(action.clone()),
                    });
                }

                if force_player_switch
                    || (self.current_player().position != TimeBoard::MAX_POSITION
                        && !matches!(self.turn_type, TurnType::SpecialPatchPlacement(_)))
                {
                    self.switch_player();
                }
                self.turn_type = TurnType::Normal;

                let previous_other_player_position = self.other_player().position;
                let time_cost = previous_other_player_position - starting_index + 1;

                let now_current_player_position;
                {
                    let current_player = self.current_player_mut();
                    now_current_player_position = current_player.position;
                    current_player.button_balance -= time_cost as i32;
                    current_player.position = starting_index;
                }

                {
                    let walking_range = (starting_index + 1)..(now_current_player_position + 1);
                    let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range) as i32;
                    let current_player = self.current_player_mut();
                    let button_income = current_player.quilt_board.button_income;
                    current_player.button_balance -= button_income_trigger * button_income;
                }

                self.time_board.set_player_position(
                    self.current_player_flag,
                    now_current_player_position,
                    starting_index,
                );

                Ok(())
            }
            ActionPayload::PatchPlacement {
                patch,
                patch_index,
                starting_index: previous_current_player_position,
                ..
            } => {
                self.turn_type = TurnType::Normal;
                self.patches.push(patch);
                self.patches.rotate_right(patch_index + 1);

                // Use overflowing sub as the current players position can be be for example 1
                // while the other player is at position 2 and bought a 2 time cost patch last turn
                if force_player_switch
                    || (self.current_player().position.overflowing_sub(patch.time_cost)
                        != (previous_current_player_position, false)
                        && self.current_player().position != TimeBoard::MAX_POSITION)
                {
                    self.switch_player();
                }

                let now_current_player_position;
                {
                    let current_player = self.current_player_mut();
                    now_current_player_position = current_player.position;
                    if current_player.quilt_board.is_full() {
                        current_player.button_balance -= 7;
                    }

                    current_player.button_balance += patch.button_cost as i32;
                    current_player.position = previous_current_player_position;
                }
                {
                    let walking_range = (previous_current_player_position + 1)
                        ..(previous_current_player_position + patch.time_cost + 1);
                    let button_income_trigger = self.time_board.is_button_income_trigger_in_range(walking_range) as i32;
                    let current_player = self.current_player_mut();
                    let button_income = current_player.quilt_board.button_income;
                    current_player.button_balance -= button_income_trigger * button_income;
                }

                self.current_player_mut().quilt_board.undo_action(action);

                self.time_board.set_player_position(
                    self.current_player_flag,
                    now_current_player_position,
                    previous_current_player_position,
                );

                Ok(())
            }
            ActionPayload::SpecialPatchPlacement { patch_id, .. } => {
                let special_patch_index = PatchManager::get_instance().get_position_from_special_patch_id(patch_id);
                self.turn_type = TurnType::SpecialPatchPlacement(special_patch_index);

                self.switch_player();

                self.time_board.set_special_patch(special_patch_index);

                self.current_player_mut().quilt_board.undo_action(action);
                if self.current_player().quilt_board.is_full() {
                    self.current_player_mut().button_balance -= 7;
                }
                Ok(())
            }
        }
    }

    /// Gets whether the given state is terminated. This is true if both players are at the end of the time board.
    ///
    /// # Returns
    ///
    /// Whether the game associated with the given state is terminated.
    pub fn is_terminated(&self) -> bool {
        let player_1_position = self.player_1.position;
        let player_2_position = self.player_2.position;

        player_1_position >= TimeBoard::MAX_POSITION && player_2_position >= TimeBoard::MAX_POSITION
    }

    /// Get the valid moves for the action "Take and Place a Patch"
    ///
    /// # Arguments
    ///
    /// * `state` - the current state (will not be modified)
    ///
    /// # Returns
    ///
    /// a list of all valid next states
    #[inline]
    fn get_take_and_place_a_patch_actions(&self, starting_index: usize) -> Vec<Action> {
        return self
            .patches
            .iter()
            .take(3)
            .enumerate()
            .filter(|patch| self.can_player_take_patch(self.current_player(), patch.1))
            .flat_map(|(index, patch)| {
                self.current_player()
                    .quilt_board
                    .get_valid_actions_for_patch(patch, index, starting_index)
            })
            .collect();
    }

    /// PERF: Fastpath for checking if a player can take a patch and avoiding costly calculations.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    /// * `patch` - The patch to take.
    ///
    /// # Returns
    ///
    /// Whether the player can take the patch.
    #[inline]
    fn can_player_take_patch(&self, player: &PlayerState, patch: &Patch) -> bool {
        // player can only place pieces that they can afford
        if patch.button_cost as i32 > player.button_balance {
            return false;
        }

        // player can only place pieces that fit on their board (fastpath)
        if QuiltBoard::TILES as u32 - player.quilt_board.tiles_filled() < patch.amount_tiles() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod functional_tests {
    use std::collections::VecDeque;

    use crate::Notation;
    use pretty_assertions::assert_eq;
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoshiro256PlusPlus;

    use super::*;

    const ITERATIONS: usize = 1000;

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
        println!("Testing undo/redo actions with force_swap = {}", force_swap);

        let mut state = Patchwork::get_initial_state(Some(GameOptions { seed }));

        let mut actions = VecDeque::new();
        let mut states = VecDeque::new();
        let mut random = Xoshiro256PlusPlus::seed_from_u64(seed);

        while !state.is_terminated() {
            let mut valid_actions: Vec<Action> = state.get_valid_actions();
            let random_index = random.gen::<usize>() % valid_actions.len();
            let action = valid_actions.remove(random_index);

            println!(
                "{:?} --> {:?}",
                state.save_to_notation().unwrap_or("Not Displayable".to_string()),
                action.save_to_notation().unwrap_or("Not Displayable".to_string())
            );

            let cloned_state = state.clone();

            state.do_action(&action, force_swap).unwrap();

            actions.push_back(action);
            states.push_back(cloned_state);
        }

        println!(
            "{:?}",
            state.save_to_notation().unwrap_or("Not Displayable".to_string())
        );

        while let Some(action) = actions.pop_back() {
            let old_state = states.pop_back().unwrap();

            state
                .undo_action(&action, force_swap)
                .map_err(|e| println!("{:?}", e))
                .unwrap();

            assert_eq!(
                state,
                old_state,
                "Restored state != Old State, Undo action {:?} failed",
                action.save_to_notation().unwrap_or("Not Displayable".to_string())
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::redundant_closure_call)]
mod performance_tests {

    use std::time::{Duration, Instant};

    use super::*;

    const ITERATIONS: usize = 1000;

    // TODO: real values
    const GET_INITIAL_STATE_THRESHOLD: u128 = 30_000_000;
    const GET_VALID_ACTIONS_THRESHOLD: u128 = 30_000_000;
    const GET_RANDOM_ACTION_THRESHOLD: u128 = 30_000_000;
    const DO_ACTION_THRESHOLD: u128 = 30_000_000;
    const UNDO_ACTION_THRESHOLD: u128 = 30_000_000;
    const CLONE_THRESHOLD: u128 = 30_000_000;

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
                    patchwork.do_action(&patchwork.get_random_action(), false);
                }
                let action = patchwork.get_random_action();
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, Action)| {
                let mut patchwork = patchwork;
                patchwork.do_action(&action, false)
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
                    patchwork.do_action(&patchwork.get_random_action(), false);
                }
                let action = patchwork.get_random_action();
                patchwork.do_action(&action, false);
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, Action)| {
                let mut patchwork = patchwork;
                patchwork.undo_action(&action, false)
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
            panic!("Performance tests should be run with --release\ncargo test --release --package patchwork_core performance_tests -- --nocapture --color always");
        }
    }
}
