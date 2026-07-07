//! Player progression: enabled levels, completions, and first-run rewards.

use spacetimedb::{Identity, Timestamp, Uuid};

pub mod reducers;
pub mod services;
pub mod views;

#[spacetimedb::table(accessor = player_enabled_level_v1, name = "player_enabled_level_v1", private,
    index(accessor = by_owner_level, btree(columns = [owner, level_id])))]
pub struct PlayerEnabledLevel {
    #[primary_key]
    pub id: Uuid,
    pub owner: Identity,
    pub level_id: Uuid,
}

#[spacetimedb::table(accessor = player_completed_level_v1, name = "player_completed_level_v1", private,
    index(accessor = by_owner_level, btree(columns = [owner, level_id])))]
pub struct PlayerCompletedLevel {
    #[primary_key]
    pub id: Uuid,
    pub owner: Identity,
    pub level_id: Uuid,
    pub completed_at: Timestamp,
}
