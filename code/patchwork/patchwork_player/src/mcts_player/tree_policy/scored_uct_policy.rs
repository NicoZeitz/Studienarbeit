use crate::{TreePolicy, TreePolicyNode};

/// UCT (Upper Confidence Bound 1 applied to trees) tree policy taking into account the final score of the game."""
pub struct ScoredUCTPolicy {
    /// The exploration parameter for the UCT policy."""
    exploration_constant: f64,
}

impl ScoredUCTPolicy {
    pub fn new(exploration_constant: f64) -> Self {
        Self {
            exploration_constant,
        }
    }
}

impl TreePolicy for ScoredUCTPolicy {
    fn select_node<Node, NodeIterator>(&self, parent: Node, children: NodeIterator) -> Node
    where
        Node: TreePolicyNode,
        NodeIterator: Iterator<Item = Node>,
    {
        let mut best_node: Option<Node> = None;
        let mut best_score = f64::NEG_INFINITY;

        // println!("==============================================================");

        for child in children {
            let score_range = (parent.max_score() - parent.min_score()).abs();
            let exploitation_score = child.score_sum() / child.visit_count() as f64;
            let exploration_score = self.exploration_constant
                * score_range
                * ((parent.visit_count() as f64).ln() / child.visit_count() as f64).sqrt();

            let score = exploitation_score + exploration_score;

            // // TODO:
            // println!(
            //     "score: {}, exploration: {}, exploitation: {}, wins: {}, max_score: {}, min_score: {}, score_sum: {}, visit_count: {}, parent_visit: {}",
            //     score,
            //     exploration_score,
            //     exploitation_score,
            //     child.wins(),
            //     child.max_score(),
            //     child.min_score(),
            //     child.score_sum(),
            //     child.visit_count(),
            //     parent.visit_count()
            // );

            if score > best_score {
                best_node = Some(child);
                best_score = score;
            }
        }

        best_node.expect("[ScoredUCTPolicy][select_node] No children were given to select.")
    }
}
