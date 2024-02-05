use std::fmt;

/// The logging configuration.
///
/// Logging is used to collect information about the progress of the player. This can be
/// used to debug the player and to understand the behavior of the player. The logging configuration
/// can be disabled, enabled, enabled with verbose or only verbose output.
pub enum Logging {
    /// Logging is disabled.
    Disabled,
    /// Logging is enabled. The progress is written to the given writer which is usually
    /// `std::io::stdout()` or a comparable console.
    Enabled { progress_writer: Box<dyn std::io::Write> },
    /// Logging is enabled in verbose mode. The progress is written to the given writer which is
    /// usually `std::io::stdout()` or a comparable console. Additionally, the debug information is
    /// written to the given writer which is usually a file.
    Verbose {
        progress_writer: Box<dyn std::io::Write>,
        debug_writer: Box<dyn std::io::Write>,
    },
    /// Logging is enabled in verbose only mode. The normal progress information is not written and
    /// only the debug information is written to the given writer which is usually a file.
    VerboseOnly { debug_writer: Box<dyn std::io::Write> },
}

impl Logging {
    /// Indicates if logging is enabled.
    ///
    /// # Returns
    ///
    /// `true` if the logging is enabled, `false` otherwise.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Logging::Disabled)
    }
}

impl fmt::Debug for Logging {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Logging::Disabled => write!(f, "Logging::Disabled"),
            Logging::Enabled { .. } => write!(f, "Logging::Enabled"),
            Logging::Verbose { .. } => write!(f, "Logging::Verbose"),
            Logging::VerboseOnly { .. } => write!(f, "Logging::VerboseOnly"),
        }
    }
}

impl Default for Logging {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Logging::Enabled {
                progress_writer: Box::new(std::io::stdout()),
            }
        } else {
            Logging::Disabled
        }
    }
}
