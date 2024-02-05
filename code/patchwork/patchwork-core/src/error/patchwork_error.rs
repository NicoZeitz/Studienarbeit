use thiserror::Error;

use crate::{ActionId, Patchwork};

#[derive(Debug, Error, Clone, Eq, PartialEq, Hash)]
pub enum PatchworkError {
    #[error("Invalid action in the current state of the game")]
    InvalidActionError {
        reason: &'static str,
        action: ActionId,
        state: Box<Patchwork>,
    },
    #[error("The Game is in its initial state and no actions can be undone")]
    GameStateIsInitialError,
    #[error("The notation string representation is invalid")]
    InvalidNotationError { notation: String, reason: &'static str },
    #[error("The given range is invalid")]
    InvalidRangeError { reason: &'static str },
}
