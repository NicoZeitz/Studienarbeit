use crate::{TreePolicy, TreePolicyNode};

/// UCT (Upper Confidence Bound 1 applied to trees) tree policy."""
pub struct UCTPolicy {
    /// The exploration parameter for the UCT policy."""
    exploration_constant: f64,
}

impl UCTPolicy {
    pub fn new(exploration_constant: f64) -> Self {
        Self {
            exploration_constant,
        }
    }
}

impl TreePolicy for UCTPolicy {
    fn select_node<Node, NodeIterator>(&self, children: NodeIterator) -> Node
    where
        Node: TreePolicyNode,
        NodeIterator: Iterator<Item = Node>,
    {
        let mut best_node: Option<Node> = None;
        let mut best_score = f64::NEG_INFINITY;

        for child in children {
            let child_visit_count = child.visit_count();
            let parent_visit_count = child.parent_visit_count();

            let exploitation_score = child.wins() as f64 / child_visit_count as f64;
            let exploration_score = self.exploration_constant
                * ((parent_visit_count as f64).ln() / child_visit_count as f64).sqrt();

            let score = exploitation_score + exploration_score;

            if score > best_score {
                best_node = Some(child);
                best_score = score;
            }
        }

        best_node.expect("[UCTPolicy][select_node] No children were given to select.")
    }
}
