use crate::TreePolicyNode;

pub trait TreePolicy: Sync {
    fn select_node<Node, NodeIterator>(&self, parent: Node, children: NodeIterator) -> Node
    where
        Node: TreePolicyNode,
        NodeIterator: Iterator<Item = Node>;
}
