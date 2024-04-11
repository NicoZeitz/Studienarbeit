pub struct Patchwork {
    pub patches: Vec<&'static Patch>,
    pub time_board: TimeBoard,
    pub player_1: PlayerState,
    pub player_2: PlayerState,
    pub turn_type: TurnType,
    pub status_flags: u8,
}