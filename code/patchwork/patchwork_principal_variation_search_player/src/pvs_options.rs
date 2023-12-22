/// Different options for the Principal Variation Search (PVS) algorithm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PVSOptions {
    pub time_limit: std::time::Duration,
}

impl PVSOptions {
    /// Creates a new [`PVSOptions`].
    pub fn new(time_limit: std::time::Duration) -> Self {
        Self { time_limit }
    }
}

impl Default for PVSOptions {
    fn default() -> Self {
        Self {
            time_limit: std::time::Duration::from_secs(10),
        }
    }
}
