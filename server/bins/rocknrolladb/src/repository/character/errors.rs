//! Character domain error variants, mapped onto the shared `ServiceError` categories.

use crate::error::ServiceError;
use spacetimedb::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CharacterError {
    #[error("piece references unknown character '{character_id}'")]
    UnknownCharacterForPiece { character_id: Uuid },
    #[error("unknown piece '{piece_id}'")]
    UnknownPiece { piece_id: Uuid },
}

impl CharacterError {
    pub fn unknown_character_for_piece(character_id: Uuid) -> ServiceError {
        CharacterError::UnknownCharacterForPiece { character_id }.into()
    }

    pub fn unknown_piece(piece_id: Uuid) -> ServiceError {
        CharacterError::UnknownPiece { piece_id }.into()
    }
}

impl From<CharacterError> for ServiceError {
    fn from(err: CharacterError) -> Self {
        ServiceError::not_found(err.to_string())
    }
}
