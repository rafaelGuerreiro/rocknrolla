//! Progression reducers: parameter/caller validation plus one service delegation.

use crate::{error::ServiceResult, repository::progression::services::ProgressionReducerContext};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer(name = "complete_level_v1")]
pub fn complete_level_v1(ctx: &ReducerContext, level_id: Uuid) -> ServiceResult<()> {
    ctx.progression_services().complete_level(ctx.sender(), level_id)
}

#[spacetimedb::reducer(name = "select_level_v1")]
pub fn select_level_v1(ctx: &ReducerContext, level_id: Uuid) -> ServiceResult<()> {
    ctx.progression_services().select_level(ctx.sender(), level_id)
}
