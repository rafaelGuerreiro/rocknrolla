//! Level reducers: parameter/caller validation plus one service delegation.

use rocknrolla_geometry::Vec2;

use crate::{
    error::ServiceResult,
    extend::validate::validate_required_str,
    repository::{
        access,
        level::{
            services::{LevelImport, LevelServicesTrait},
            types::PlacementImportV1,
        },
    },
};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer(name = "import_level_v1")]
#[allow(clippy::too_many_arguments)]
pub fn import_level_v1(
    ctx: &ReducerContext,
    id: Uuid,
    slug: String,
    name: String,
    is_starting: bool,
    active: bool,
    reward_lootbox_id: Option<Uuid>,
    successors: Vec<Uuid>,
    backdrop_slug: String,
    spawn: Vec2,
    finish: Vec2,
    placements: Vec<PlacementImportV1>,
) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&slug, "slug", 64)?;
    validate_required_str(&name, "name", 128)?;
    validate_required_str(&backdrop_slug, "backdrop_slug", 64)?;
    ctx.level_services().import_level(LevelImport {
        id,
        slug,
        name,
        is_starting,
        active,
        reward_lootbox_id,
        successors,
        backdrop_slug,
        spawn,
        finish,
        placements,
    })
}
