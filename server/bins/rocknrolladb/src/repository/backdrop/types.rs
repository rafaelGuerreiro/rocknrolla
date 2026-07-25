//! Reducer argument and layer types for the backdrop domain.

use spacetimedb::SpacetimeType;

/// One backdrop layer's art: a standalone SVG document plus its natural
/// size and content hash. Shared by the table row, the import argument,
/// and the public view.
#[derive(SpacetimeType, Clone)]
pub struct BackdropLayerV1 {
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[derive(SpacetimeType)]
pub struct BackdropImportV1 {
    pub slug: String,
    pub sky: BackdropLayerV1,
    pub far: BackdropLayerV1,
    pub mid: BackdropLayerV1,
}
