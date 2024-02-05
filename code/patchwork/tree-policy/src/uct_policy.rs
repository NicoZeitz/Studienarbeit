use patchwork_core::{ScoredTreePolicy, TreePolicyNode};

/// An implementation of the UCT (Upper Confidence Bound 1 applied to trees)
/// tree policy.
///
/// # Formula
///
/// ```math
/// 𝓌 / 𝑛  + 𝒸 · √(㏑ 𝒩 / 𝑛)
///
/// with 𝓌 = The wins of the child node from the perspective of the parent
///      𝑛 = The amount of visits of the child node
///      𝒩 = The amount of visits of the parent node
///      𝒸 = exploration constant (usually √2)
/// ```
///
/// # See also
///
/// - [Wikipedia article on UCT](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation)
pub struct UCTPolicy {
    /// The exploration parameter for the UCT policy.
    exploration_constant: f64,
}

impl UCTPolicy {
    /// Creates a new [`UCTPolicy`] with the given exploration constant.
    ///
    /// # Arguments
    ///
    /// * `exploration_constant` - The exploration constant for the UCT policy.
    ///
    /// # Returns
    ///
    /// The new [`UCTPolicy`].
    pub fn new(exploration_constant: f64) -> Self {
        Self { exploration_constant }
    }
}

impl Default for UCTPolicy {
    fn default() -> Self {
        Self::new(2f64.sqrt())
    }
}

impl ScoredTreePolicy for UCTPolicy {
    fn get_score<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let child_visit_count = child.visit_count() as f64;
        let parent_visit_count = parent.visit_count() as f64;
        let parent_player = parent.current_player();

        let exploitation_wins = child.wins_for(parent_player) as f64 / child_visit_count;

        let exploration = (parent_visit_count.ln() / child_visit_count).sqrt();
        let exploration_wins = self.exploration_constant * exploration;

        exploitation_wins + exploration_wins
    }
}
