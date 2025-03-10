/// The statistics of a search.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchStatistics<const ACTIVE: bool> {
    /// The number of nodes searched in the previous iteration.
    pub nodes_searched_previous_iteration: usize,
    /// The number of nodes searched.
    pub nodes_searched: usize,
    /// The number of leaf nodes searched.
    pub leaf_nodes_searched: usize,
    /// The time when the search started.
    pub start_time: std::time::Instant,
    /// The number of times the search failed high.
    pub fail_high: usize,
    /// The number of times the search failed high on the first node.
    pub fail_high_first: usize,
    /// The number of times the aspiration window failed low.
    pub aspiration_window_fail_low: usize,
    /// The number of times the aspiration window failed high.
    pub aspiration_window_fail_high: usize,
    /// The number of times the zero window search was performed.
    pub zero_window_search: usize,
    /// The number of times the zero window search failed.
    pub zero_window_search_fail: usize,
    /// The number of times a special patch extension was made.
    pub special_patch_extensions: usize,
    /// The number of times a special tile (7x7) extension was made.
    pub special_tile_extensions: usize,
    /// The number of times late move reductions were performed.
    pub late_move_reductions: usize,
    /// The number of times late move reductions were performed and failed.
    pub late_move_reduction_fails: usize,
    /// The number of times late move pruning was performed.
    pub late_move_pruning: usize,
}

impl<const ACTIVE: bool> Default for SearchStatistics<ACTIVE> {
    fn default() -> Self {
        Self {
            nodes_searched_previous_iteration: 0,
            nodes_searched: 0,
            leaf_nodes_searched: 0,
            start_time: std::time::Instant::now(),
            fail_high: 0,
            fail_high_first: 0,
            aspiration_window_fail_low: 0,
            aspiration_window_fail_high: 0,
            zero_window_search: 0,
            zero_window_search_fail: 0,
            special_patch_extensions: 0,
            special_tile_extensions: 0,
            late_move_reductions: 0,
            late_move_reduction_fails: 0,
            late_move_pruning: 0,
        }
    }
}

impl<const ACTIVE: bool> SearchStatistics<ACTIVE> {
    #[inline]
    pub fn reset(&mut self) {
        if !ACTIVE {
            return;
        }
        self.nodes_searched_previous_iteration = 0;
        self.nodes_searched = 0;
        self.leaf_nodes_searched = 0;
        self.start_time = std::time::Instant::now();
        self.fail_high = 0;
        self.fail_high_first = 0;
        self.aspiration_window_fail_low = 0;
        self.aspiration_window_fail_high = 0;
        self.zero_window_search = 0;
        self.zero_window_search_fail = 0;
        self.special_patch_extensions = 0;
        self.special_tile_extensions = 0;
        self.late_move_reductions = 0;
        self.late_move_reduction_fails = 0;
        self.late_move_pruning = 0;
    }

    /// Resets some statistics after one iteration of iterative deepening.
    ///
    /// Resets:
    /// * The number of nodes searched.
    /// * The number of leaf nodes searched.
    /// * The time when the search started.
    /// * The number of times the search failed high.
    /// * The number of times the search failed high on the first node.
    /// * The number of times the aspiration window failed low.
    /// * The number of times the aspiration window failed high.
    /// * The number of times the zero window search was performed.
    /// * The number of times the zero window search failed.
    /// * The number of times a special patch extension was made.
    /// * The number of times late move reductions were performed.
    /// * The number of times late move reductions were performed and failed.
    /// * The number of times late move pruning was performed.
    ///
    /// Sets:
    /// * The number of nodes searched in the previous iteration to the number of nodes searched.
    #[inline]
    pub fn reset_iterative_deepening_iteration(&mut self) {
        if !ACTIVE {
            return;
        }
        self.nodes_searched_previous_iteration = self.nodes_searched;
        self.nodes_searched = 0;
        self.leaf_nodes_searched = 0;
        self.start_time = std::time::Instant::now();
        self.fail_high = 0;
        self.fail_high_first = 0;
        self.aspiration_window_fail_low = 0;
        self.aspiration_window_fail_high = 0;
        self.zero_window_search = 0;
        self.zero_window_search_fail = 0;
        self.special_patch_extensions = 0;
        self.special_tile_extensions = 0;
        self.late_move_reductions = 0;
        self.late_move_reduction_fails = 0;
        self.late_move_pruning = 0;
    }

