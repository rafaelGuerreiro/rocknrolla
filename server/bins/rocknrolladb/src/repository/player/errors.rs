//! Player domain error variants, mapped onto the shared `ServiceError` categories.

use crate::error::ServiceError;
use spacetimedb::{Identity, Uuid};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("player not initialized")]
    NotInitialized,
    #[error("character '{character_id}' is not unlocked")]
    CharacterNotUnlocked { owner: Identity, character_id: Uuid },
}

impl PlayerError {
    pub fn not_initialized() -> ServiceError {
        PlayerError::NotInitialized.into()
    }

    pub fn character_not_unlocked(owner: Identity, character_id: Uuid) -> ServiceError {
        PlayerError::CharacterNotUnlocked { owner, character_id }.into()
    }
}

impl From<PlayerError> for ServiceError {
    fn from(err: PlayerError) -> Self {
        let message = err.to_string();
        match err {
            PlayerError::NotInitialized => ServiceError::conflict(message),
            PlayerError::CharacterNotUnlocked { owner, .. } => ServiceError::forbidden(owner, message),
        }
    }
}
