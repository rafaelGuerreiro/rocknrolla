//! Reducer argument types for the level domain.

use spacetimedb::SpacetimeType;

#[derive(SpacetimeType)]
pub struct LayerImport {
    pub z: u8,
    pub width_px: u32,
    pub height_px: u32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}
