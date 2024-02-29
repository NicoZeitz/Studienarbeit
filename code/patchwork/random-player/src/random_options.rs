/// The options for [`RandomPlayer`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RandomOptions {
    /// The seed for the random number generator.
    pub seed: u64,
}

impl RandomOptions {
    /// Creates a new [`RandomOptions`].
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl Default for RandomOptions {
    fn default() -> Self {
        Self { seed: rand::random() }
    }
}
