use crate::TreePolicyNode;

pub trait TreePolicy {
    fn select_node<Node, NodeIterator>(&self, children: NodeIterator) -> Node
    where
        Node: TreePolicyNode,
        NodeIterator: Iterator<Item = Node>;
}
