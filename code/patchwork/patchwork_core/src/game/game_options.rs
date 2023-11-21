/// Options for creating a new game of patchwork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameOptions {
    /// The seed to use for the random number generator.
    pub seed: u64,
}
