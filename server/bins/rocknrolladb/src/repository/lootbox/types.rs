//! Reducer argument types for the lootbox domain.

use spacetimedb::{SpacetimeType, Uuid};

#[derive(SpacetimeType)]
pub struct DropImportV1 {
    pub piece_id: Uuid,
    pub weight: u32,
}
