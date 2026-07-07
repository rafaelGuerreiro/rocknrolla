//! Public read models for character content.

use crate::{
    extend::stdb::all_uuids,
    repository::character::{character_def_v1__view, piece_def_v1__view},
};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct CharacterViewV1 {
    pub id: Uuid,
    pub name: String,
    pub style: String,
    pub density: f32,
    pub jump_speed: f32,
    pub flight_time_ms: u32,
    pub buoyancy: f32,
    pub fire_resistance: f32,
}

#[derive(SpacetimeType)]
pub struct PieceViewV1 {
    pub id: Uuid,
    pub name: String,
    pub character_id: Uuid,
}

#[view(accessor = vw_character_v1, name = "vw_character_v1", public)]
pub fn vw_character_v1(ctx: &AnonymousViewContext) -> Vec<CharacterViewV1> {
    ctx.db
        .character_def_v1()
        .starter()
        .filter(false..=true)
        .map(|c| CharacterViewV1 {
            id: c.id,
            name: c.name,
            style: c.style,
            density: c.density,
            jump_speed: c.jump_speed,
            flight_time_ms: c.flight_time_ms,
            buoyancy: c.buoyancy,
            fire_resistance: c.fire_resistance,
        })
        .collect()
}

#[view(accessor = vw_piece_v1, name = "vw_piece_v1", public)]
pub fn vw_piece_v1(ctx: &AnonymousViewContext) -> Vec<PieceViewV1> {
    ctx.db
        .piece_def_v1()
        .character_id()
        .filter(all_uuids())
        .map(|p| PieceViewV1 {
            id: p.id,
            name: p.name,
            character_id: p.character_id,
        })
        .collect()
}
