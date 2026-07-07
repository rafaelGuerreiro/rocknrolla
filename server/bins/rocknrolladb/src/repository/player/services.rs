//! Player repository services: bootstrap, character selection, piece grants,
//! and character unlocks.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::player::{
        Player, PlayerPiece, PlayerUnlockedCharacter, errors::PlayerError, player_piece_v1, player_unlocked_character_v1,
        player_v1,
    },
};
use spacetimedb::{Identity, Table, Uuid};

make_service!(player_services);

impl PlayerServicesImpl<'_> {
    /// Idempotently ensure `owner` has a player row, every starter character,
    /// and a selected character. Safe to re-run so content imported after a
    /// player's first connection is picked up.
    pub fn ensure_player(
        &self,
        owner: Identity,
        starter_character_ids: &[Uuid],
        default_character_id: Option<Uuid>,
    ) -> ServiceResult<()> {
        for character_id in starter_character_ids {
            self.unlock_character_if_absent(owner, *character_id)?;
        }
        match self.db.player_v1().identity().find(owner) {
            None => {
                self.db.player_v1().insert(Player {
                    identity: owner,
                    selected_character_id: default_character_id,
                });
            },
            Some(existing) if existing.selected_character_id.is_none() => {
                self.db.player_v1().identity().update(Player {
                    selected_character_id: default_character_id,
                    ..existing
                });
            },
            Some(_) => {},
        }
        Ok(())
    }

    /// Select one of the caller's unlocked characters.
    pub fn select_character(&self, owner: Identity, character_id: Uuid) -> ServiceResult<()> {
        let player = self
            .db
            .player_v1()
            .identity()
            .find(owner)
            .ok_or_else(PlayerError::not_initialized)?;
        if !self.has_unlocked(owner, character_id) {
            return Err(PlayerError::character_not_unlocked(owner, character_id));
        }
        self.db.player_v1().identity().update(Player {
            selected_character_id: Some(character_id),
            ..player
        });
        Ok(())
    }

    /// Add one copy of `piece_id` to the owner's collection and return the
    /// new count. Duplicates are allowed.
    pub fn grant_piece(&self, owner: Identity, piece_id: Uuid) -> ServiceResult<u32> {
        match self.db.player_piece_v1().by_owner_piece().filter((owner, piece_id)).next() {
            Some(existing) => {
                let count = existing.count + 1;
                self.db.player_piece_v1().id().update(PlayerPiece { count, ..existing });
                Ok(count)
            },
            None => {
                self.db.player_piece_v1().insert(PlayerPiece {
                    id: self.ctx.generate_uuid()?,
                    owner,
                    piece_id,
                    count: 1,
                });
                Ok(1)
            },
        }
    }

    /// Unlock `character_id` once the owner holds every required piece.
    pub fn unlock_character_when_owned(
        &self,
        owner: Identity,
        character_id: Uuid,
        required_piece_ids: &[Uuid],
    ) -> ServiceResult<()> {
        let owned: Vec<Uuid> = self
            .db
            .player_piece_v1()
            .by_owner_piece()
            .filter(owner)
            .filter(|row| row.count > 0)
            .map(|row| row.piece_id)
            .collect();
        if unlocks_character(required_piece_ids, &owned) {
            self.unlock_character_if_absent(owner, character_id)?;
        }
        Ok(())
    }

    pub fn unlock_character_if_absent(&self, owner: Identity, character_id: Uuid) -> ServiceResult<()> {
        if !self.has_unlocked(owner, character_id) {
            self.db.player_unlocked_character_v1().insert(PlayerUnlockedCharacter {
                id: self.ctx.generate_uuid()?,
                owner,
                character_id,
            });
        }
        Ok(())
    }

    fn has_unlocked(&self, owner: Identity, character_id: Uuid) -> bool {
        self.db
            .player_unlocked_character_v1()
            .by_owner_character()
            .filter((owner, character_id))
            .next()
            .is_some()
    }
}

/// A character unlocks when the player owns at least one of every unique
/// piece assigned to it. Characters with no pieces never unlock via drops.
pub fn unlocks_character(required: &[Uuid], owned: &[Uuid]) -> bool {
    !required.is_empty() && required.iter().all(|piece| owned.contains(piece))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[u128]) -> Vec<Uuid> {
        list.iter().map(|&n| Uuid::from_u128(n)).collect()
    }

    #[test]
    fn duplicate_pieces_count_toward_a_single_ownership() {
        let required = ids(&[1, 2, 3]);
        let owned_partial = ids(&[1, 2]);
        assert!(!unlocks_character(&required, &owned_partial));
        let owned_all = ids(&[2, 1, 9, 3]);
        assert!(unlocks_character(&required, &owned_all));
    }

    #[test]
    fn characters_without_pieces_never_unlock_from_drops() {
        assert!(!unlocks_character(&[], &ids(&[7])));
    }
}
