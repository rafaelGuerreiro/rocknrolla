//! Shared level encoding for the RocknRolla module, admin shell, and tests.
//!
//! - [`catalog`]: semantic tile IDs and gameplay-layer constants.
//! - [`rle`]: the `rle-v1` codec, its identifier, and its error type.
//! - [`hash`]: deterministic layer content hashing.
//! - [`layer`]: [`LayerFacts`] and whole-layer-set validation.

pub mod catalog;
pub mod hash;
pub mod layer;
pub mod rle;

pub use catalog::{GAMEPLAY_CELL_SIZE, GAMEPLAY_Z, tile};
pub use hash::content_hash;
pub use layer::{LayerFacts, validate_layers};
pub use rle::{CodecError, ENCODING_RLE_V1, rle_decode, rle_encode};
