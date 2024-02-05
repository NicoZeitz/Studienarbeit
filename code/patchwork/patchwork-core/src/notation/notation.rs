use crate::PatchworkError;

/// Saves and loads the given type to and from a notation shorthand.
pub trait Notation: std::marker::Sized {
    /// Saves the given type to a notation shorthand.
    fn save_to_notation(&self) -> Result<String, PatchworkError>;

    /// Loads the given notation into the given type.
    fn load_from_notation(notation: &str) -> Result<Self, PatchworkError>;
}
