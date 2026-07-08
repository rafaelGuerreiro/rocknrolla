//! Domain repositories and cross-repository lifecycle coordination.

use crate::{
    error::ServiceResult,
    repository::{
        character::services::CharacterServicesTrait, level::services::LevelServicesTrait,
        player::services::PlayerServicesTrait, progression::services::ProgressionServicesTrait,
    },
};
use spacetimedb::ReducerContext;

pub mod access;
pub mod backdrop;
pub mod character;
pub mod component;
pub mod level;
pub mod lootbox;
pub mod player;
pub mod progression;

pub fn init(ctx: &ReducerContext) {
    access::record_module_owner(ctx, ctx.sender());
}

/// Bootstrap the connecting player: profile row, starter characters, and
/// every active starting level. Idempotent so content imported after a
/// player's first connection is picked up.
pub fn identity_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    let sender = ctx.sender();
    let starters = ctx.character_services().starter_character_ids();
    let default_character = ctx.character_services().default_starter_character_id();
    ctx.player_services().ensure_player(sender, &starters, default_character)?;
    let starting_levels = ctx.level_services().active_starting_level_ids();
    ctx.progression_services().enable_levels_if_absent(sender, &starting_levels)?;
    Ok(())
}
