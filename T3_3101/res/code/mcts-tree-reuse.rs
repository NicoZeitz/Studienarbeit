pub fn from_root(last_tree: Option<Tree>, game: &Patchwork, policy: &Policy, evaluator: &Eval, abort_search_after: Option<std::time::Duration>) -> Self {
  let Some(mut last_tree) = last_tree else {
    return Self::new(game, policy, evaluator);
  };
  let mut queue = VecDeque::new();
  queue.push_back((0, last_tree.root));
  let start_time = std::time::Instant::now();
  while(!queue.is_empty()) {
    if abort_search_after.map_or(false, |time| start_time.elapsed() > time) {
      break;
    }
    let (depth, node_id) = queue.pop_front().unwrap();
    if depth >= 8 {
      break;
    }
    let node = last_tree.allocator.get_node(node_id);
    if node.state == *game { // found the correct node
      let root = last_tree.allocator
        .realloc_to_new_root(node_id);
      return SearchTree { root, policy, evaluator, depth: 0, reused: true, allocator: last_tree.allocator };
    }
    for child in &node.children {
      queue.push_back((depth + 1, *child));
    }
  }
  // The root node was not found in the tree.
  Self::new_with_allocator(last_tree.allocator, game, policy, evaluator)
}