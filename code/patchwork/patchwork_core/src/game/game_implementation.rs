use crate::{
    Action, ActionPatchPlacementPayload, ActionPayload, ActionSpecialPatchPlacementPayload, Game,
    GameOptions, Patch, PatchManager, Patchwork, PlayerState, QuiltBoard, TimeBoard,
};

/// The game logic for Patchwork.
impl Game for Patchwork {
    type GameOptions = Option<GameOptions>;
    type Action = Action;
    type ActionList = Vec<Action>;
    type Player = i8;

    fn get_initial_state(options: Self::GameOptions) -> Self {
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
            special_patch_placement_move: None,
        }
    }

    fn get_valid_actions(&self) -> Vec<Action> {
        // Course of Play
        //
        // In this game, you do not necessarily alternate between turns. The
        // player whose time token is the furthest behind on the time board takes
        // his turn. This may result in a player taking multiple turns in a row
        // before his opponent can take one.
        // If both time tokens are on the same space, the player whose token is
        // on top goes first.

        // Placing a Special Patch is a special action
        if let Some(special_patch_placement_move) = self.special_patch_placement_move {
            let special_patch =
                PatchManager::get_instance().get_special_patch(special_patch_placement_move);
            return self
                .current_player()
                .quilt_board
                .get_valid_actions_for_special_patch(special_patch);
        }

        // On your turn, you carry out one of the following actions:
        let mut valid_actions: Vec<Action> = vec![
            // A: Advance and Receive Buttons
            Action::walking(),
        ];

        // B: Take and Place a Patch
        valid_actions.append(&mut self.get_take_and_place_a_patch_actions());

        valid_actions
    }

    fn get_random_action(&self) -> Action {
        // TODO: more efficient implementation
        let valid_actions = self.get_valid_actions();
        let random_index = rand::random::<usize>() % valid_actions.len();
        valid_actions[random_index].clone()
    }

    fn get_current_player(&self) -> Self::Player {
        self.current_player_flag
    }

    fn get_next_state(&self, action: &Action) -> Patchwork {
        let mut new_state = self.clone(); // TODO: check if we copy too much

        // IF special patch
        //   1. place patch
        //      a) if the board is full the current player get +7 points
        //   2. switch player
        //   3. reset special patch state
        if let ActionPayload::SpecialPatchPlacement {
            payload:
                ActionSpecialPatchPlacementPayload {
                    patch_id: _,
                    row: _,
                    column: _,
                    next_quilt_board,
                },
        } = action.payload
        {
            if new_state.special_patch_placement_move.is_none() {
                // FIXME: Better error handling
                panic!("Invalid action for special patch placement")
            }

            let current_player = new_state.current_player_mut();
            current_player.quilt_board.update(next_quilt_board, 0);
            if current_player.quilt_board.is_full() {
                current_player.button_balance += 7;
            }
            new_state.switch_player();
            new_state.special_patch_placement_move = None;
            return new_state;
        }

        let other_player_position = new_state.other_player().position;
        let old_current_player_position = new_state.current_player().position;
        let mut time_cost = 0;

        match action.payload {
            // IF walking
            //   1. add +1 to current player button balance for every tile walked over
            ActionPayload::Walking => {
                let current_player = new_state.current_player_mut();
                time_cost = other_player_position - old_current_player_position + 1;
                current_player.button_balance += time_cost as i32;
            }
            // IF patch placement
            //  1. place patch
            //  2. rollover first patches and remove patch from available patches
            //  3. subtract button cost from current player button balance
            //      a) if the board is full the current player get +7 points
            ActionPayload::PatchPlacement {
                payload:
                    ActionPatchPlacementPayload {
                        patch,
                        patch_index,
                        patch_rotation: _,
                        patch_orientation: _,
                        row: _,
                        column: _,
                        next_quilt_board,
                    },
            } => {
                new_state.patches.rotate_left(patch_index + 1_usize);
                new_state.patches.remove(new_state.patches.len() - 1);

                let current_player = new_state.current_player_mut();
                current_player
                    .quilt_board
                    .update(next_quilt_board, patch.button_income as i32);
                current_player.button_balance -= patch.button_cost as i32;
                if current_player.quilt_board.is_full() {
                    current_player.button_balance += 7;
                }

                time_cost = patch.time_cost;
            }
            _ => {}
        }

        // 4. move player by time_cost
        let new_current_player_position = {
            let current_player = new_state.current_player_mut();
            current_player.position += time_cost;
            current_player.position
        };
        new_state.time_board.set_player_position(
            self.current_player_flag,
            old_current_player_position,
            new_current_player_position,
        );

        let walking_range = (old_current_player_position + 1)..(new_current_player_position + 1);

        // 5. test if player moved over button income trigger (multiple possible) and add button income
        {
            let button_income_triggers = new_state
                .time_board
                .get_amount_button_income_triggers_in_range(&walking_range);
            let current_player = new_state.current_player_mut();
            let button_income = current_player.quilt_board.button_income;
            current_player.button_balance += button_income_triggers * button_income;
        }

        // 6. test if player moved over special patch (only a single one possible) and conditionally change the state
        let special_patch = new_state
            .time_board
            .get_special_patch_in_range(&walking_range);
        if let Some(special_patch_index) = special_patch {
            new_state
                .time_board
                .clear_special_patch(special_patch_index);

            // Test if special patch can even be placed
            if new_state.current_player().quilt_board.is_full() {
                // If not throw the special patch away and switch player
                new_state.switch_player();
                return new_state;
            }

            new_state.special_patch_placement_move = Some(special_patch_index);
            return new_state;
        }

        // test player position and optionally switch (always true if action.is_walking)
        if new_current_player_position > other_player_position {
            new_state.switch_player();
        }

        new_state
    }

    fn is_terminated(&self) -> bool {
        let player_1_position = self.player_1.position;
        let player_2_position = self.player_2.position;

        player_1_position >= TimeBoard::MAX_POSITION && player_2_position >= TimeBoard::MAX_POSITION
    }
}

impl Patchwork {
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
    fn get_take_and_place_a_patch_actions(&self) -> Vec<Action> {
        return self
            .patches
            .iter()
            .take(3)
            .enumerate()
            .filter(|patch| self.can_player_take_patch(self.current_player(), patch.1))
            .flat_map(|(index, patch)| {
                self.current_player()
                    .quilt_board
                    .get_valid_actions_for_patch(patch, index)
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
    fn can_player_take_patch(&self, player: &PlayerState, patch: &Patch) -> bool {
        // player can only place pieces that they can afford
        if patch.button_cost as i32 > player.button_balance {
            return false;
        }

        // player can only place pieces that fit on their board (fastpath)
        if QuiltBoard::TILES - player.quilt_board.tiles_filled() < patch.amount_tiles() {
            return false;
        }

        true
    }
}
