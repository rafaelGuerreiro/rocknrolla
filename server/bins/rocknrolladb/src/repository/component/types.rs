//! Reducer argument types for the component domain.

use spacetimedb::SpacetimeType;

#[derive(SpacetimeType)]
pub struct ComponentImportV1 {
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}
