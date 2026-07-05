//! Public read models for the caller's own progression.

use crate::repository::progression::{
    PlayerCompletedLevel, PlayerEnabledLevel, player_completed_level__view,
    player_enabled_level__view,
};
use spacetimedb::{SpacetimeType, Timestamp, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct MyEnabledLevelView {
    pub level_id: Uuid,
}

#[derive(SpacetimeType)]
pub struct MyCompletedLevelView {
    pub level_id: Uuid,
    pub completed_at: Timestamp,
}

/// The caller's enabled levels; never another player's.
#[view(accessor = vw_my_enabled_level, public)]
pub fn vw_my_enabled_level(ctx: &ViewContext) -> Vec<MyEnabledLevelView> {
    ctx.db
        .player_enabled_level()
        .by_owner_level()
        .filter(ctx.sender())
        .map(|PlayerEnabledLevel { level_id, .. }| MyEnabledLevelView { level_id })
        .collect()
}

/// The caller's completed levels; never another player's.
#[view(accessor = vw_my_completed_level, public)]
pub fn vw_my_completed_level(ctx: &ViewContext) -> Vec<MyCompletedLevelView> {
    ctx.db
        .player_completed_level()
        .by_owner_level()
        .filter(ctx.sender())
        .map(
            |PlayerCompletedLevel {
                 level_id,
                 completed_at,
                 ..
             }| MyCompletedLevelView {
                level_id,
                completed_at,
            },
        )
        .collect()
}
