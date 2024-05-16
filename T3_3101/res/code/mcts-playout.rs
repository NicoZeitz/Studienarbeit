pub fn playout(&mut self) -> PlayerResult<()> {
    let mut node_id = self.root;
    // 1. Selection
    while self.should_be_selected(node_id) {
        node_id = self.node_select(node_id);
    }
    let value = if self.is_terminal(node_id) {
        // 3. Leaf/Terminal Node -> Direct Evaluation
        let node = self.allocator.get_node(node_id);
        self.evaluator.evaluate_terminal_node(&node.state)
    } else {
        // 2. Expansion
        node_id = self.node_expand(node_id)?;
        // 3. Simulation
        self.node_simulate(node_id)
    };
    // 4. Backpropagation
    self.node_backpropagate(node_id, value);
    Ok(())
}
