//! Public read models for the caller's own progression.

use crate::repository::progression::{
    PlayerCompletedLevel, PlayerEnabledLevel, player_completed_level_v1__view, player_enabled_level_v1__view,
};
use spacetimedb::{SpacetimeType, Timestamp, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct MyEnabledLevelViewV1 {
    pub level_id: Uuid,
}

#[derive(SpacetimeType)]
pub struct MyCompletedLevelViewV1 {
    pub level_id: Uuid,
    pub completed_at: Timestamp,
}

/// The caller's enabled levels; never another player's.
#[view(accessor = vw_my_enabled_level_v1, public)]
pub fn vw_my_enabled_level_v1(ctx: &ViewContext) -> Vec<MyEnabledLevelViewV1> {
    ctx.db
        .player_enabled_level_v1()
        .by_owner_level()
        .filter(ctx.sender())
        .map(|PlayerEnabledLevel { level_id, .. }| MyEnabledLevelViewV1 { level_id })
        .collect()
}

/// The caller's completed levels; never another player's.
#[view(accessor = vw_my_completed_level_v1, public)]
pub fn vw_my_completed_level_v1(ctx: &ViewContext) -> Vec<MyCompletedLevelViewV1> {
    ctx.db
        .player_completed_level_v1()
        .by_owner_level()
        .filter(ctx.sender())
        .map(
            |PlayerCompletedLevel {
                 level_id, completed_at, ..
             }| MyCompletedLevelViewV1 { level_id, completed_at },
        )
        .collect()
}
