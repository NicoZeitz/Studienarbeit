pub fn get_initial_state(options: Option<GameOptions>) -> Self {
    // 1. Each player takes a quilt board, a time token and
    //    5 buttons (as currency). Keep the remaining buttons
    //    on the table close at hand.
    let player_1 = PlayerState::default();
    let player_2 = PlayerState::default();
    // 2. Place the central time board in the middle of the
    //    table.
    // 3. Place your time tokens on the starting space of the
    //    time board. The player who last used a needle
    //    begins
    let time_board = TimeBoard::default();
    let status_flags = Self::get_player_1_flag();
    // 4. Place the (regular) patches in a circle or oval
    //    around the time board.
    // 5. Locate the smallest patch, i.e. the patch of size
    //    1x2, and place the neutral token between this patch
    //    and the next patch in clockwise order.
    let patches = PatchManager::generate_patches(options.map(|o| o.seed));
    // 6. Lay out the special tile
    // 7. Place the special patches on the marked spaces of
    //    the time board
    // 8. Now you are ready to go!
    Self {
        patches,
        time_board,
        player_1,
        player_2,
        status_flags,
        turn_type: TurnType::Normal,
    }
}
