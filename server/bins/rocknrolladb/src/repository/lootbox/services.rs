//! Lootbox repository services: import, granting, and the opening workflow.

use crate::error::{ServiceError, ServiceResult};
use crate::extend::access::ensure_owner;
use crate::extend::stdb::UuidGen;
use crate::repository::character::services::CharacterReducerContext;
use crate::repository::lootbox::types::DropImport;
use crate::repository::lootbox::{
    LootboxDef, LootboxDrop, PlayerLootbox, lootbox_def, lootbox_drop, player_lootbox,
};
use crate::repository::player::services::PlayerReducerContext;
use spacetimedb::rand::Rng;
use spacetimedb::{Identity, ReducerContext, Table, Uuid};
use std::ops::Deref;

pub trait LootboxReducerContext {
    fn lootbox_services(&self) -> LootboxServices<'_>;
}

impl LootboxReducerContext for ReducerContext {
    fn lootbox_services(&self) -> LootboxServices<'_> {
        LootboxServices { ctx: self }
    }
}

pub struct LootboxServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for LootboxServices<'_> {
    type Target = ReducerContext;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl LootboxServices<'_> {
    /// Overwrite one authored lootbox definition and its drop table,
    /// verifying every referenced piece and weight.
    pub fn import_lootbox(
        &self,
        id: Uuid,
        name: String,
        drops: Vec<DropImport>,
    ) -> ServiceResult<()> {
        if drops.is_empty() {
            return Err(ServiceError::validation(
                "lootbox must configure at least one drop",
            ));
        }
        for drop in &drops {
            if drop.weight == 0 {
                return Err(ServiceError::validation(format!(
                    "drop '{}' must have a positive weight",
                    drop.piece_id
                )));
            }
            if !self.character_services().piece_exists(drop.piece_id) {
                return Err(ServiceError::not_found(format!(
                    "drop references unknown piece '{}'",
                    drop.piece_id
                )));
            }
        }
        self.db
            .lootbox_def()
            .id()
            .insert_or_update(LootboxDef { id, name });
        self.db.lootbox_drop().lootbox_id().delete(id);
        for drop in drops {
            self.db.lootbox_drop().insert(LootboxDrop {
                id: self.ctx.generate_uuid()?,
                lootbox_id: id,
                piece_id: drop.piece_id,
                weight: drop.weight,
            });
        }
        Ok(())
    }

    pub fn lootbox_exists(&self, lootbox_id: Uuid) -> bool {
        self.db.lootbox_def().id().find(lootbox_id).is_some()
    }

    /// Grant `owner` one unopened lootbox of the given definition.
    pub fn grant_lootbox(&self, owner: Identity, lootbox_id: Uuid) -> ServiceResult<()> {
        self.db.player_lootbox().insert(PlayerLootbox {
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
            .player_lootbox()
            .id()
            .find(player_lootbox_id)
            .ok_or_else(|| ServiceError::not_found("unknown lootbox"))?;
        ensure_owner(sender, lootbox.owner)?;
        if lootbox.opened {
            return Err(ServiceError::conflict("lootbox already opened"));
        }

        let mut weighted: Vec<(Uuid, u64)> = Vec::new();
        for drop in self
            .db
            .lootbox_drop()
            .lootbox_id()
            .filter(lootbox.lootbox_id)
        {
            let rarity = self
                .character_services()
                .piece_rarity_weight(drop.piece_id)?;
            weighted.push((drop.piece_id, drop.weight as u64 * rarity as u64));
        }
        let total: u64 = weighted.iter().map(|(_, w)| w).sum();
        if total == 0 {
            return Err(ServiceError::validation(format!(
                "lootbox '{}' has no weighted drops",
                lootbox.lootbox_id
            )));
        }
        let roll = self.rng().gen_range(0..total);
        let piece_id = pick_weighted(&weighted, roll)
            .ok_or_else(|| ServiceError::internal("weighted pick failed"))?;

        let count = self.player_services().grant_piece(sender, piece_id)?;
        log::info!("player {sender} got piece {piece_id} (count {count})");

        self.db.player_lootbox().id().update(PlayerLootbox {
            opened: true,
            awarded_piece_id: Some(piece_id),
            ..lootbox
        });

        let piece = self.character_services().find_piece(piece_id)?;
        let required = self
            .character_services()
            .piece_ids_of_character(piece.character_id);
        self.player_services().unlock_character_when_owned(
            sender,
            piece.character_id,
            &required,
        )?;
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
        let weighted = vec![
            (stone_chip, 3u64),
            (paper_scrap_a, 0u64),
            (paper_scrap_b, 2u64),
        ];
        assert_eq!(pick_weighted(&weighted, 0), Some(stone_chip));
        assert_eq!(pick_weighted(&weighted, 2), Some(stone_chip));
        assert_eq!(pick_weighted(&weighted, 3), Some(paper_scrap_b));
        assert_eq!(pick_weighted(&weighted, 4), Some(paper_scrap_b));
        assert_eq!(pick_weighted(&weighted, 5), None);
    }
}
