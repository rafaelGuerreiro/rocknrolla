//! Character repository services: content import and piece/character lookups.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::character::{
        CharacterArt, CharacterDef, Face, PieceDef, character_art_v1, character_def_v1,
        errors::CharacterError,
        face_v1, piece_def_v1,
        types::{CharacterArtImportV1, FaceImportV1},
    },
};
use spacetimedb::Uuid;

make_service!(character_services);

impl CharacterServicesImpl<'_> {
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

    /// Atomically overwrite one character's art blob for one kind, verifying
    /// the character. The row UUID is generated on first import and kept
    /// stable by (character, kind) across overwrites.
    pub fn import_character_art(&self, import: CharacterArtImportV1) -> ServiceResult<()> {
        if self.db.character_def_v1().id().find(import.character_id).is_none() {
            return Err(CharacterError::unknown_character_for_art(import.character_id));
        }
        let existing = self
            .db
            .character_art_v1()
            .character_id()
            .filter(import.character_id)
            .find(|row| row.kind == import.kind);
        let id = match existing {
            Some(row) => row.id,
            None => self.ctx.generate_uuid()?,
        };
        self.db.character_art_v1().id().insert_or_update(CharacterArt {
            id,
            character_id: import.character_id,
            kind: import.kind,
            width_px: import.width_px,
            height_px: import.height_px,
            content_hash: import.content_hash,
            data: import.data,
        });
        Ok(())
    }

    /// Atomically overwrite one face expression by slug. The slug is the
    /// authored identity (filename); the UUID is generated on first import
    /// and kept stable across overwrites.
    pub fn import_face(&self, import: FaceImportV1) -> ServiceResult<()> {
        let id = match self.db.face_v1().slug().filter(&import.slug).next() {
            Some(existing) => existing.id,
            None => self.ctx.generate_uuid()?,
        };
        self.db.face_v1().id().insert_or_update(Face {
            id,
            slug: import.slug,
            width_px: import.width_px,
            height_px: import.height_px,
            content_hash: import.content_hash,
            data: import.data,
        });
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
