/// The diagnostics of a search.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchDiagnostics {
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
    /// The number of times late move pruning was performed.
    pub late_move_pruning: usize,
    // TODO: pv/zws hits, ...
}

impl Default for SearchDiagnostics {
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
            late_move_pruning: 0,
        }
    }
}

impl SearchDiagnostics {
    #[inline(always)]
    pub fn reset(&mut self) {
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
        self.late_move_pruning = 0;
    }

    /// Resets some diagnostics after one iteration of iterative deepening.
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
    /// * The number of times late move pruning was performed.
    ///
    /// Sets:
    /// * The number of nodes searched in the previous iteration to the number of nodes searched.
    #[inline(always)]
    pub fn reset_iterative_deepening_iteration(&mut self) {
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
        self.late_move_pruning = 0;
    }

    /// Increments the number of nodes searched.
    #[inline(always)]
    pub fn increment_nodes_searched(&mut self) {
        self.nodes_searched += 1;
    }

    /// Increments the number of leaf nodes searched.
    #[inline(always)]
    pub fn increment_leaf_nodes_searched(&mut self) {
        self.leaf_nodes_searched += 1;
    }

    /// Increments the number of times the search failed high.
    ///
    /// # Arguments
    ///
    /// * `first` - Whether the search failed high on the first / pv node.
    #[inline(always)]
    pub fn increment_fail_high(&mut self, first: bool) {
        if first {
            self.fail_high_first += 1;
        }
        self.fail_high += 1;
    }

    /// Increments the number of times the aspiration window failed low.
    #[inline(always)]
    pub fn increment_aspiration_window_fail_low(&mut self) {
        self.aspiration_window_fail_low += 1;
    }

    /// Increments the number of times the aspiration window failed high.
    #[inline(always)]
    pub fn increment_aspiration_window_fail_high(&mut self) {
        self.aspiration_window_fail_high += 1;
    }

    /// Increments the number of times the zero window search was performed.
    #[inline(always)]
    pub fn increment_zero_window_search(&mut self) {
        self.zero_window_search += 1;
    }
    /// Increments the number of times the zero window search failed.
    #[inline(always)]
    pub fn increment_zero_window_search_fail(&mut self) {
        self.zero_window_search_fail += 1;
    }

    /// Increments the number of times a special patch extension was made.
    #[inline(always)]
    pub fn increment_special_patch_extensions(&mut self) {
        self.special_patch_extensions += 1;
    }

    /// Increments the number of times a special tile extension was made.
    #[inline(always)]
    pub fn increment_special_tile_extensions(&mut self) {
        self.special_tile_extensions += 1;
    }

    /// Increments the number of times late move reductions were performed.
    #[inline(always)]
    pub fn increment_late_move_reductions(&mut self) {
        self.late_move_reductions += 1;
    }

    // Increment the number of times late move reductions were performed.
    #[inline(always)]
    pub fn increment_late_move_pruning(&mut self) {
        self.late_move_pruning += 1;
    }

    /// Returns the rate of failing zero window searches in relation to the number of zero window searches.
    #[inline(always)]
    pub fn zero_window_search_fail_rate(&self) -> f64 {
        self.zero_window_search_fail as f64 / self.zero_window_search as f64
    }
}
