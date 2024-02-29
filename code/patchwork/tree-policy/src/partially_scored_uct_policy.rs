use patchwork_core::{ScoredTreePolicy, TreePolicyNode};

/// An implementation of the UCT (Upper Confidence Bound 1 applied to trees)
/// tree policy but partially taking into account the final score of the game.
///
/// The final score is taken into account by using the average score of the
/// child node from the perspective of the parent node and scaling the
/// exploration score by the difference between the maximum and minimum scores
/// of the parent node. The normal UCT score is then combined with the score
/// using linear interpolation with the given portion for the score and the
/// rest for the wins.
///
/// The score portion parameter is a value between 0 and 100, where 0 means
/// that only the wins are taken into account and 100 means that only the
/// score is taken into account.
///
/// The default portion is 10% for the score and the rest for the wins.
///
/// # Formula
///
/// ```math
///        𝓅  · (∑𝓈ᵢ / 𝑛 + 𝒸 · √(㏑ 𝒩 / 𝑛) · |maxᵢ 𝓈ᵢ - minᵢ 𝓈ᵢ|)
/// + (1 - 𝓅) · (𝓌 / 𝑛  + 𝒸 · √(㏑ 𝒩 / 𝑛))
///
/// with 𝓅 = The portion that scores should be taken into account
///      𝓈ᵢ = The score of the 𝒾's visit
///      𝓌 = The wins of the child node from the perspective of the parent
///      𝑛 = The amount of visits of the child node
///      𝒩 = The amount of visits of the parent node
///      𝒸 = exploration constant (usually √2)
/// ```
///
/// # See also
///
/// - [Wikipedia article on UCT](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation)
/// - [MCTS UCT with a scoring system](https://stackoverflow.com/questions/36664993/mcts-uct-with-a-scoring-system)
pub struct PartiallyScoredUCTPolicy<const SCORE_PORTION: u8 = 10> {
    /// The exploration parameter for the UCT policy.
    exploration_constant: f64,
}

impl<const SCORE_PORTION: u8> PartiallyScoredUCTPolicy<SCORE_PORTION> {
    /// The const parameter [`SCORE_PORTION`] as a percentage.
    const PORTION: f64 = SCORE_PORTION as f64 / 100f64;

    /// Creates a new [`PartiallyScoredUCTPolicy`] with the given exploration constant.
    ///
    /// # Arguments
    ///
    /// * `exploration_constant` - The exploration constant for the UCT policy.
    ///
    /// # Returns
    ///
    /// The new [`PartiallyScoredUCTPolicy`].
    #[must_use]
    pub const fn new(exploration_constant: f64) -> Self {
        Self { exploration_constant }
    }
}

impl<const SCORE_PORTION: u8> Default for PartiallyScoredUCTPolicy<SCORE_PORTION> {
    fn default() -> Self {
        Self::new(2f64.sqrt())
    }
}

impl<const SCORE_PORTION: u8> ScoredTreePolicy for PartiallyScoredUCTPolicy<SCORE_PORTION> {
    fn get_score<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let child_visit_count = child.visit_count() as f64;
        let parent_visit_count = parent.visit_count() as f64;
        let parent_player = parent.current_player();

        let exploitation_wins = f64::from(child.wins_for(parent_player)) / child_visit_count;
        let exploitation_score = child.average_score_for(parent_player);

        let exploration = (parent_visit_count.ln() / child_visit_count).sqrt();
        let exploration_wins = self.exploration_constant * exploration;
        let exploration_score = self.exploration_constant * parent.score_range() * exploration;

        Self::PORTION.mul_add(
            exploitation_score + exploration_score,
            (1f64 - Self::PORTION) * (exploitation_wins + exploration_wins),
        )
    }
}
