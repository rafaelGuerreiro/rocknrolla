//! Character content: playable character definitions and their pieces.

use spacetimedb::Uuid;

pub mod reducers;
pub mod services;
pub mod views;

#[spacetimedb::table(accessor = character_def, private)]
pub struct CharacterDef {
    #[primary_key]
    pub id: Uuid,
    pub name: String,
    /// Stable backend style/asset key resolved by the client to a sprite.
    pub style: String,
    pub rarity_weight: u32,
    pub density: f32,
    pub jump_speed: f32,
    pub flight_time_ms: u32,
    pub buoyancy: f32,
    pub fire_resistance: f32,
    #[index(btree)]
    pub starter: bool,
}

#[spacetimedb::table(accessor = piece_def, private)]
pub struct PieceDef {
    #[primary_key]
    pub id: Uuid,
    pub name: String,
    #[index(btree)]
    pub character_id: Uuid,
}
