use patchwork_core::{ScoredTreePolicy, TreePolicyNode};

/// The First Play Urgency (FPU) strategy to use for the MCTS. The FPU is used to give a exploitation value to unvisited
/// nodes. The original `AlphaZero` paper used a value of -1 for the FPU (see [AlphaZero paper, and Lc0 v0.19.1](https://lczero.org/blog/2018/12/alphazero-paper-and-lc0-v0191/)).
/// This means that unvisited nodes are considered as losses. Leela Chess Zero (Lc0) initializes the FPU with the parent
/// value reduced by a certain amount (0.44) (see [Engine parameters](https://lczero.org/play/flags/)).
#[derive(Debug, Clone)]
pub enum FPUStrategy {
    /// Directly initialize the FPU with a constant value.
    Absolute(f64),
    /// Initialize the FPU with the parent value reduced by a certain amount.
    Reduction(f64),
}

impl FPUStrategy {
    /// Returns the FPU value for the given parent value.
    ///
    /// # Arguments
    ///
    /// * `parent_value` - The value of the parent node.
    ///
    /// # Returns
    ///
    /// The FPU value for the given parent value.
    #[must_use]
    pub fn get_fpu(&self, parent_value: f64) -> f64 {
        match self {
            Self::Absolute(fpu) => *fpu,
            Self::Reduction(reduction) => parent_value - reduction,
        }
    }
}

impl Default for FPUStrategy {
    fn default() -> Self {
        Self::Reduction(0.44)
    }
}

/// An implementation of the PUCT policy used in `AlphaZero` and Leela Chess Zero (Lc0).
///
/// # Formula
///
/// ```math
/// 𝒬 + 𝒸 · 𝓅 · √𝒩 / (1 + 𝑛)
///
/// with 𝒬 = fpu if 𝑛 = 0 else 𝓌 / 𝑛
///      𝓌 = The wins of the child node from the perspective of the parent
///      𝑛 = The amount of visits of the child node
///      𝒩 = The amount of visits of the parent node
///      𝒸 = exploration constant (usually √2)
///      𝓅 = prior believe about the child node
/// ```
///
/// # See also
///
/// - [AlphaZero paper, and Lc0 v0.19.1](https://lczero.org/blog/2018/12/alphazero-paper-and-lc0-v0191/)
/// - [Engine parameters](https://lczero.org/play/flags/)
/// - [Wikipedia article on UCT](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search#Exploration_and_exploitation)
/// - [Mastering Chess and Shogi by Self-Play with a General Reinforcement Learning Algorithm, S.8](https://arxiv.org/abs/1712.01815)
/// - [A Simple Alpha(Go) Zero Tutorial](https://web.stanford.edu/~surag/posts/alphazero.html)
/// - [Multi-armed bandits with episode context](https://link.springer.com/article/10.1007/s10472-011-9258-6)
#[derive(Debug, Clone)]
pub struct PUCTPolicy {
    /// The exploration parameter for the UCT policy.
    exploration_constant: f64,
    /// The First Play Urgency (FPU) strategy to use for the MCTS.
    fpu_strategy: FPUStrategy,
}

impl PUCTPolicy {
    /// Creates a new [`AlphaZeroUCTPolicy`] with the given exploration constant.
    ///
    /// # Arguments
    ///
    /// * `exploration_constant` - The exploration constant for the UCT policy.
    ///
    /// # Returns
    ///
    /// The new [`AlphaZeroUCTPolicy`].
    #[must_use]
    pub const fn new(exploration_constant: f64, fpu_strategy: FPUStrategy) -> Self {
        Self {
            exploration_constant,
            fpu_strategy,
        }
    }
}

impl Default for PUCTPolicy {
    fn default() -> Self {
        Self::new(2f64.sqrt(), FPUStrategy::default())
    }
}

impl PUCTPolicy {
    // #[inline]
    fn get_exploitation<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let child_visit_count = child.visit_count() as f64;
        let parent_visit_count = parent.visit_count() as f64;
        let parent_player = parent.current_player();

        if child.visit_count() == 0 {
            // use FPU
            return match self.fpu_strategy {
                FPUStrategy::Absolute(fpu) => fpu,
                FPUStrategy::Reduction(reduction) => {
                    if parent.visit_count() == 0 {
                        // use 0 as default value for a 'draw' if the root node has not been visited jet like Leela Chess Zero (Lc0)
                        // [EdgeAndNode::GetQ](https://github.com/LeelaChessZero/lc0/blob/master/src/mcts/node.h#L375)
                        // [Engine Parameters](https://lczero.org/play/flags)
                        // 0.0
                        f64::from(child.wins_for(parent_player)) // push up virtual loss
                    } else {
                        f64::from(parent.wins_for(parent_player)) / parent_visit_count - reduction
                    }
                }
            };
        }

        let res = f64::from(child.wins_for(parent_player)) / child_visit_count;

        #[cfg(debug_assertions)]
        if res.is_infinite() {
            println!(
                "Infinite exploitation: wins: {}, visit count: {} or {} = {} or {}",
                f64::from(child.wins_for(parent_player)),
                child.visit_count() as f64,
                child_visit_count,
                f64::from(child.wins_for(parent_player)) / child.visit_count() as f64,
                res
            );
            panic!("Infinite exploitation")
        }

        res
    }

    #[inline]

    fn get_exploration<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let child_visit_count = child.visit_count() as f64;
        let parent_visit_count = parent.visit_count() as f64;
        self.exploration_constant * child.prior_value() * (parent_visit_count.sqrt() / (1.0 + child_visit_count))
    }
}

impl ScoredTreePolicy for PUCTPolicy {
    fn get_score<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64 {
        let exploitation = self.get_exploitation(parent, child);
        let exploration = self.get_exploration(parent, child);

        exploitation + exploration
    }
}
