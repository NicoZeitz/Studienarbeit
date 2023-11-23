// A node a tree policy gets to select from.
pub trait TreePolicyNode {
    /// The maximum score of all the nodes in the subtree rooted at this node.
    fn max_score(&self) -> f64;
    // The minimum score of all the nodes in the subtree rooted at this node.
    fn min_score(&self) -> f64;
    // The sum of the scores of all the nodes in the subtree rooted at this node.
    fn score_sum(&self) -> f64;
    // The number of wins this node has from the perspective of the player whose turn it is to move.
    fn wins(&self) -> i32;
    // The number of times this node has been visited.
    fn visit_count(&self) -> i32;
    // The number of times the parent of this node has been visited.
    fn parent_visit_count(&self) -> i32;
}
