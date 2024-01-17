use patchwork_core::ActionId;

/// A list of actions with their scores.
pub struct ActionList<'a> {
    /// The actions.
    pub(crate) actions: &'a mut [ActionId],
    /// The scores for the actions.
    pub(crate) scores: &'a mut [isize],
}

impl<'a> ActionList<'a> {
    /// Creates a new [`ActionList`].
    ///
    /// # Arguments
    ///
    /// * `actions` - The actions.
    /// * `scores` - The scores for the actions.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn new(actions: &'a mut [ActionId], scores: &'a mut [isize]) -> Self {
        Self { actions, scores }
    }

    /// Returns the amount of actions.
    ///
    /// # Returns
    ///
    /// The amount of actions.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns if the list is empty.
    ///
    /// # Returns
    ///
    /// If the list is empty.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the action at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the action.
    ///
    /// # Returns
    ///
    /// The action at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_action(&self, index: usize) -> ActionId {
        self.actions[index]
    }

    /// Returns the score at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the score.
    ///
    /// # Returns
    ///
    /// The score at the given index.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn get_score(&self, index: usize) -> isize {
        self.scores[index]
    }

    /// Swaps the actions at the given indices in place.
    ///
    /// # Arguments
    ///
    /// * `index1` - The index of the first action.
    /// * `index2` - The index of the second action.
    ///
    /// # Complexity
    ///
    /// `𝒪(𝟣)`
    pub fn swap(&mut self, index1: usize, index2: usize) {
        self.actions.swap(index1, index2);
        self.scores.swap(index1, index2);
    }
}
