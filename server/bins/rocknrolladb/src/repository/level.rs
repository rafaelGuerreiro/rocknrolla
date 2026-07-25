//! Level content: metadata, component placements, and successor edges.

use rocknrolla_geometry::{Vec2, Vec3};
use spacetimedb::Uuid;

pub mod errors;
pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[spacetimedb::table(accessor = level_v1, name = "level_v1", private)]
pub struct Level {
    #[primary_key]
    pub id: Uuid,
    /// Readable operator/UI slug; never a foreign key.
    #[unique]
    pub slug: String,
    pub name: String,
    pub is_starting: bool,
    #[index(btree)]
    pub active: bool,
    pub reward_lootbox_id: Option<Uuid>,
    /// The level's scenery theme; resolved from the authored backdrop slug
    /// at import.
    pub backdrop_id: Uuid,
    pub spawn: Vec2,
    pub finish: Vec2,
}

#[spacetimedb::table(accessor = level_placement_v1, name = "level_placement_v1", private)]
pub struct LevelPlacement {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub level_id: Uuid,
    pub component_id: Uuid,
    pub position: Vec3,
    pub flip_x: bool,
    pub scale: f32,
    /// Draw order within the level (import list order).
    pub order: u32,
}

#[spacetimedb::table(accessor = level_successor_v1, name = "level_successor_v1", private)]
pub struct LevelSuccessor {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub level_id: Uuid,
    pub successor_id: Uuid,
}
