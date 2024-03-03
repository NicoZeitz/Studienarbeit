#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct TrainingArgs {
    // Hyperparameters

    /// The exploration constant to use for the PUCT algorithm.
    pub c: f32,
    /// The learning rate to use for training the neural network.
    pub learning_rate: f64,
    /// The dirichlet epsilon that gives the proportion between noise and probabilities in the root node
    pub dirichlet_epsilon: f32,
    /// The dirichlet alpha value to use for the noise distribution on the root node. Noise ~ Dir(α)
    pub dirichlet_alpha: f32,
    /// The temperature to use for action selection during training.
    pub temperature: f32,
    /// The ply after which the temperature is set to a infinitesimal value (0.00001)
    pub temperature_end: usize,
    /// The L² regularization factor to use for training the neural network.
    pub regularization: f64,

    // Train loop

    /// How often to repeat the self-play & train training loop
    pub number_of_training_iterations: usize,

    // Self-Play

    /// The number of MCTS iterations to run before choosing an action.
    pub number_of_mcts_iterations: usize,
    /// The number of iterations to self play games for for creating the training dataset
    pub number_of_self_play_iterations: usize,
    /// The number of parallel games to play during self playing
    pub number_of_parallel_games: usize,

    // Training model

    /// The number of epochs to learn for.
    pub number_of_epochs: usize,
    /// The batch size to use for training
    pub batch_size: usize,
}

impl Default for TrainingArgs {
    fn default() -> Self {
        Self {
            c: 2f32.sqrt(),
            learning_rate: 0.2, // 0.02, 0.002 0.0002 drop after some time
            dirichlet_epsilon: 0.25,
            dirichlet_alpha: 0.2,
            temperature: 1.25,
            temperature_end: 35,
            regularization: 5e-5, // 1e-4,

            number_of_training_iterations: 8,

            number_of_mcts_iterations: 800,
            number_of_self_play_iterations: 500,
            number_of_parallel_games: 100,

            number_of_epochs: 4,
            batch_size: 128,
        }
    }
}
