/// The size of the transposition table.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Size {
    /// The size in bytes.
    B(u64),
    /// The size in kilobytes (1000 bytes).
    KB(u64),
    /// The size in megabytes (10^6 bytes)
    MB(u64),
    /// The size in gigabytes (10^9 bytes)
    GB(u64),
    /// The size in kibibytes (1024 bytes).
    KiB(u64),
    /// The size in mebibytes (2^20 bytes).
    MiB(u64),
    /// The size in gibibytes (2^30 bytes).
    GiB(u64),
}
