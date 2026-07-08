//! Reducer argument types for the character domain.

use spacetimedb::{SpacetimeType, Uuid};

#[derive(SpacetimeType)]
pub struct CharacterArtImportV1 {
    pub character_id: Uuid,
    /// "body" or "silhouette".
    pub kind: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[derive(SpacetimeType)]
pub struct FaceImportV1 {
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}
