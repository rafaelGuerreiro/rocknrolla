//! Reducer argument types for the level domain.

use rocknrolla_geometry::Vec3;
use spacetimedb::SpacetimeType;

#[derive(SpacetimeType)]
pub struct PlacementImportV1 {
    /// The component's authored identity; resolved to its UUID at import.
    pub component_slug: String,
    pub position: Vec3,
    pub flip_x: bool,
    pub scale: f32,
}
