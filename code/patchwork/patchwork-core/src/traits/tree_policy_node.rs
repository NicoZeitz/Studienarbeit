// A node a tree policy gets to select from.
pub trait TreePolicyNode {
    /// The type of the player in the game.
    type Player: Copy;
    /// The number of times this node has been visited.
    ///
    /// # Returns
    ///
    /// The number of times this node has been visited.
    fn visit_count(&self) -> i32;
    /// The player whose turn it is at the current node.
    ///
    /// # Returns
    ///
    /// The player whose turn it is at the current node.
    fn current_player(&self) -> Self::Player;
    /// The number of wins this node has had for the given player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to get the number of wins for.
    ///
    /// # Returns
    ///
    /// The number of wins for the given player.
    fn wins_for(&self, player: Self::Player) -> i32;
    /// Gets the maximum score of all games played from this node from the perspective of the given
    /// player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to get the maximum score for.
    ///
    /// # Returns
    ///
    /// The maximum score of all games played from this node from the perspective of the given
    fn maximum_score_for(&self, player: Self::Player) -> i32;
    /// Gets the minimum score of all games played from this node from the perspective of the given
    /// player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to get the minimum score for.
    ///
    /// # Returns
    ///
    /// The minimum score of all games played from this node from the perspective of the given
    fn minimum_score_for(&self, player: Self::Player) -> i32;
    /// Gets the difference between the maximum and minimum scores of the node.
    /// This is the same as the difference between the biggest win and biggest
    /// loss the node has had from the perspective of either player.
    ///
    /// # Returns
    ///
    /// The difference between the maximum and minimum scores of the node.
    fn score_range(&self) -> i32 {
        let some_player = self.current_player();
        (self.maximum_score_for(some_player) - self.minimum_score_for(some_player)).abs()
    }
    /// Gets the sum of the scores of all games played from this node from the perspective of the
    /// given player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to get the sum of the scores for.
    ///
    /// # Returns
    ///
    /// The sum of the scores of all games played from this node from the perspective of the given
    /// player.
    fn score_sum_for(&self, player: Self::Player) -> i64;
    /// Gets the average score of all games played from this node from the perspective of the given
    /// player.
    ///
    /// # Arguments
    ///
    /// * `player` - The player to get the sum of the scores for.
    ///
    /// # Returns
    ///
    /// The average score of all games played from this node from the perspective of the given
    /// player.
    fn average_score_for(&self, player: Self::Player) -> f64 {
        if self.visit_count() == 0 {
            return 0.0;
        }
        self.score_sum_for(player) as f64 / self.visit_count() as f64
    }
    /// Returns a prior believe of the value of the node if available.
    /// Otherwise returns 0.
    ///
    /// # Returns
    ///
    /// The prior value of the node.
    fn prior_value(&self) -> f64 {
        0.0
    }
}
