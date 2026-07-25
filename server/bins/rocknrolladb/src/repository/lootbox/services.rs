//! Lootbox repository services: import, granting, and the opening workflow.

use crate::{
    error::ServiceResult,
    extend::{access::ensure_owner, make_service::make_service, stdb::UuidGen},
    repository::{
        character::services::CharacterServicesTrait,
        lootbox::{
            LootboxDef, LootboxDrop, PlayerLootbox, errors::LootboxError, lootbox_def_v1, lootbox_drop_v1, player_lootbox_v1,
            types::DropImportV1,
        },
        player::services::PlayerServicesTrait,
    },
};
use spacetimedb::{Identity, Table, Uuid, rand::Rng};

make_service!(lootbox_services);

impl LootboxServicesImpl<'_> {
    /// Overwrite one authored lootbox definition and its drop table,
    /// verifying every referenced piece and weight.
    pub fn import_lootbox(&self, id: Uuid, name: String, drops: Vec<DropImportV1>) -> ServiceResult<()> {
        if drops.is_empty() {
            return Err(LootboxError::no_drops());
        }
        for drop in &drops {
            if drop.weight == 0 {
                return Err(LootboxError::zero_weight(drop.piece_id));
            }
            if !self.character_services().piece_exists(drop.piece_id) {
                return Err(LootboxError::unknown_drop_piece(drop.piece_id));
            }
        }
        self.db.lootbox_def_v1().id().insert_or_update(LootboxDef { id, name });
        self.db.lootbox_drop_v1().lootbox_id().delete(id);
        for drop in drops {
            self.db.lootbox_drop_v1().insert(LootboxDrop {
                id: self.ctx.generate_uuid()?,
                lootbox_id: id,
                piece_id: drop.piece_id,
                weight: drop.weight,
            });
        }
        Ok(())
    }

    pub fn lootbox_exists(&self, lootbox_id: Uuid) -> bool {
        self.db.lootbox_def_v1().id().find(lootbox_id).is_some()
    }

    /// Grant `owner` one unopened lootbox of the given definition.
    pub fn grant_lootbox(&self, owner: Identity, lootbox_id: Uuid) -> ServiceResult<()> {
        self.db.player_lootbox_v1().insert(PlayerLootbox {
            id: self.ctx.generate_uuid()?,
            owner,
            lootbox_id,
            granted_at: self.timestamp,
            opened: false,
            awarded_piece_id: None,
        });
        Ok(())
    }

    /// Open an unopened lootbox `sender` owns. Picks one unique piece using
    /// the reducer RNG and weights of `drop.weight * rarity_weight` of the
    /// piece's character, increments the owner's piece count (duplicates
    /// allowed), persists the award on the lootbox row, and unlocks the
    /// piece's character once every required unique piece is owned.
    pub fn open_lootbox(&self, sender: Identity, player_lootbox_id: Uuid) -> ServiceResult<()> {
        let lootbox = self
            .db
            .player_lootbox_v1()
            .id()
            .find(player_lootbox_id)
            .ok_or_else(LootboxError::unknown_lootbox)?;
        ensure_owner(sender, lootbox.owner)?;
        if lootbox.opened {
            return Err(LootboxError::already_opened());
        }

        let mut weighted: Vec<(Uuid, u64)> = Vec::new();
        for drop in self.db.lootbox_drop_v1().lootbox_id().filter(lootbox.lootbox_id) {
            let rarity = self.character_services().piece_rarity_weight(drop.piece_id)?;
            weighted.push((drop.piece_id, drop.weight as u64 * rarity as u64));
        }
        let total: u64 = weighted.iter().map(|(_, w)| w).sum();
        if total == 0 {
            return Err(LootboxError::no_weighted_drops(lootbox.lootbox_id));
        }
        let roll = self.rng().gen_range(0..total);
        let piece_id = pick_weighted(&weighted, roll).ok_or_else(LootboxError::weighted_pick_failed)?;

        let count = self.player_services().grant_piece(sender, piece_id)?;
        log::info!("player {sender} got piece {piece_id} (count {count})");

        self.db.player_lootbox_v1().id().update(PlayerLootbox {
            opened: true,
            awarded_piece_id: Some(piece_id),
            ..lootbox
        });

        let piece = self.character_services().find_piece(piece_id)?;
        let required = self.character_services().piece_ids_of_character(piece.character_id);
        self.player_services()
            .unlock_character_when_owned(sender, piece.character_id, &required)?;
        Ok(())
    }
}

/// Pick one entry from `(id, weight)` pairs given `roll in 0..total_weight`.
/// Zero-weight entries can never be picked.
pub fn pick_weighted(weighted: &[(Uuid, u64)], roll: u64) -> Option<Uuid> {
    let mut cursor = roll;
    for (id, weight) in weighted {
        if cursor < *weight {
            return Some(*id);
        }
        cursor -= weight;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_pick_covers_every_band_and_rejects_overflow() {
        let stone_chip = Uuid::from_u128(1);
        let paper_scrap_a = Uuid::from_u128(2);
        let paper_scrap_b = Uuid::from_u128(3);
        let weighted = vec![(stone_chip, 3u64), (paper_scrap_a, 0u64), (paper_scrap_b, 2u64)];
        assert_eq!(pick_weighted(&weighted, 0), Some(stone_chip));
        assert_eq!(pick_weighted(&weighted, 2), Some(stone_chip));
        assert_eq!(pick_weighted(&weighted, 3), Some(paper_scrap_b));
        assert_eq!(pick_weighted(&weighted, 4), Some(paper_scrap_b));
        assert_eq!(pick_weighted(&weighted, 5), None);
    }
}
