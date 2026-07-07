//! Level content: metadata, layers, and successor edges.

use spacetimedb::Uuid;

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[spacetimedb::table(accessor = level_v1, private)]
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
}

#[spacetimedb::table(accessor = level_layer_v1, private)]
pub struct LevelLayer {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub level_id: Uuid,
    pub z: u8,
    pub width_px: u32,
    pub height_px: u32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    /// Layer codec identifier (`svg-v1`): `data` is one SVG document.
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[spacetimedb::table(accessor = level_successor_v1, private)]
pub struct LevelSuccessor {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub level_id: Uuid,
    pub successor_id: Uuid,
}
