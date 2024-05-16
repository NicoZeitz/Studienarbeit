fn get_search_extension<const ZERO_WINDOW_SEARCH: bool>(
    &mut self,
    game: &Patchwork,
    num_extensions: usize,
    previous_special_tile_condition_reached: bool,
) -> usize {
    if ZERO_WINDOW_SEARCH || !Self::ENABLE_SEARCH_EXTENSIONS {
        return 0;
    }
    if num_extensions >= Self::MAX_SEARCH_EXTENSIONS {
        return 0;
    }

    let mut extension = 0;
    if matches!(game.turn_type, TurnType::SpecialPatchPlacement) {
        // Extend the depth of search for special patch placements
        extension += 1;
    }
    if !previous_special_tile_condition_reached && game.is_special_tile_condition_reached() {
        // Extend the depth of search if the 7x7 special tile was given
        extension += 1;
    }
    extension
}
