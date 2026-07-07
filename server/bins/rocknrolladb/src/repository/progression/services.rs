//! Progression services: level enabling and the first-completion workflow.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::{
        level::services::LevelReducerContext,
        lootbox::services::LootboxReducerContext,
        progression::{
            PlayerCompletedLevel, PlayerEnabledLevel, PlayerSelectedLevel, errors::ProgressionError, player_completed_level_v1,
            player_enabled_level_v1, player_selected_level_v1,
        },
    },
};
use spacetimedb::{Identity, Table, Uuid};

make_service!(ProgressionReducerContext, progression_services, ProgressionServices);

impl ProgressionServices<'_> {
    /// Idempotently enable each of the given levels for `owner`.
    pub fn enable_levels_if_absent(&self, owner: Identity, level_ids: &[Uuid]) -> ServiceResult<()> {
        let enabled = self.enabled_level_ids(owner);
        for level_id in successor_inserts(level_ids, &enabled) {
            self.db.player_enabled_level_v1().insert(PlayerEnabledLevel {
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
            .player_enabled_level_v1()
            .by_owner_level()
            .filter((owner, level_id))
            .next()
            .is_none()
        {
            return Err(ProgressionError::level_not_enabled(owner, &level.slug));
        }
        let already_completed = self
            .db
            .player_completed_level_v1()
            .by_owner_level()
            .filter((owner, level_id))
            .next()
            .is_some();
        if already_completed {
            return Ok(());
        }

        self.db.player_completed_level_v1().insert(PlayerCompletedLevel {
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
            self.lootbox_services().grant_lootbox(owner, reward_lootbox_id)?;
        }
        Ok(())
    }

    /// Select one of the caller's enabled levels to play, replacing any
    /// previous selection. Gates what `vw_level_placement_v1` exposes.
    pub fn select_level(&self, owner: Identity, level_id: Uuid) -> ServiceResult<()> {
        let level = self.level_services().find_active_level(level_id)?;
        if self
            .db
            .player_enabled_level_v1()
            .by_owner_level()
            .filter((owner, level_id))
            .next()
            .is_none()
        {
            return Err(ProgressionError::level_not_enabled(owner, &level.slug));
        }
        self.db
            .player_selected_level_v1()
            .owner()
            .insert_or_update(PlayerSelectedLevel { owner, level_id });
        Ok(())
    }

    fn enabled_level_ids(&self, owner: Identity) -> Vec<Uuid> {
        self.db
            .player_enabled_level_v1()
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
