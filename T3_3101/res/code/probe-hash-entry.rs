pub fn probe_hash_entry(&self, game: &Patchwork, alpha: i32, beta: i32, depth: usize) -> Option<(ActionId, i32)> {
  let hash = self.zobrist_hash.hash(game);
  let index = (hash % self.entries_len() as u64) as usize;
  let data = self.index_entries(index).data;
  let test_key = hash ^ data;
  if self.index_entries(index).key != test_key {
    return None;
  }

  let (table_depth, table_eval, table_eval_type, table_action) = Entry::unpack_data(data);
  if table_depth < depth {
    return None;
  }

  match table_eval_type {
    EvaluationType::Exact => {
      // PV-Node but alpha-beta range could have changed
      let table_eval = table_eval.clamp(alpha, beta);
      Some((table_action, table_eval))
    }
    EvaluationType::UpperBound => if table_eval <= alpha {
        Some((table_action, alpha))
    } else { None }
    EvaluationType::LowerBound => if table_eval >= beta {
        Some((table_action, beta))
    } else { None }
  }
}
