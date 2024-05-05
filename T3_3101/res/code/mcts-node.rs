pub struct Node {
    pub id: NodeId,
    pub state: Patchwork,
    pub parent: Option<NodeId>,
    pub action_taken: Option<ActionId>,
    pub children: Vec<NodeId>,
    pub expandable_actions: Vec<ActionId>,
    pub neutral_max_score: i32,
    pub neutral_min_score: i32,
    pub neutral_score_sum: i64,
    pub neutral_wins: i32,
    pub visit_count: usize,
}
