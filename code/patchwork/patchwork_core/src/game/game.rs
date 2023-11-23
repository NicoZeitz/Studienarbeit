use std::fmt;

/// Represents a game.
pub trait Game: Clone + Send + fmt::Debug {
    /// Different options for the game.
    type GameOptions;
    /// A type representing a player.
    type Player;
    /// A type representing an action.
    type Action: Clone + Send + Sync + fmt::Debug;
    /// A type representing a list of actions.
    type ActionList: std::iter::IntoIterator<Item = Self::Action>;

    /// Gets the initial state of the game.
    ///
    /// # Arguments
    ///
    /// * `seed` - The seed to use for the random number generator.
    /// * `player_1_name` - The name of the first player.
    /// * `player_2_name` - The name of the second player.
    ///
    /// # Returns
    ///
    /// The initial state of the game.
    fn get_initial_state(options: Self::GameOptions) -> Self;

    /// Gets the valid actions for the current player in the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// The valid actions for the current player in the given state.
    fn get_valid_actions(&self) -> Self::ActionList;

    /// Gets a random action for the current player in the given state.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// A random action for the current player in the given state.
    fn get_random_action(&self) -> Self::Action;

    /// Gets the next state of the game after the given action has been taken.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    /// * `action` - The action to take.
    ///
    /// # Returns
    ///
    /// The next state of the game.
    fn get_next_state(&self, action: &Self::Action) -> Self;

    /// Gets whether the given state is terminated.
    ///
    /// # Arguments
    ///
    /// * `state` - The state of the game.
    ///
    /// # Returns
    ///
    /// Whether the game associated with the given state is terminated.
    fn is_terminated(&self) -> bool;

    /// Gets the current player.
    ///
    /// # Returns
    ///
    /// The current player.
    fn get_current_player(&self) -> Self::Player;
}
