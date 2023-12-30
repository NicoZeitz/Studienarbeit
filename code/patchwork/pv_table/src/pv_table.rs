use std::ops::{Index, IndexMut};

use patchwork_core::ActionId;

/// Principal Variation Table
///
/// A table that stores the principal variation of a game state.
/// The principal variations are stored as a [Triangular PV-Table](https://www.chessprogramming.org/Triangular_PV-Table)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PVTable {
    // FEATURE:PV_TABLE
    /// A triangular table of actions.
    ///
    /// The maximum size of the table is `(MAX_DEPTH + 1) * MAX_DEPTH / 2`
    /// the [Triangular number](https://en.wikipedia.org/wiki/Triangular_number).
    ///
    ///
    /// The current `MAX_DEPTH` is `256`. With this the table has a maximum of
    /// `32896` entries. Each entry is 4 bytes big. This means the table can
    /// have a size of `≈0.13 MB`. Theoretically we could store the whole table
    /// on the stack, but this would be a wast of memory as well as producing
    /// a stack overflow.
    table: Vec<ActionId>,
}

impl PVTable {
    /// Create a new [`PVTable`]
    pub fn new() -> Self {
        Self { table: vec![] }
    }

    /// Insert a new action into the table.
    ///
    /// If the index is bigger than the current size of the table, the table
    /// will be resized to fit the new index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the action in the principal variation.
    /// * `action` - The action to insert.
    ///
    /// # Example
    ///
    /// ```
    /// use patchwork_core::ActionId;
    /// use pv_table::PVTable;
    ///
    /// let mut pv_table = PVTable::new();
    ///
    /// pv_table.insert(0, ActionId::walking(13));
    /// pv_table.insert(1, ActionId::null());
    ///
    /// assert_eq!(pv_table.get(0), Some(ActionId::walking(13)));
    /// assert_eq!(pv_table.get(1), Some(ActionId::null()));
    /// ```
    ///
    /// # Complexity
    ///
    /// `𝒪(𝑛)` where `𝑛` is the index of the action. This is because the table
    /// has to be resized to fit the new action.
    ///
    /// The amortized complexity is `𝒪(𝟣)`.
    pub fn insert(&mut self, index: usize, action: ActionId) {
        if index >= self.table.len() {
            self.table.resize(index + 1, ActionId::null());
        }
        self.table[index] = action;
    }

    /// Get the action at the given index.
    /// If the index is bigger than the current size of the table, `None` is
    /// returned.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the action in the principal variation.
    ///
    /// # Example
    ///
    /// ```
    /// use patchwork_core::ActionId;
    /// use pv_table::PVTable;
    ///
    /// let mut pv_table = PVTable::new();
    ///
    /// pv_table.insert(0, ActionId::walking(13));
    ///
    /// assert_eq!(pv_table.get(0), Some(ActionId::walking(13)));
    /// ```
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)` where `𝟣` is the index of the action.
    pub fn get(&self, index: usize) -> Option<ActionId> {
        if index < self.table.len() {
            Some(self.table[index])
        } else {
            None
        }
    }
}

impl Default for PVTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for PVTable {
    type Output = ActionId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl IndexMut<usize> for PVTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}
