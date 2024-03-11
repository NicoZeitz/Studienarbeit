use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq)]
pub struct TrainingArgs {
    /// The number of games to record before testing a new state of the network.
    pub number_of_games: NonZeroUsize,
    /// The number of games to play in parallel.
    pub parallelization: NonZeroUsize,
    /// The number of epochs to train for
    pub epochs: usize,
    /// The batch size used for training
    pub batch_size: usize,
    /// Number of evaluation games to play for determining if a new network is better than the old one
    pub evaluation_games: usize,
    /// The percentage to determine if a new network is better than the old one
    pub evaluation_percentage: f64,
    /// The learning rate for the optimizer
    pub learning_rate: f64,
    /// The amount of games to play against the handcrafted evaluator for comparison.
    pub comparison_games: usize,
}

impl Default for TrainingArgs {
    fn default() -> Self {
        let parallelization = std::thread::available_parallelism()
            .ok()
            .and_then(|n| NonZeroUsize::new(n.get() - 1 + 1))
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap());

        Self {
            number_of_games: NonZeroUsize::new(10_000).unwrap(),
            parallelization,
            epochs: 5,
            batch_size: 128,
            evaluation_games: 200,
            evaluation_percentage: 0.55,
            learning_rate: 0.05,
            comparison_games: 100,
        }
    }
}
