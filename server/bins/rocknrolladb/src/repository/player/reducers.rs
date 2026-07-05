//! Player reducers: parameter/caller validation plus one service delegation.

use crate::error::ServiceResult;
use crate::repository::player::services::PlayerReducerContext;
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer]
pub fn select_character(ctx: &ReducerContext, character_id: Uuid) -> ServiceResult<()> {
    ctx.player_services()
        .select_character(ctx.sender(), character_id)
}
