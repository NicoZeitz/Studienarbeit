use std::{
    fmt::{Display, Formatter},
    num::NonZeroUsize,
};

use patchwork_core::Logging;
use transposition_table::Size;

/// Different options for the Principal Variation Search (PVS) algorithm.
pub struct PVSOptions {
    /// The time limit for the search.
    pub time_limit: std::time::Duration,
    /// The features to enable or disable.
    pub features: PVSFeatures,
    /// If logging configuration for what should be printed.
    pub logging: Logging,
}

impl PVSOptions {
    /// Creates a new [`PVSOptions`].
    ///
    /// # Panics
    ///
    /// When some features are used in combination that is not allowed
    #[must_use]
    pub fn new(time_limit: std::time::Duration, features: PVSFeatures, logging: Logging) -> Self {
        if matches!(features.lazy_smp, LazySMPFeature::Yes(_))
            && matches!(features.transposition_table, TranspositionTableFeature::Disabled)
        {
            panic!("The lazy SMP feature can only be enabled if the transposition table feature is enabled.");
        }

        Self {
            time_limit,
            features,
            logging,
        }
    }
}

impl Default for PVSOptions {
    fn default() -> Self {
        Self {
            time_limit: std::time::Duration::from_secs(10),
            features: PVSFeatures::default(),
            logging: Logging::default(),
        }
    }
}

/// Different features that can be enabled or disabled for the pvs player.
#[allow(clippy::struct_excessive_bools)]
pub struct PVSFeatures {
    /// The failing strategy to use.
    pub failing_strategy: FailingStrategy,
    /// If [Aspiration Windows](https://www.chessprogramming.org/Aspiration_Windows) should be used.
    pub aspiration_window: bool,
    /// If [Late Move Reductions](https://www.chessprogramming.org/Late_Move_Reductions) should be used.
    pub late_move_reductions: bool,
    /// If [Late Move Pruning](https://disservin.github.io/stockfish-docs/pages/Terminology.html#late-move-pruning)
    /// should be used.
    pub late_move_pruning: bool,
    /// If [Extensions](https://www.chessprogramming.org/Extensions) should be used for special patches.
    pub search_extensions: bool,
    // If a [Transposition Table](https://www.chessprogramming.org/Transposition_Table) should be used.
    pub transposition_table: TranspositionTableFeature,
    /// If [Lazy SMP](https://www.chessprogramming.org/Lazy_SMP) should be used. Requires the transposition table
    /// feature to be enabled.
    pub lazy_smp: LazySMPFeature,
}

impl Default for PVSFeatures {
    fn default() -> Self {
        Self {
            failing_strategy: FailingStrategy::FailHard,
            aspiration_window: true,
            transposition_table: TranspositionTableFeature::default(),
            late_move_reductions: true,
            late_move_pruning: true,
            search_extensions: true,
            lazy_smp: LazySMPFeature::default(),
        }
    }
}

/// Different options for the lazy Symmetric multiprocessing (Lazy SMP) feature.
///
/// The lazy SMP feature is used to parallelize the search by sharing a
/// transposition table between multiple threads. Because of this the lazy SMP
/// feature can only be enabled if the transposition table feature is enabled.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LazySMPFeature {
    /// The lazy SMP feature is disabled.
    No,
    /// The lazy SMP feature is enabled with the given parallelism.
    Yes(NonZeroUsize),
}

impl Display for LazySMPFeature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::No => write!(f, "No"),
            Self::Yes(parallelism) => write!(f, "Yes(Parallelism: {parallelism})"),
        }
    }
}

impl Default for LazySMPFeature {
    fn default() -> Self {
        std::thread::available_parallelism()
            .map(|n| unsafe { NonZeroUsize::new_unchecked(n.get() / 2) })
            .map_or(Self::No, Self::Yes)
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

impl Display for FailingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailSoft => write!(f, "Soft"),
            Self::FailHard => write!(f, "Hard"),
        }
    }
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

impl Display for TranspositionTableFeature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "Disabled"),
            Self::Enabled { size, strategy } => write!(f, "Enabled(Size: {size:?}, Strategy: {strategy})"),
            Self::SymmetryEnabled { size, strategy } => {
                write!(f, "Symmetry(Size: {size:?}, Strategy: {strategy})")
            }
        }
    }
}

impl TranspositionTableFeature {
    pub const DEFAULT_SIZE: Size = Size::MiB(250);
    pub const DEFAULT_STRATEGY: FailingStrategy = FailingStrategy::FailHard;
}

impl Default for TranspositionTableFeature {
    fn default() -> Self {
        Self::SymmetryEnabled {
            size: Self::DEFAULT_SIZE,
            strategy: Self::DEFAULT_STRATEGY,
        }
    }
}
