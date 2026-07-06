//! Public read models for character content.

use crate::{
    extend::stdb::all_uuids,
    repository::character::{character_def__view, piece_def__view},
};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct CharacterView {
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
pub struct PieceView {
    pub id: Uuid,
    pub name: String,
    pub character_id: Uuid,
}

#[view(accessor = vw_character, public)]
pub fn vw_character(ctx: &AnonymousViewContext) -> Vec<CharacterView> {
    ctx.db
        .character_def()
        .starter()
        .filter(false..=true)
        .map(|c| CharacterView {
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

#[view(accessor = vw_piece, public)]
pub fn vw_piece(ctx: &AnonymousViewContext) -> Vec<PieceView> {
    ctx.db
        .piece_def()
        .character_id()
        .filter(all_uuids())
        .map(|p| PieceView {
            id: p.id,
            name: p.name,
            character_id: p.character_id,
        })
        .collect()
}
