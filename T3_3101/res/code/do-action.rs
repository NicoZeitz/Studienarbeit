pub fn do_action(&mut self, action: ActionId, force_player_switch: bool) -> Result<(), PatchworkError> {
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
            _ => Err(PatchworkError::InvalidActionError { ... }),
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
            let special_patch_index = self.time_board
                .get_special_patch_before_position(current_player_position)
                .unwrap();

            let current_player = self.current_player_mut();
            current_player.quilt_board.do_action(action);
            if current_player.quilt_board.is_special_tile_condition_reached()
                && !self.is_special_tile_condition_reached()
            {
                self.set_special_tile_condition(self.get_current_player());
            }

            self.switch_player();
            self.time_board.unset_special_patch(special_patch_index);
            self.turn_type = TurnType::Normal;
            Ok(())
        } else {
            Err(PatchworkError::InvalidActionError { ... })
        };
    }

    let now_other_player_position = self.other_player().position;
    let now_current_player_position = self.current_player().position;
    let time_cost;
    if action.is_walking() {
        // IF walking
        //   1. add +1 to current player button balance for every tile walked over

        let current_player = self.current_player_mut();
        time_cost = now_other_player_position - now_current_player_position + 1;

        let button_income = i32::from(now_other_player_position.min(TimeBoard::MAX_POSITION))
            - i32::from(now_current_player_position);
        if now_current_player_position + time_cost > TimeBoard::MAX_POSITION {
            current_player.button_balance += button_income;
        } else {
            current_player.button_balance += button_income + 1;
        }
    } else {
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

        current_player.button_balance -= i32::from(patch.button_cost);

        current_player.quilt_board.do_action(action);
        if current_player.quilt_board.is_special_tile_condition_reached()
            && !self.is_special_tile_condition_reached()
        {
            self.set_special_tile_condition(self.get_current_player());
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

    if next_current_player_position >= TimeBoard::MAX_POSITION && !self.is_goal_reached() {
        self.set_goal_reached(self.get_current_player());
    }
    self.time_board.move_player_position(
        self.status_flags & status_flags::BOTH_PLAYERS,
        now_current_player_position,
        next_current_player_position,
    );
    let walking_range = (now_current_player_position as usize + 1)
        ..=(next_current_player_position.min(TimeBoard::MAX_POSITION) as usize);

    // 5. test if player moved over button income trigger (only a single one possible) and add button income
    {
        let button_income_trigger = i32::from(self.time_board
            .is_button_income_trigger_in_range(walking_range.clone()));
        let current_player = self.current_player_mut();
        let button_income = i32::from(current_player.quilt_board.button_income);
        current_player.button_balance += button_income_trigger * button_income;
    }
    // 6. test if player moved over special patch (only a single one possible) and conditionally change the state
    if let Some(special_patch_index) = self.time_board.get_single_special_patch_in_range(walking_range) {
        // Test if special patch can even be placed
        if self.current_player().quilt_board.is_full() {
            // If not throw the special patch away and switch player
            self.time_board.unset_special_patch(special_patch_index);
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