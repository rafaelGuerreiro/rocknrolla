//! Lootbox domain error variants, mapped onto the shared `ServiceError` categories.

use crate::error::ServiceError;
use spacetimedb::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LootboxError {
    #[error("lootbox must configure at least one drop")]
    NoDrops,
    #[error("drop '{piece_id}' must have a positive weight")]
    ZeroWeight { piece_id: Uuid },
    #[error("drop references unknown piece '{piece_id}'")]
    UnknownDropPiece { piece_id: Uuid },
    #[error("unknown lootbox")]
    UnknownLootbox,
    #[error("lootbox already opened")]
    AlreadyOpened,
    #[error("lootbox '{lootbox_id}' has no weighted drops")]
    NoWeightedDrops { lootbox_id: Uuid },
    #[error("weighted pick failed")]
    WeightedPickFailed,
}

impl LootboxError {
    pub fn no_drops() -> ServiceError {
        LootboxError::NoDrops.into()
    }

    pub fn zero_weight(piece_id: Uuid) -> ServiceError {
        LootboxError::ZeroWeight { piece_id }.into()
    }

    pub fn unknown_drop_piece(piece_id: Uuid) -> ServiceError {
        LootboxError::UnknownDropPiece { piece_id }.into()
    }

    pub fn unknown_lootbox() -> ServiceError {
        LootboxError::UnknownLootbox.into()
    }

    pub fn already_opened() -> ServiceError {
        LootboxError::AlreadyOpened.into()
    }

    pub fn no_weighted_drops(lootbox_id: Uuid) -> ServiceError {
        LootboxError::NoWeightedDrops { lootbox_id }.into()
    }

    pub fn weighted_pick_failed() -> ServiceError {
        LootboxError::WeightedPickFailed.into()
    }
}

impl From<LootboxError> for ServiceError {
    fn from(err: LootboxError) -> Self {
        let message = err.to_string();
        match err {
            LootboxError::NoDrops | LootboxError::ZeroWeight { .. } | LootboxError::NoWeightedDrops { .. } => {
                ServiceError::validation(message)
            },
            LootboxError::UnknownDropPiece { .. } | LootboxError::UnknownLootbox => ServiceError::not_found(message),
            LootboxError::AlreadyOpened => ServiceError::conflict(message),
            LootboxError::WeightedPickFailed => ServiceError::internal(message),
        }
    }
}
