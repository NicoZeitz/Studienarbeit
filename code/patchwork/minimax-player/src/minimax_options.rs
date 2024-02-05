/// The options for [`MinimaxPlayer`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MinimaxOptions {
    /// The depth to search to.
    pub depth: usize,
    /// The amount of actions to consider per piece.
    /// This is used to reduce the branching factor.
    pub amount_actions_per_piece: usize,
}

impl MinimaxOptions {
    /// Creates a new [`MinimaxOptions`].
    pub fn new(depth: usize, amount_actions_per_piece: usize) -> Self {
        Self {
            depth,
            amount_actions_per_piece,
        }
    }
}

impl Default for MinimaxOptions {
    fn default() -> Self {
        Self {
            depth: 8,
            amount_actions_per_piece: 3,
        }
    }
}
