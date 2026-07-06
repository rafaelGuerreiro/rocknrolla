//! Shared level encoding for the RocknRolla module, admin shell, and tests.
//!
//! - [`catalog`]: semantic tile IDs and gameplay-layer constants.
//! - [`hash`]: deterministic layer content hashing.
//! - [`layer`]: [`LayerFacts`] and whole-layer-set validation for `svg-v1`
//!   scene layers (standalone SVG documents with hidden collider markers).

pub mod catalog;
pub mod hash;
pub mod layer;

pub use catalog::{GAMEPLAY_CELL_SIZE, GAMEPLAY_Z, tile};
pub use hash::content_hash;
pub use layer::{ENCODING_SVG_V1, LayerFacts, MAX_SVG_BYTES, validate_layers};
