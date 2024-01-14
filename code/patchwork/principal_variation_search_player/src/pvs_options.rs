use action_sorter::{ActionSorter, NoopActionSorter};
use evaluator::StaticEvaluator;
use patchwork_core::StableEvaluator;
use transposition_table::Size;

/// Different options for the Principal Variation Search (PVS) algorithm.
pub struct PVSOptions {
    /// The time limit for the search.
    pub time_limit: std::time::Duration,
    /// The evaluator to evaluate the game state.
    pub evaluator: Box<dyn StableEvaluator>,
    /// The action sorter to sort the actions.
    pub action_sorter: Box<dyn ActionSorter>,
    /// The features to enable or disable.
    pub features: PVSFeatures,
}

impl PVSOptions {
    /// Creates a new [`PVSOptions`].
    pub fn new(
        time_limit: std::time::Duration,
        evaluator: Box<dyn StableEvaluator>,
        action_sorter: Box<dyn ActionSorter>,
        features: PVSFeatures,
    ) -> Self {
        Self {
            time_limit,
            evaluator,
            action_sorter,
            features,
        }
    }
}

impl Default for PVSOptions {
    fn default() -> Self {
        Self {
            time_limit: std::time::Duration::from_secs(20), // TODO: real time limit
            evaluator: Box::<StaticEvaluator>::default(),
            action_sorter: Box::<NoopActionSorter>::default(),
            features: Default::default(),
        }
    }
}

/// Different features that can be enabled or disabled for the pvs player.
pub struct PVSFeatures {
    /// The failing strategy to use.
    pub failing_strategy: FailingStrategy,
    /// If [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows) should be used.
    pub aspiration_window: bool,
    // If a [Transposition Table](https://www.chessprogramming.org/Transposition_Table) should be used.
    pub transposition_table: TranspositionTableFeature,
    /// If [Late Move Reductions](https://www.chessprogramming.org/Late_Move_Reductions) should be used.
    pub late_move_reductions: bool,
    /// If [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning) should be used.
    pub late_move_pruning: bool,
    /// If [Extensions](https://www.chessprogramming.org/Extensions) should be used for special patches.
    pub search_extensions: bool,
    /// If diagnostics should be printed.
    pub diagnostics: DiagnosticsFeature,
    // TODO: Other Features
    // late_move_pruning: bool,
    // null_move_pruning: bool,
    // internal_iterative_deepening: bool,
    // lazy_smp: Enum { No, Yes(parallelism) }
}

impl Default for PVSFeatures {
    fn default() -> Self {
        Self {
            failing_strategy: FailingStrategy::FailHard,
            aspiration_window: false, // TODO: reenable
            transposition_table: Default::default(),
            late_move_reductions: true,
            late_move_pruning: true,
            search_extensions: true,
            diagnostics: Default::default(),
        }
    }
}

/// Different options for the failing strategy.
///
/// The failing strategy determines how the search behaves when a fail-high or fail-low occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailingStrategy {
    /// The failing strategy is [Fail-Soft](https://www.chessprogramming.org/Fail-Soft).
    ///
    /// This means the returned evaluation might be outside the bounds:
    /// * An upper bound less than alpha at All-Nodes
    /// * A lower bound greater than beta at Cut-Nodes
    FailSoft,
    /// The failing strategy is [Fail-Hard](https://www.chessprogramming.org/Fail-Hard).
    ///
    /// This means the returned evaluation will always be within the bounds of
    /// the alpha-beta window (Alpha <= Evaluation <= Beta).
    FailHard,
}

/// Different options for the transposition table feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranspositionTableFeature {
    /// The transposition table is disabled.
    Disabled,
    /// The transposition table is enabled with the given size.
    Enabled { size: Size, strategy: FailingStrategy },
    /// The transposition table is enabled and for a position all symmetric
    /// positions are stored in the table as well.
    SymmetryEnabled { size: Size, strategy: FailingStrategy },
}

impl Default for TranspositionTableFeature {
    fn default() -> Self {
        Self::SymmetryEnabled {
            size: Size::MiB(10),
            strategy: FailingStrategy::FailHard,
        }
    }
}

/// Different options for the diagnostics feature.
pub enum DiagnosticsFeature {
    /// No diagnostics are printed.
    Disabled,
    /// Diagnostics are printed to the writer.
    Enabled { writer: Box<dyn std::io::Write> },
    /// Verbose diagnostics are printed to the writer.
    /// This includes a printout of 100 entries in the transposition table if the transposition table feature is enabled.
    Verbose { writer: Box<dyn std::io::Write> },
}

impl Default for DiagnosticsFeature {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Enabled {
                writer: Box::new(std::io::stdout()),
            }
        } else {
            Self::Disabled
        }
    }
}
