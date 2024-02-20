use patchwork_core::{ScoredTreePolicy, TreePolicyNode};

/// An implementation of the UCT (Upper Confidence Bound 1 applied to trees)
/// tree policy but taking into account the final score of the game.
///
/// The final score is taken into account by using the average score of the
/// child node from the perspective of the parent node and scaling the
/// exploration score by the difference between the maximum and minimum scores
/// of the parent node.
///
/// # Formula
///
/// ```math
/// ∑𝓈ᵢ / 𝑛 + 𝒸 · |maxᵢ 𝓈ᵢ - minᵢ 𝓈ᵢ| · √(㏑ 𝒩 / 𝑛)
///
/// with 𝓈ᵢ = The score of the 𝒾's visit
///      𝑛 = The amount of visits of the child node
///      𝒩 = The amount of visits of the parent node
///      𝒸 = exploration constant (usually √2)
/// ```
///
/// # See also
///
/// - [Wikipedia article on UCT](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation)
/// - [MCTS UCT with a scoring system](https://stackoverflow.com/questions/36664993/mcts-uct-with-a-scoring-system)
pub struct ScoredUCTPolicy {
    /// The exploration parameter for the UCT policy.
    exploration_constant: f64,
}

impl ScoredUCTPolicy {
    /// Creates a new [`ScoredUCTPolicy`] with the given exploration constant.
    ///
    /// # Arguments
    ///
    /// * `exploration_constant` - The exploration constant for the UCT policy.
    ///
    /// # Returns
    ///
    /// The new [`ScoredUCTPolicy`].
    pub fn new(exploration_constant: f64) -> Self {
        Self { exploration_constant }
    }
}

impl Default for ScoredUCTPolicy {
    fn default() -> Self {
        Self::new(2f64.sqrt())
    }
}

impl ScoredTreePolicy for ScoredUCTPolicy {
    fn get_score<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let child_visit_count = child.visit_count() as f64;
        let parent_visit_count = parent.visit_count() as f64;
        let parent_player = parent.current_player();

        let exploitation_score = child.average_score_for(parent_player);

        let exploration = (parent_visit_count.ln() / child_visit_count).sqrt();
        let exploration_score = self.exploration_constant * parent.score_range() * exploration;

        exploitation_score + exploration_score
    }
}
