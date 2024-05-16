pub fn get_termination_result(&self) -> Termination {
    let player_1_score = self.get_score(status_flags::PLAYER_1);
    let player_2_score = self.get_score(status_flags::PLAYER_2);

    let termination = match player_1_score.cmp(&player_2_score) {
        Ordering::Less => TerminationType::Player2Won,
        Ordering::Greater => TerminationType::Player1Won,
        Ordering::Equal => {
            if (self.status_flags & status_flags::PLAYER_1_FIRST_AT_END) > 0 {
                TerminationType::Player1Won
            } else if (self.status_flags & status_flags::PLAYER_2_FIRST_AT_END) > 0 {
                TerminationType::Player2Won
            } else {
                panic!("...")
            }
        }
    };

    Termination {
        termination,
        player_1_score,
        player_2_score,
    }
}
