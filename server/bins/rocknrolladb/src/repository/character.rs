//! Character content: playable character definitions, their pieces, their
//! art (body + derived silhouette), and the shared face expression set.

use spacetimedb::Uuid;

pub mod errors;
pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

/// Art kinds stored per character; the silhouette is derived from the body
/// by the importer, never authored.
pub const ART_KIND_BODY: &str = "body";
pub const ART_KIND_SILHOUETTE: &str = "silhouette";

#[spacetimedb::table(accessor = character_def_v1, name = "character_def_v1", private)]
pub struct CharacterDef {
    #[primary_key]
    pub id: Uuid,
    pub name: String,
    /// Authored identity: the character's art filename in
    /// `content/characters/`. Import-time link only; never exposed to views.
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

#[spacetimedb::table(accessor = piece_def_v1, name = "piece_def_v1", private)]
pub struct PieceDef {
    #[primary_key]
    pub id: Uuid,
    pub name: String,
    #[index(btree)]
    pub character_id: Uuid,
}

#[spacetimedb::table(accessor = character_art_v1, name = "character_art_v1", private)]
pub struct CharacterArt {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub character_id: Uuid,
    /// [`ART_KIND_BODY`] or [`ART_KIND_SILHOUETTE`]; one row per kind per
    /// character, kept stable across overwrites by (character, kind).
    pub kind: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[spacetimedb::table(accessor = face_v1, name = "face_v1", private)]
pub struct Face {
    #[primary_key]
    pub id: Uuid,
    /// Authored identity: the expression's filename in `content/faces/`.
    /// One row per slug, enforced by the import upsert (btree, not unique,
    /// so views can range-scan the whole set).
    #[index(btree)]
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}
