/// The type of termination of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminationType {
    /// Player 1 won the game.
    Player1Won,
    /// Player 2 won the game.
    Player2Won,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Termination {
    /// The type of termination of the game.
    pub termination: TerminationType,
    /// The score of player 1.
    pub player_1_score: i32,
    /// The score of player 2.
    pub player_2_score: i32,
}
impl Termination {
    /// Returns the score of the game. Positive if player 1 won, negative if player 2 won, 0 if draw. The score is calculated by taking the difference between the score of player 1 and player 2.
    ///
    /// # Returns
    ///
    /// The score of the game.
    #[inline]
    pub fn score(&self) -> i32 {
        self.player_1_score - self.player_2_score
    }
}
