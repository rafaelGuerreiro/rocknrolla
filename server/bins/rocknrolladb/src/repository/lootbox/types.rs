//! Reducer argument types for the lootbox domain.

use spacetimedb::{SpacetimeType, Uuid};

#[derive(SpacetimeType)]
pub struct DropImport {
    pub piece_id: Uuid,
    pub weight: u32,
}
