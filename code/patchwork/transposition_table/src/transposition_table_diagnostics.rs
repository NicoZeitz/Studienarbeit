use std::sync::atomic::AtomicUsize;

use patchwork_core::Notation;

use crate::{Entry, TranspositionTable};

/// Diagnostics for the transposition table.
#[derive(Debug)]
pub struct TranspositionTableDiagnostics {
    // ─────────── general ───────────
    /// The capacity of the transposition table (in Entries).
    pub capacity: AtomicUsize,
    /// The amount of entries in the transposition table.
    pub entries: AtomicUsize,
    // ─────────── probe ───────────
    /// The amount of accesses to the transposition table.
    pub accesses: AtomicUsize,
    /// The amount of misses to the transposition table.
    pub misses: AtomicUsize,
    // ─────────── store ───────────
    /// The amount of overwrites to the transposition table.
    pub overwrites: AtomicUsize,
}

impl TranspositionTableDiagnostics {
    pub const LOAD_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::SeqCst;
    pub const STORE_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::SeqCst;

    /// Creates a new [`TranspositionTableDiagnostics`].
    ///
    /// # Arguments
    ///
    /// * `capacity_in_entries` - The capacity of the transposition table (in Entries).
    pub fn new(capacity_in_entries: usize) -> Self {
        Self {
            capacity: AtomicUsize::new(capacity_in_entries),
            entries: AtomicUsize::new(0),
            accesses: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            overwrites: AtomicUsize::new(0),
        }
    }

    // ──────────────────────────────────────────── GETTERS ────────────────────────────────────────────

    /// Gets the amount of transposition table hits
    pub fn hits(&self) -> usize {
        self.accesses.load(Self::LOAD_ORDERING) - self.misses.load(Self::LOAD_ORDERING)
    }

    pub fn fill_ratio(&self) -> f64 {
        self.entries.load(Self::LOAD_ORDERING) as f64 / self.capacity.load(Self::LOAD_ORDERING) as f64
    }

    /// Gets the amount of transposition table hits as a ration to the amount of accesses
    pub fn hit_ratio(&self) -> f64 {
        self.hits() as f64 / self.accesses.load(Self::LOAD_ORDERING) as f64
    }

    /// Gets the amount of transposition table misses as a ration to the amount of accesses
    pub fn miss_ratio(&self) -> f64 {
        self.misses.load(Self::LOAD_ORDERING) as f64 / self.accesses.load(Self::LOAD_ORDERING) as f64
    }

    // ──────────────────────────────────────────── SETTERS ────────────────────────────────────────────

    /// Increments the amount of entries in the transposition table.
    /// This should only be called when an entry is stored.
    pub fn increment_entries(&self) {
        self.entries.fetch_add(1, Self::STORE_ORDERING);
    }

    /// Increments the amount of accesses to the transposition table.
    /// This should only be called when an entry is found.
    pub fn increment_accesses(&self) {
        self.accesses.fetch_add(1, Self::STORE_ORDERING);
    }

    /// Increments the amount of misses to the transposition table.
    /// This should only be called when an entry is not found.
    pub fn increment_misses(&self) {
        self.misses.fetch_add(1, Self::STORE_ORDERING);
    }

    /// Increments the amount of overwrites to the transposition table.
    /// This should only be called when an entry is overwritten.
    pub fn increment_overwrites(&self) {
        self.overwrites.fetch_add(1, Self::STORE_ORDERING);
    }

    /// Resets the diagnostics of the transposition table.
    pub fn reset_diagnostics(&mut self) {
        self.entries.store(0, Self::STORE_ORDERING);
        self.accesses.store(0, Self::STORE_ORDERING);
        self.misses.store(0, Self::STORE_ORDERING);
        self.overwrites.store(0, Self::STORE_ORDERING);
    }

    // ───────────────────────────────────────────── OTHER ─────────────────────────────────────────────

    /// Prints the diagnostics of the transposition table.
    ///
    /// # Arguments
    ///
    /// * `printer` - The printer to print the diagnostics to.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the printing.
    #[rustfmt::skip]
    pub fn write_diagnostics(&self, writer: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        writeln!(writer, "┌────────── Transposition Table Diagnostics ──────────┐")?;
        writeln!(writer, "│Capacity:   {: >17}                        │", self.capacity.load(Self::LOAD_ORDERING))?;
        writeln!(writer, "│Entries:    {: >17} / {:6.2}% filled       │", self.entries.load(Self::LOAD_ORDERING), self.fill_ratio() * 100.0)?;
        writeln!(writer, "│Overwrites: {: >17}                        │", self.overwrites.load(Self::LOAD_ORDERING))?;
        writeln!(writer, "│Accesses:   {: >17}                        │", self.accesses.load(Self::LOAD_ORDERING))?;
        writeln!(writer, "│├──► Hit:   {: >17} / {:6.2}%              │", self.hits(), self.hit_ratio() * 100.0)?;
        writeln!(writer, "│└──► Miss:  {: >17} / {:6.2}%              │", self.misses.load(Self::LOAD_ORDERING), self.miss_ratio() * 100.0)?;
        writeln!(writer, "└─────────────────────────────────────────────────────┘")
    }

    /// Prints the transposition table.
    ///
    /// # Arguments
    ///
    /// * `printer` - The printer to print the transposition table to.
    /// * `transposition_table` - The transposition table to print.
    /// * `max_entries` - The maximum amount of entries to print. If `None` is given all entries are printed.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - The result of the printing.
    #[rustfmt::skip]
    pub fn write_transposition_table(&self, writer: &mut dyn std::io::Write, transposition_table: &TranspositionTable, max_entries: Option<usize>) -> Result<(), std::io::Error> {
        writeln!(writer, "┌────────┬───────────────────── Transposition Table Entries ────────────┬─────────────────┐")?;
        writeln!(writer, "│ Index  │         Key          │ Depth │ Age │    Type    │ Evaluation │    Action       │")?;
        writeln!(writer, "├────────┼──────────────────────┼───────┼─────┼────────────┼────────────┼─────────────────┤")?;
        let max_entries_value = max_entries.unwrap_or(0);
        let mut written_entries = 0;
        for (index, entry) in transposition_table.entries.iter().enumerate() {
            if entry.key == 0 {
                continue;
            }


            if max_entries.is_some() && written_entries > max_entries_value {
                writeln!(writer, "│  ...   │         ...          │  ...  │ ... │     ...    │     ...    │       ...       │")?;
                break;
            } else {
                let (table_depth, table_evaluation, table_evaluation_type, table_action) = Entry::unpack_data(entry.data);
                writeln!(writer, "│{: >7?} │ {: >20?} | {: >5?} | {: >3?} | {: >10} | {: >10?} |{: >16} │", index, entry.key, table_depth, entry.age, format!("{:?}", table_evaluation_type), table_evaluation, table_action.save_to_notation().unwrap_or("######".to_string()))?;
                written_entries += 1;
            }
        }
        writeln!(writer, "└────────┴──────────────────────┴───────┴─────┴────────────┴────────────┴─────────────────┘")
    }
}