    /// Increments the number of nodes searched.
    #[inline]
    pub fn increment_nodes_searched(&mut self) {
        if !ACTIVE {
            return;
        }
        self.nodes_searched += 1;
    }

    /// Increments the number of leaf nodes searched.
    #[inline]
    pub fn increment_leaf_nodes_searched(&mut self) {
        if !ACTIVE {
            return;
        }
        self.leaf_nodes_searched += 1;
    }

    /// Increments the number of times the search failed high.
    ///
    /// # Arguments
    ///
    /// * `first` - Whether the search failed high on the first / pv node.
    #[inline]
    pub fn increment_fail_high(&mut self, first: bool) {
        if !ACTIVE {
            return;
        }
        if first {
            self.fail_high_first += 1;
        }
        self.fail_high += 1;
    }

    /// Increments the number of times the aspiration window failed low.
    #[inline]
    pub fn increment_aspiration_window_fail_low(&mut self) {
        if !ACTIVE {
            return;
        }
        self.aspiration_window_fail_low += 1;
    }

    /// Increments the number of times the aspiration window failed high.
    #[inline]
    pub fn increment_aspiration_window_fail_high(&mut self) {
        if !ACTIVE {
            return;
        }
        self.aspiration_window_fail_high += 1;
    }

    /// Increments the number of times the zero window search was performed.
    #[inline]
    pub fn increment_zero_window_search(&mut self) {
        if !ACTIVE {
            return;
        }
        self.zero_window_search += 1;
    }

    /// Increments the number of times the zero window search failed.
    #[inline]
    pub fn increment_zero_window_search_fail(&mut self) {
        if !ACTIVE {
            return;
        }
        self.zero_window_search_fail += 1;
    }

    /// Increments the number of times a special patch extension was made.
    #[inline]
    pub fn increment_special_patch_extensions(&mut self) {
        if !ACTIVE {
            return;
        }
        self.special_patch_extensions += 1;
    }

    /// Increments the number of times a special tile extension was made.
    #[inline]
    pub fn increment_special_tile_extensions(&mut self) {
        if !ACTIVE {
            return;
        }
        self.special_tile_extensions += 1;
    }

    /// Increments the number of times late move reductions were performed.
    #[inline]
    pub fn increment_late_move_reductions(&mut self) {
        if !ACTIVE {
            return;
        }
        self.late_move_reductions += 1;
    }

    // Increment the number of times late move reductions were performed.
    #[inline]
    pub fn increment_late_move_pruning(&mut self) {
        if !ACTIVE {
            return;
        }
        self.late_move_pruning += 1;
    }

    /// Increments the number of times late move reductions were performed and failed.
    #[inline]
    pub fn increment_late_move_reduction_fails(&mut self) {
        if !ACTIVE {
            return;
        }
        self.late_move_reduction_fails += 1;
    }

    /// Returns the rate of failing zero window searches in relation to the number of zero window searches.
    #[inline]
    #[must_use]
    pub fn zero_window_search_fail_rate(&self) -> f64 {
        if self.zero_window_search == 0 {
            return 0.0;
        }

        self.zero_window_search_fail as f64 / self.zero_window_search as f64
    }

    /// Return the rate of failing late move reduction searches in relation to the number of late move reduction searches.
    #[inline]
    #[must_use]
    pub fn late_move_reduction_fail_rate(&self) -> f64 {
        if self.late_move_reductions == 0 {
            return 0.0;
        }

        self.late_move_reduction_fails as f64 / self.late_move_reductions as f64
    }
}
