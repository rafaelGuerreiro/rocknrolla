//! Character reducers: parameter/caller validation plus one service delegation.

use crate::error::ServiceResult;
use crate::extend::validate::{validate_f32_range, validate_positive_u32, validate_required_str};
use crate::repository::access;
use crate::repository::character::services::CharacterReducerContext;
use crate::repository::character::{CharacterDef, PieceDef};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn import_character(
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

#[spacetimedb::reducer]
pub fn import_piece(
    ctx: &ReducerContext,
    id: Uuid,
    name: String,
    character_id: Uuid,
) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&name, "name", 128)?;
    ctx.character_services().import_piece(PieceDef {
        id,
        name,
        character_id,
    })
}
