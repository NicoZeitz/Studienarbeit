use crate::traits::TreePolicyNode;

/// A tree policy for selecting nodes in a tree.
///
/// This trait is used to select nodes in a tree. It is used in the MCTS
/// algorithm to select nodes when traversing the tree.
pub trait TreePolicy {
    /// Selects a node from the given children of the parent node.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent node to select the best child from.
    /// * `children` - The children of the parent node.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    ///
    /// # Remarks
    ///
    /// This method should panic if no children were given to select.
    fn select_node<'a, Node: TreePolicyNode>(
        &self,
        parent: &Node,
        children: impl Iterator<Item = &'a Node>,
    ) -> &'a Node;
}

/// A convenience trait for tree policies that score nodes and choose the node
/// with the highest score.
pub trait ScoredTreePolicy {
    /// Gets the score of the given child node from the perspective of the given
    /// parent node.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent node from where to score the child node.
    /// * `child` - The child node to get the score for.
    ///
    /// # Returns
    ///
    /// The score of the child node from the perspective of the parent node.
    fn get_score<Player: Copy>(
        &self,
        parent: &impl TreePolicyNode<Player = Player>,
        child: &impl TreePolicyNode<Player = Player>,
    ) -> f64;
}

impl<T> TreePolicy for T
where
    T: ScoredTreePolicy,
{
    /// Selects the best child node of the given parent node by scoring each
    /// node from the perspective of the parent and selecting the node with the
    /// best score
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent node to select the best child from.
    /// * `children` - The children of the parent node.
    ///
    /// # Returns
    ///
    /// The best child node of the given parent node.
    ///
    /// # Panics
    ///
    /// Panics if no children were given to select.
    fn select_node<'a, Node: TreePolicyNode>(
        &self,
        parent: &Node,
        children: impl Iterator<Item = &'a Node>,
    ) -> &'a Node {
        let mut best_node = None;
        let mut best_score = f64::NEG_INFINITY;

        for child in children {
            let score = self.get_score(parent, child);
            if score > best_score {
                best_node = Some(child);
                best_score = score;
            }
        }

        best_node.expect("[ScoredTreePolicy::select_node] No children were given to select.")
    }
}
