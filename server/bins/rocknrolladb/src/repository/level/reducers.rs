//! Level reducers: parameter/caller validation plus one service delegation.

use crate::{
    error::ServiceResult,
    extend::validate::validate_required_str,
    repository::{
        access,
        level::{
            services::{LevelImport, LevelReducerContext},
            types::LayerImport,
        },
    },
};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn import_level(
    ctx: &ReducerContext,
    id: Uuid,
    slug: String,
    name: String,
    is_starting: bool,
    active: bool,
    reward_lootbox_id: Option<Uuid>,
    successors: Vec<Uuid>,
    layers: Vec<LayerImport>,
) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&slug, "slug", 64)?;
    validate_required_str(&name, "name", 128)?;
    ctx.level_services().import_level(LevelImport {
        id,
        slug,
        name,
        is_starting,
        active,
        reward_lootbox_id,
        successors,
        layers,
    })
}
