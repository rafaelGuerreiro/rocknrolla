//! Public read models for level content. Only active levels are exposed and
//! internal fields stay private.

use crate::repository::{
    level::{level_placement_v1__view, level_v1__view},
    progression::player_selected_level_v1__view,
};
use rocknrolla_geometry::{Vec2, Vec3};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct LevelViewV1 {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub spawn: Vec2,
    pub finish: Vec2,
}

#[derive(SpacetimeType)]
pub struct LevelPlacementViewV1 {
    pub level_id: Uuid,
    pub component_id: Uuid,
    pub position: Vec3,
    pub flip_x: bool,
    pub scale: f32,
    pub order: u32,
}

#[view(accessor = vw_level_v1, name = "vw_level_v1", public)]
pub fn vw_level_v1(ctx: &AnonymousViewContext) -> Vec<LevelViewV1> {
    ctx.db
        .level_v1()
        .active()
        .filter(true)
        .map(|level| LevelViewV1 {
            id: level.id,
            slug: level.slug,
            name: level.name,
            spawn: level.spawn,
            finish: level.finish,
        })
        .collect()
}

/// Placements for the caller's currently selected level only; never every
/// active level in the game (see `player_selected_level_v1`).
#[view(accessor = vw_level_placement_v1, name = "vw_level_placement_v1", public)]
pub fn vw_level_placement_v1(ctx: &ViewContext) -> Vec<LevelPlacementViewV1> {
    let Some(selected) = ctx.db.player_selected_level_v1().owner().find(ctx.sender()) else {
        return Vec::new();
    };
    ctx.db
        .level_placement_v1()
        .level_id()
        .filter(selected.level_id)
        .map(|placement| LevelPlacementViewV1 {
            level_id: placement.level_id,
            component_id: placement.component_id,
            position: placement.position,
            flip_x: placement.flip_x,
            scale: placement.scale,
            order: placement.order,
        })
        .collect()
}
