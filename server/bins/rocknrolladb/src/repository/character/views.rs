//! Public read models for character content.

use crate::{
    extend::stdb::{all_slugs, all_uuids},
    repository::character::{character_art_v1__view, character_def_v1__view, face_v1__view, piece_def_v1__view},
};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct CharacterViewV1 {
    pub id: Uuid,
    pub name: String,
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

#[derive(SpacetimeType)]
pub struct CharacterArtViewV1 {
    pub id: Uuid,
    pub character_id: Uuid,
    /// "body" or "silhouette".
    pub kind: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[derive(SpacetimeType)]
pub struct FaceViewV1 {
    pub id: Uuid,
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
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

#[view(accessor = vw_character_art_v1, name = "vw_character_art_v1", public)]
pub fn vw_character_art_v1(ctx: &AnonymousViewContext) -> Vec<CharacterArtViewV1> {
    ctx.db
        .character_art_v1()
        .character_id()
        .filter(all_uuids())
        .map(|a| CharacterArtViewV1 {
            id: a.id,
            character_id: a.character_id,
            kind: a.kind,
            width_px: a.width_px,
            height_px: a.height_px,
            content_hash: a.content_hash,
            data: a.data,
        })
        .collect()
}

#[view(accessor = vw_face_v1, name = "vw_face_v1", public)]
pub fn vw_face_v1(ctx: &AnonymousViewContext) -> Vec<FaceViewV1> {
    ctx.db
        .face_v1()
        .slug()
        .filter(all_slugs())
        .map(|f| FaceViewV1 {
            id: f.id,
            slug: f.slug,
            width_px: f.width_px,
            height_px: f.height_px,
            content_hash: f.content_hash,
            data: f.data,
        })
        .collect()
}
