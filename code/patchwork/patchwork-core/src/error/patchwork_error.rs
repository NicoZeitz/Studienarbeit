use thiserror::Error;

use crate::{ActionId, Patchwork};

#[derive(Debug, Error, Clone, Eq, PartialEq, Hash)]
pub enum PatchworkError {
    #[error("[PatchworkError::InvalidActionError] Invalid action in the current state of the game,  action: {action:?}, reason: {reason}, state: {state:?}")]
    InvalidActionError {
        reason: &'static str,
        action: ActionId,
        state: Box<Patchwork>,
    },
    #[error("[PatchworkError::GameStateIsInitialError] The Game is in its initial state and no actions can be undone")]
    GameStateIsInitialError,
    #[error("[PatchworkError::] The notation string representation is invalid ({notation}), reason: {reason}")]
    InvalidNotationError { notation: String, reason: &'static str },
    #[error("[PatchworkError::InvalidRangeError] The given range is invalid, reason: {reason}")]
    InvalidRangeError { reason: &'static str },
}
