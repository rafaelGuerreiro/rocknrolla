//! Character repository services: content import and piece/character lookups.

use crate::{
    error::ServiceResult,
    extend::make_service::make_service,
    repository::character::{CharacterDef, PieceDef, character_def_v1, errors::CharacterError, piece_def_v1},
};
use spacetimedb::Uuid;

make_service!(CharacterReducerContext, character_services, CharacterServices);

impl CharacterServices<'_> {
    /// Overwrite one authored character definition by its stable UUID.
    pub fn import_character(&self, row: CharacterDef) -> ServiceResult<()> {
        self.db.character_def_v1().id().insert_or_update(row);
        Ok(())
    }

    /// Overwrite one authored piece definition, verifying its character.
    pub fn import_piece(&self, row: PieceDef) -> ServiceResult<()> {
        if self.db.character_def_v1().id().find(row.character_id).is_none() {
            return Err(CharacterError::unknown_character_for_piece(row.character_id));
        }
        self.db.piece_def_v1().id().insert_or_update(row);
        Ok(())
    }

    /// The starter character every new player should begin with:
    /// deterministically the lowest starter UUID.
    pub fn default_starter_character_id(&self) -> Option<Uuid> {
        self.starter_character_ids().into_iter().min()
    }

    pub fn starter_character_ids(&self) -> Vec<Uuid> {
        self.db.character_def_v1().starter().filter(true).map(|c| c.id).collect()
    }

    pub fn find_piece(&self, piece_id: Uuid) -> ServiceResult<PieceDef> {
        self.db
            .piece_def_v1()
            .id()
            .find(piece_id)
            .ok_or_else(|| CharacterError::unknown_piece(piece_id))
    }

    /// The rarity weight of the character owning `piece_id`, used to scale
    /// lootbox drop weights.
    pub fn piece_rarity_weight(&self, piece_id: Uuid) -> ServiceResult<u32> {
        let piece = self.find_piece(piece_id)?;
        let character = self
            .db
            .character_def_v1()
            .id()
            .find(piece.character_id)
            .ok_or_else(|| CharacterError::unknown_character_for_piece(piece.character_id))?;
        Ok(character.rarity_weight)
    }

    /// Every unique piece required to unlock `character_id`.
    pub fn piece_ids_of_character(&self, character_id: Uuid) -> Vec<Uuid> {
        self.db
            .piece_def_v1()
            .character_id()
            .filter(character_id)
            .map(|piece| piece.id)
            .collect()
    }

    pub fn piece_exists(&self, piece_id: Uuid) -> bool {
        self.db.piece_def_v1().id().find(piece_id).is_some()
    }
}
