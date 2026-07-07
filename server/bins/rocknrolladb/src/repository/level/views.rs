//! Public read models for level content. Only active levels are exposed and
//! internal fields stay private.

use crate::repository::level::{level_layer_v1__view, level_v1__view};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct LevelViewV1 {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

#[derive(SpacetimeType)]
pub struct LevelLayerViewV1 {
    pub level_id: Uuid,
    pub z: u8,
    pub width_px: u32,
    pub height_px: u32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[view(accessor = vw_level_v1, public)]
pub fn vw_level_v1(ctx: &AnonymousViewContext) -> Vec<LevelViewV1> {
    ctx.db
        .level_v1()
        .active()
        .filter(true)
        .map(|level| LevelViewV1 {
            id: level.id,
            slug: level.slug,
            name: level.name,
        })
        .collect()
}

#[view(accessor = vw_level_layer_v1, public)]
pub fn vw_level_layer_v1(ctx: &AnonymousViewContext) -> Vec<LevelLayerViewV1> {
    ctx.db
        .level_v1()
        .active()
        .filter(true)
        .flat_map(|level| ctx.db.level_layer_v1().level_id().filter(level.id))
        .map(|layer| LevelLayerViewV1 {
            level_id: layer.level_id,
            z: layer.z,
            width_px: layer.width_px,
            height_px: layer.height_px,
            parallax_x: layer.parallax_x,
            parallax_y: layer.parallax_y,
            encoding: layer.encoding,
            content_hash: layer.content_hash,
            data: layer.data,
        })
        .collect()
}
