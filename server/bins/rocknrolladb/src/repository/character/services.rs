//! Character repository services: content import and piece/character lookups.

use crate::{
    error::{ServiceError, ServiceResult},
    repository::character::{CharacterDef, PieceDef, character_def, piece_def},
};
use spacetimedb::{ReducerContext, Uuid};
use std::ops::Deref;

pub trait CharacterReducerContext {
    fn character_services(&self) -> CharacterServices<'_>;
}

impl CharacterReducerContext for ReducerContext {
    fn character_services(&self) -> CharacterServices<'_> {
        CharacterServices { ctx: self }
    }
}

pub struct CharacterServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for CharacterServices<'_> {
    type Target = ReducerContext;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl CharacterServices<'_> {
    /// Overwrite one authored character definition by its stable UUID.
    pub fn import_character(&self, row: CharacterDef) -> ServiceResult<()> {
        self.db.character_def().id().insert_or_update(row);
        Ok(())
    }

    /// Overwrite one authored piece definition, verifying its character.
    pub fn import_piece(&self, row: PieceDef) -> ServiceResult<()> {
        if self.db.character_def().id().find(row.character_id).is_none() {
            return Err(ServiceError::not_found(format!(
                "piece references unknown character '{}'",
                row.character_id
            )));
        }
        self.db.piece_def().id().insert_or_update(row);
        Ok(())
    }

    /// The starter character every new player should begin with:
    /// deterministically the lowest starter UUID.
    pub fn default_starter_character_id(&self) -> Option<Uuid> {
        self.starter_character_ids().into_iter().min()
    }

    pub fn starter_character_ids(&self) -> Vec<Uuid> {
        self.db.character_def().starter().filter(true).map(|c| c.id).collect()
    }

    pub fn find_piece(&self, piece_id: Uuid) -> ServiceResult<PieceDef> {
        self.db
            .piece_def()
            .id()
            .find(piece_id)
            .ok_or_else(|| ServiceError::not_found(format!("unknown piece '{piece_id}'")))
    }

    /// The rarity weight of the character owning `piece_id`, used to scale
    /// lootbox drop weights.
    pub fn piece_rarity_weight(&self, piece_id: Uuid) -> ServiceResult<u32> {
        let piece = self.find_piece(piece_id)?;
        let character =
            self.db.character_def().id().find(piece.character_id).ok_or_else(|| {
                ServiceError::not_found(format!("piece references unknown character '{}'", piece.character_id))
            })?;
        Ok(character.rarity_weight)
    }

    /// Every unique piece required to unlock `character_id`.
    pub fn piece_ids_of_character(&self, character_id: Uuid) -> Vec<Uuid> {
        self.db
            .piece_def()
            .character_id()
            .filter(character_id)
            .map(|piece| piece.id)
            .collect()
    }

    pub fn piece_exists(&self, piece_id: Uuid) -> bool {
        self.db.piece_def().id().find(piece_id).is_some()
    }
}
