//! Character reducers: parameter/caller validation plus one service delegation.

use crate::{
    error::{ServiceError, ServiceResult},
    extend::validate::{validate_f32_range, validate_positive_u32, validate_required_str},
    repository::{
        access,
        character::{
            ART_KIND_BODY, ART_KIND_SILHOUETTE, CharacterDef, PieceDef,
            services::CharacterServicesTrait,
            types::{CharacterArtImportV1, FaceImportV1},
        },
    },
};
use rocknrolla_level::validate_svg_asset;
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer(name = "import_character_v1")]
#[allow(clippy::too_many_arguments)]
pub fn import_character_v1(
    ctx: &ReducerContext,
    id: Uuid,
    name: String,
    style: String,
    rarity_weight: u32,
    density: f32,
    jump_speed: f32,
    flight_time_ms: u32,
    buoyancy: f32,
    fire_resistance: f32,
    starter: bool,
) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&name, "name", 128)?;
    validate_required_str(&style, "style", 128)?;
    validate_positive_u32(rarity_weight, "rarity_weight")?;
    validate_f32_range(density, "density", f32::MIN_POSITIVE, 1.0)?;
    validate_f32_range(jump_speed, "jump_speed", 0.0, 100.0)?;
    validate_f32_range(buoyancy, "buoyancy", 0.0, 10.0)?;
    validate_f32_range(fire_resistance, "fire_resistance", 0.0, 1.0)?;
    ctx.character_services().import_character(CharacterDef {
        id,
        name,
        style,
        rarity_weight,
        density,
        jump_speed,
        flight_time_ms,
        buoyancy,
        fire_resistance,
        starter,
    })
}

#[spacetimedb::reducer(name = "import_piece_v1")]
pub fn import_piece_v1(ctx: &ReducerContext, id: Uuid, name: String, character_id: Uuid) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&name, "name", 128)?;
    ctx.character_services().import_piece(PieceDef { id, name, character_id })
}

#[spacetimedb::reducer(name = "import_character_art_v1")]
pub fn import_character_art_v1(ctx: &ReducerContext, art: CharacterArtImportV1) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    if art.kind != ART_KIND_BODY && art.kind != ART_KIND_SILHOUETTE {
        return Err(ServiceError::validation(format!(
            "character art kind must be '{ART_KIND_BODY}' or '{ART_KIND_SILHOUETTE}', got '{}'",
            art.kind
        )));
    }
    validate_svg_asset(
        "character art",
        &format!("{}/{}", art.character_id, art.kind),
        art.width_px,
        art.height_px,
        &art.content_hash,
        &art.data,
    )?;
    ctx.character_services().import_character_art(art)
}

#[spacetimedb::reducer(name = "import_face_v1")]
pub fn import_face_v1(ctx: &ReducerContext, face: FaceImportV1) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&face.slug, "slug", 64)?;
    validate_svg_asset(
        "face",
        &face.slug,
        face.width_px,
        face.height_px,
        &face.content_hash,
        &face.data,
    )?;
    ctx.character_services().import_face(face)
}
