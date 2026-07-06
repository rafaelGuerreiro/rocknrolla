//! Progression reducers: parameter/caller validation plus one service delegation.

use crate::{error::ServiceResult, repository::progression::services::ProgressionReducerContext};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer]
pub fn complete_level(ctx: &ReducerContext, level_id: Uuid) -> ServiceResult<()> {
    ctx.progression_services().complete_level(ctx.sender(), level_id)
}
