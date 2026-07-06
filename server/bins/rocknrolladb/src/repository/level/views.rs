//! Public read models for level content. Only active levels are exposed and
//! internal fields stay private.

use crate::repository::level::{level__view, level_layer__view};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct LevelView {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

#[derive(SpacetimeType)]
pub struct LevelLayerView {
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

#[view(accessor = vw_level, public)]
pub fn vw_level(ctx: &AnonymousViewContext) -> Vec<LevelView> {
    ctx.db
        .level()
        .active()
        .filter(true)
        .map(|level| LevelView {
            id: level.id,
            slug: level.slug,
            name: level.name,
        })
        .collect()
}

#[view(accessor = vw_level_layer, public)]
pub fn vw_level_layer(ctx: &AnonymousViewContext) -> Vec<LevelLayerView> {
    ctx.db
        .level()
        .active()
        .filter(true)
        .flat_map(|level| ctx.db.level_layer().level_id().filter(level.id))
        .map(|layer| LevelLayerView {
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
