/// The diagnostics of a search.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchDiagnostics {
    /// The number of nodes searched.
    pub nodes_searched: usize,
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
    /// The number of times the zero window search failed.
    pub zero_window_search_fail: usize,
    /// The number of times a special patch extension was made.
    pub special_patch_extensions: usize,
    // TODO: pv/zws hits, ...
}

impl Default for SearchDiagnostics {
    fn default() -> Self {
        Self {
            nodes_searched: 0,
            start_time: std::time::Instant::now(),
            fail_high: 0,
            fail_high_first: 0,
            aspiration_window_fail_low: 0,
            aspiration_window_fail_high: 0,
            zero_window_search_fail: 0,
            special_patch_extensions: 0,
        }
    }
}

impl SearchDiagnostics {
    /// Increments the number of nodes searched.
    #[inline]
    pub fn increment_nodes_searched(&mut self) {
        self.nodes_searched += 1;
    }

    /// Resets some diagnostics after one iteration of iterative deepening.
    ///
    /// Resets:
    /// * The number of nodes searched.
    /// * The number of times the aspiration window failed low.
    /// * The number of times the aspiration window failed high.
    #[inline]
    pub fn reset_iterative_deepening_iteration(&mut self) {
        self.nodes_searched = 0;
        self.aspiration_window_fail_low = 0;
        self.aspiration_window_fail_high = 0;
    }

    /// Increments the number of times the search failed high.
    ///
    /// # Arguments
    ///
    /// * `first` - Whether the search failed high on the first / pv node.
    #[inline]
    pub fn increment_fail_high(&mut self, first: bool) {
        if first {
            self.fail_high_first += 1;
        }
        self.fail_high += 1;
    }

    /// Increments the number of times the aspiration window failed low.
    #[inline]
    pub fn increment_aspiration_window_fail_low(&mut self) {
        self.aspiration_window_fail_low += 1;
    }

    /// Increments the number of times the aspiration window failed high.
    #[inline]
    pub fn increment_aspiration_window_fail_high(&mut self) {
        self.aspiration_window_fail_high += 1;
    }

    /// Increments the number of times the zero window search failed.
    #[inline]
    pub fn increment_zero_window_search_fail(&mut self) {
        self.zero_window_search_fail += 1;
    }

    /// Increments the number of times a special patch extension was made.
    #[inline]
    pub fn increment_special_patch_extensions(&mut self) {
        self.special_patch_extensions += 1;
    }
}
