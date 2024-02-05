use std::fmt;

/// The diagnostics configuration.
///
/// The diagnostics are used to collect information about the progress of the player. This can be
/// used to debug the player and to understand the behavior of the player. The diagnostics can be
/// disabled, enabled or enabled with verbose output.
pub enum Diagnostics {
    /// The diagnostics are disabled.
    Disabled,
    /// The diagnostics are enabled. The progress is written to the given writer which is usually
    /// `std::io::stdout()` or a comparable console.
    Enabled {
        progress_writer: Box<dyn std::io::Write>,
    },
    /// The verbose diagnostics are enabled. The progress is written to the given writer which is
    /// usually `std::io::stdout()` or a comparable console. Additionally, the debug information is
    /// written to the given writer which is usually a file.
    Verbose {
        progress_writer: Box<dyn std::io::Write>,
        debug_writer: Box<dyn std::io::Write>,
    },
    VerboseOnly {
        debug_writer: Box<dyn std::io::Write>,
    },
}

impl Diagnostics {
    /// Indicates if the diagnostics are enabled.
    ///
    /// # Returns
    ///
    /// `true` if the diagnostics are enabled, `false` otherwise.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Diagnostics::Disabled)
    }
}

impl fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diagnostics::Disabled => write!(f, "Diagnostics::Disabled"),
            Diagnostics::Enabled { .. } => write!(f, "Diagnostics::Enabled"),
            Diagnostics::Verbose { .. } => write!(f, "Diagnostics::Verbose"),
            Diagnostics::VerboseOnly { .. } => write!(f, "Diagnostics::VerboseOnly"),
        }
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Diagnostics::Enabled {
                progress_writer: Box::new(std::io::stdout()),
            }
        } else {
            Diagnostics::Disabled
        }
    }
}
