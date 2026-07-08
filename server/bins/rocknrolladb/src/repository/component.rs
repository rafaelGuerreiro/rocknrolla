//! Component library: reusable SVG fragments (art + collider markers)
//! that levels compose via placements.

use spacetimedb::Uuid;

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[spacetimedb::table(accessor = component_v1, name = "component_v1", private)]
pub struct Component {
    #[primary_key]
    pub id: Uuid,
    /// Authored identity: the component's filename in `content/components/`.
    #[unique]
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}
