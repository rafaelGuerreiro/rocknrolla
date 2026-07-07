//! Shared level encoding for the RocknRolla module, admin shell, and tests.
//!
//! - [`catalog`]: semantic tile IDs and gameplay constants.
//! - [`hash`]: deterministic content hashing.
//! - [`component`]: [`ComponentFacts`] and validation for library SVG
//!   components (standalone SVG documents with hidden collider markers).
//! - [`placement`]: [`PlacementFacts`] and whole-level geometry validation.

pub mod catalog;
pub mod component;
pub mod hash;
pub mod placement;

pub use catalog::{GAMEPLAY_CELL_SIZE, tile};
pub use component::{ComponentFacts, MAX_SVG_BYTES, validate_component};
pub use hash::content_hash;
pub use placement::{GAMEPLAY_PLANE_Z, MAX_PLACEMENT_SCALE, PlacementFacts, validate_level_geometry};
pub use rocknrolla_geometry::{Vec2, Vec3};
