//! Player reducers: parameter/caller validation plus one service delegation.

use crate::{error::ServiceResult, repository::player::services::PlayerReducerContext};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer(name = "select_character_v1")]
pub fn select_character_v1(ctx: &ReducerContext, character_id: Uuid) -> ServiceResult<()> {
    ctx.player_services().select_character(ctx.sender(), character_id)
}
