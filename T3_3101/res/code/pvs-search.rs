fn principal_variation_search<const ZERO_WINDOW_SEARCH: bool>(&mut self, game: &mut Patchwork, ply_from_root: usize, depth: usize, alpha: i32, beta: i32, num_extensions: usize) -> PlayerResult<i32> {
  // search canceled, return as fast as possible
  if self.search_canceled.load(Ordering::Relaxed) {
    return Ok(0);
  }
  // skip phantom moves
  if matches!(game.turn_type,
    TurnType::NormalPhantom | TurnType::SpecialPhantom) {
    let evaluation = self.phantom_skip::<ZERO_WINDOW_SEARCH>(
      game,
      ply_from_root, depth,
      alpha, beta,
      num_extensions,
    )?;
    return Ok(evaluation);
  }
  // Transposition table lookup
  if Self::ENABLE_TRANSPOSITION_TABLE {
    if let Some((table_action, table_evaluation)) = self
      .transposition_table
      .probe_hash_entry(game, alpha, beta, depth)
    {
      // cannot happen in Zero window search since alpha = beta - 1
      if ply_from_root == 0 && !ZERO_WINDOW_SEARCH {
        self.best_action = Some(table_action);
        self.best_evaluation = Some(table_evaluation);
      }
      return Ok(table_evaluation);
    }
  }
  if depth == 0 || game.is_terminated() {
    return Ok(self.evaluation(game));
  }
  let mut actions = game.get_valid_actions();
  let mut scores = vec![0.0; actions.len()];
  let mut action_list = self.get_action_list(game, ply_from_root, &mut actions, &mut scores);
  let mut is_pv_node = true;
  let mut best_action = ActionId::null();
  let mut alpha = alpha;
  let mut evaluation_bound = EvaluationType::UpperBound;
  let mut lmp_flags = self
    .get_late_move_pruning_flags(&action_list);

  for i in 0..action_list.len() {
    let Some(action) = self.get_next_action(ply_from_root, i, &mut action_list, &mut lmp_flags)
    else {
      break;
    };

    // Save previous state characteristics that are needed later
    let previous_special_tile_condition_reached = game.is_special_tile_condition_reached();

    game.do_action(action, true)?;

    let (next_depth, extension) = self.get_next_depth::<ZERO_WINDOW_SEARCH>(
      game,
      depth,
      !is_pv_node,
      num_extensions,
      previous_special_tile_condition_reached,
    );
    let mut evaluation = 0;
    if is_pv_node {
      // Full window search for pv node in non-zero window search
      evaluation = -self
        .principal_variation_search::<ZERO_WINDOW_SEARCH>(
          game,
          ply_from_root + 1, next_depth,
          -beta, -alpha,
          num_extensions + extension,
      )?;
    } else {
      let mut needs_full_search = true;
      // Apply Late Move Reductions (LMR)
      if self.should_late_move_reduce(game, i, ply_from_root, extension) {
        // Search this move with reduced depth
        let next_depth = self.get_late_move_reduced_depth(next_depth);
        evaluation = -self.zero_window_search(
          game,
          ply_from_root + 1, next_depth,
          -alpha,
          num_extensions + extension,
        )?;
        needs_full_search = evaluation > alpha;
      }
      if needs_full_search {
        // Null Window at full search depth
        evaluation = -self.zero_window_search(
          game,
          ply_from_root + 1, next_depth, 
          -alpha,
          num_extensions + extension,
        )?;
        if evaluation > alpha && evaluation < beta && !ZERO_WINDOW_SEARCH {
          // Cannot happen in Zero window search since alpha = beta - 1 < evaluation < beta

          // Zero-Window-Search failed, re-search with full window
          let (next_depth, extension) = self.get_next_depth::<ZERO_WINDOW_SEARCH>(
            game,
            depth,
            false,
            num_extensions,
            previous_special_tile_condition_reached,
          );
          evaluation = -self
            .principal_variation_search::<ZERO_WINDOW_SEARCH>(
              game,
              ply_from_root + 1, next_depth,
              -beta, -alpha,
              num_extensions + extension,
          )?;
        } 
      }
    }
    game.undo_action(action, true)?;

    if self.search_canceled.load(Ordering::Relaxed) {
      return Ok(0);
    }
    if evaluation >= beta {
      self.store_transposition_table(game, depth, beta, EvaluationType::LowerBound, action);

      return Ok(if Self::SOFT_FAILING_STRATEGY {
        evaluation // Fail-soft beta-cutoff
      } else {
        beta       // Fail-hard beta-cutoff
      });
    }

    // Cannot happen in Zero window search since alpha = beta - 1
    if evaluation > alpha && !ZERO_WINDOW_SEARCH {
      evaluation_bound = EvaluationType::Exact;
      alpha = evaluation; // alpha acts like max in minimax
      best_action = action;
    }
    is_pv_node = false;
  }

  // In case of a UpperBound we store a null action, as the 
  // true best action is unknown
  self.store_transposition_table(game, depth, alpha, evaluation_bound, best_action);
  // Cannot happen in Zero window search since ply is 0
  if ply_from_root == 0 && !ZERO_WINDOW_SEARCH {
    self.best_action = Some(best_action);
    self.best_evaluation = Some(alpha);
  }
  Ok(alpha)
}
