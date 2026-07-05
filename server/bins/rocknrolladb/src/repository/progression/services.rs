//! Progression services: level enabling and the first-completion workflow.

use crate::error::{ServiceError, ServiceResult};
use crate::extend::stdb::UuidGen;
use crate::repository::level::services::LevelReducerContext;
use crate::repository::lootbox::services::LootboxReducerContext;
use crate::repository::progression::{
    PlayerCompletedLevel, PlayerEnabledLevel, player_completed_level, player_enabled_level,
};
use spacetimedb::{Identity, ReducerContext, Table, Uuid};
use std::ops::Deref;

pub trait ProgressionReducerContext {
    fn progression_services(&self) -> ProgressionServices<'_>;
}

impl ProgressionReducerContext for ReducerContext {
    fn progression_services(&self) -> ProgressionServices<'_> {
        ProgressionServices { ctx: self }
    }
}

pub struct ProgressionServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for ProgressionServices<'_> {
    type Target = ReducerContext;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl ProgressionServices<'_> {
    /// Idempotently enable each of the given levels for `owner`.
    pub fn enable_levels_if_absent(
        &self,
        owner: Identity,
        level_ids: &[Uuid],
    ) -> ServiceResult<()> {
        let enabled = self.enabled_level_ids(owner);
        for level_id in successor_inserts(level_ids, &enabled) {
            self.db.player_enabled_level().insert(PlayerEnabledLevel {
                id: self.ctx.generate_uuid()?,
                owner,
                level_id,
            });
        }
        Ok(())
    }

    /// Client-reported completion of an enabled level. Idempotent: the first
    /// completion records it, enables configured successors, and grants
    /// exactly one unopened reward lootbox in the same transaction; replays
    /// are no-ops.
    pub fn complete_level(&self, owner: Identity, level_id: Uuid) -> ServiceResult<()> {
        let level = self.level_services().find_active_level(level_id)?;
        if self
            .db
            .player_enabled_level()
            .by_owner_level()
            .filter((owner, level_id))
            .next()
            .is_none()
        {
            return Err(ServiceError::forbidden(
                owner,
                format!("level '{}' is not enabled for this player", level.slug),
            ));
        }
        let already_completed = self
            .db
            .player_completed_level()
            .by_owner_level()
            .filter((owner, level_id))
            .next()
            .is_some();
        if already_completed {
            return Ok(());
        }

        self.db
            .player_completed_level()
            .insert(PlayerCompletedLevel {
                id: self.ctx.generate_uuid()?,
                owner,
                level_id,
                completed_at: self.timestamp,
            });

        let successors = self.level_services().successor_ids(level_id);
        self.enable_levels_if_absent(owner, &successors)?;

        if let Some(reward_lootbox_id) = level.reward_lootbox_id
            && self.lootbox_services().lootbox_exists(reward_lootbox_id)
        {
            self.lootbox_services()
                .grant_lootbox(owner, reward_lootbox_id)?;
        }
        Ok(())
    }

    fn enabled_level_ids(&self, owner: Identity) -> Vec<Uuid> {
        self.db
            .player_enabled_level()
            .by_owner_level()
            .filter(owner)
            .map(|row| row.level_id)
            .collect()
    }
}

/// Configured successors to insert: targets not already enabled, deduplicated.
pub fn successor_inserts(configured: &[Uuid], enabled: &[Uuid]) -> Vec<Uuid> {
    let mut inserts: Vec<Uuid> = Vec::new();
    for successor in configured {
        if !enabled.contains(successor) && !inserts.contains(successor) {
            inserts.push(*successor);
        }
    }
    inserts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[u128]) -> Vec<Uuid> {
        list.iter().map(|&n| Uuid::from_u128(n)).collect()
    }

    #[test]
    fn successors_unlock_only_missing_targets() {
        let riverside = Uuid::from_u128(2);
        let configured = ids(&[2, 1, 2]);
        let enabled = ids(&[1]);
        assert_eq!(successor_inserts(&configured, &enabled), vec![riverside]);
        assert!(successor_inserts(&configured, &ids(&[1, 2])).is_empty());
    }
}
