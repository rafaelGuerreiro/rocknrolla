use rocknrolla_level::{LayerFacts, validate_layers};
use spacetimedb::rand::Rng;
use spacetimedb::{
    Filter, Identity, ReducerContext, SpacetimeType, Table, Timestamp, client_visibility_filter,
};

mod progression;

// ---------------------------------------------------------------------------
// Content tables (public, imported by the owner CLI)
// ---------------------------------------------------------------------------

#[spacetimedb::table(accessor = level, public)]
pub struct Level {
    #[primary_key]
    pub id: String,
    pub name: String,
    pub is_starting: bool,
    pub active: bool,
    pub reward_lootbox_id: String,
}

#[spacetimedb::table(accessor = level_layer, public)]
pub struct LevelLayer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub level_id: String,
    pub z: u8,
    pub width: u16,
    pub height: u16,
    pub cell_width: u16,
    pub cell_height: u16,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[spacetimedb::table(accessor = level_successor, public)]
pub struct LevelSuccessor {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub level_id: String,
    pub successor_id: String,
}

#[spacetimedb::table(accessor = character_def, public)]
pub struct CharacterDef {
    #[primary_key]
    pub id: String,
    pub name: String,
    /// Visual style key; a CSS hex color for procedural rendering.
    pub style: String,
    pub rarity_weight: u32,
    pub density: f32,
    pub jump_speed: f32,
    pub flight_time_ms: u32,
    pub buoyancy: f32,
    pub fire_resistance: f32,
    pub starter: bool,
}

#[spacetimedb::table(accessor = piece_def, public)]
pub struct PieceDef {
    #[primary_key]
    pub id: String,
    pub name: String,
    #[index(btree)]
    pub character_id: String,
}

#[spacetimedb::table(accessor = lootbox_def, public)]
pub struct LootboxDef {
    #[primary_key]
    pub id: String,
    pub name: String,
}

#[spacetimedb::table(accessor = lootbox_drop, public)]
pub struct LootboxDrop {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub lootbox_id: String,
    pub piece_id: String,
    pub weight: u32,
}

// ---------------------------------------------------------------------------
// Player-owned tables (public, but visible only to their owner)
// ---------------------------------------------------------------------------

#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub selected_character_id: String,
}

#[client_visibility_filter]
const PLAYER_VIS: Filter = Filter::Sql("SELECT * FROM player WHERE identity = :sender");

#[spacetimedb::table(accessor = player_enabled_level, public,
    index(accessor = by_owner_level, btree(columns = [owner, level_id])))]
pub struct PlayerEnabledLevel {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner: Identity,
    pub level_id: String,
}

#[client_visibility_filter]
const PLAYER_ENABLED_LEVEL_VIS: Filter =
    Filter::Sql("SELECT * FROM player_enabled_level WHERE owner = :sender");

#[spacetimedb::table(accessor = player_completed_level, public,
    index(accessor = by_owner_level, btree(columns = [owner, level_id])))]
pub struct PlayerCompletedLevel {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner: Identity,
    pub level_id: String,
    pub completed_at: Timestamp,
}

#[client_visibility_filter]
const PLAYER_COMPLETED_LEVEL_VIS: Filter =
    Filter::Sql("SELECT * FROM player_completed_level WHERE owner = :sender");

#[spacetimedb::table(accessor = player_lootbox, public)]
pub struct PlayerLootbox {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub owner: Identity,
    pub lootbox_id: String,
    pub granted_at: Timestamp,
    pub opened: bool,
    /// Set by `open_lootbox` before the client plays any reveal animation.
    pub awarded_piece_id: Option<String>,
}

#[client_visibility_filter]
const PLAYER_LOOTBOX_VIS: Filter =
    Filter::Sql("SELECT * FROM player_lootbox WHERE owner = :sender");

#[spacetimedb::table(accessor = player_piece, public,
    index(accessor = by_owner_piece, btree(columns = [owner, piece_id])))]
pub struct PlayerPiece {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner: Identity,
    pub piece_id: String,
    pub count: u32,
}

#[client_visibility_filter]
const PLAYER_PIECE_VIS: Filter = Filter::Sql("SELECT * FROM player_piece WHERE owner = :sender");

#[spacetimedb::table(accessor = player_unlocked_character, public,
    index(accessor = by_owner_character, btree(columns = [owner, character_id])))]
pub struct PlayerUnlockedCharacter {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner: Identity,
    pub character_id: String,
}

#[client_visibility_filter]
const PLAYER_UNLOCKED_CHARACTER_VIS: Filter =
    Filter::Sql("SELECT * FROM player_unlocked_character WHERE owner = :sender");

// ---------------------------------------------------------------------------
// Private module state
// ---------------------------------------------------------------------------

#[spacetimedb::table(accessor = module_owner)]
pub struct ModuleOwner {
    #[primary_key]
    pub id: u8,
    pub owner: Identity,
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    ctx.db.module_owner().insert(ModuleOwner {
        id: 0,
        owner: ctx.sender(),
    });
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    bootstrap_player(ctx);
}

/// Idempotently ensure the connecting player has a row, the starter
/// characters, and every active starting level enabled. Safe to re-run so
/// content imported after a player's first connection is picked up.
fn bootstrap_player(ctx: &ReducerContext) {
    let sender = ctx.sender();
    for character in ctx.db.character_def().iter().filter(|c| c.starter) {
        unlock_character_if_absent(ctx, sender, &character.id);
    }
    for level in ctx.db.level().iter().filter(|l| l.active && l.is_starting) {
        enable_level_if_absent(ctx, sender, &level.id);
    }
    if ctx.db.player().identity().find(sender).is_none() {
        let starter = ctx
            .db
            .character_def()
            .iter()
            .filter(|c| c.starter)
            .map(|c| c.id)
            .min()
            .unwrap_or_default();
        ctx.db.player().insert(Player {
            identity: sender,
            selected_character_id: starter,
        });
    } else if let Some(player) = ctx.db.player().identity().find(sender)
        && player.selected_character_id.is_empty()
        && let Some(starter) = ctx
            .db
            .character_def()
            .iter()
            .filter(|c| c.starter)
            .map(|c| c.id)
            .min()
    {
        ctx.db.player().identity().update(Player {
            selected_character_id: starter,
            ..player
        });
    }
}

fn enable_level_if_absent(ctx: &ReducerContext, owner: Identity, level_id: &str) {
    if ctx
        .db
        .player_enabled_level()
        .by_owner_level()
        .filter((owner, level_id))
        .next()
        .is_none()
    {
        ctx.db.player_enabled_level().insert(PlayerEnabledLevel {
            id: 0,
            owner,
            level_id: level_id.to_string(),
        });
    }
}

fn unlock_character_if_absent(ctx: &ReducerContext, owner: Identity, character_id: &str) {
    if ctx
        .db
        .player_unlocked_character()
        .by_owner_character()
        .filter((owner, character_id))
        .next()
        .is_none()
    {
        ctx.db
            .player_unlocked_character()
            .insert(PlayerUnlockedCharacter {
                id: 0,
                owner,
                character_id: character_id.to_string(),
            });
    }
}

// ---------------------------------------------------------------------------
// Player reducers
// ---------------------------------------------------------------------------

#[spacetimedb::reducer]
pub fn select_character(ctx: &ReducerContext, character_id: String) -> Result<(), String> {
    let sender = ctx.sender();
    let player = ctx
        .db
        .player()
        .identity()
        .find(sender)
        .ok_or("player not initialized")?;
    if ctx
        .db
        .player_unlocked_character()
        .by_owner_character()
        .filter((sender, character_id.as_str()))
        .next()
        .is_none()
    {
        return Err(format!("character '{character_id}' is not unlocked"));
    }
    ctx.db.player().identity().update(Player {
        selected_character_id: character_id,
        ..player
    });
    Ok(())
}

/// Client-reported completion of an enabled level. Idempotent: the first
/// completion records it, enables configured successors, and grants exactly
/// one unopened reward lootbox in the same transaction; replays are no-ops.
#[spacetimedb::reducer]
pub fn complete_level(ctx: &ReducerContext, level_id: String) -> Result<(), String> {
    let sender = ctx.sender();
    let level = ctx
        .db
        .level()
        .id()
        .find(&level_id)
        .ok_or_else(|| format!("unknown level '{level_id}'"))?;
    if !level.active {
        return Err(format!("level '{level_id}' is not active"));
    }
    if ctx
        .db
        .player_enabled_level()
        .by_owner_level()
        .filter((sender, level_id.as_str()))
        .next()
        .is_none()
    {
        return Err(format!("level '{level_id}' is not enabled for this player"));
    }
    let already_completed = ctx
        .db
        .player_completed_level()
        .by_owner_level()
        .filter((sender, level_id.as_str()))
        .next()
        .is_some();
    if !progression::grants_first_completion_rewards(already_completed) {
        return Ok(());
    }

    ctx.db
        .player_completed_level()
        .insert(PlayerCompletedLevel {
            id: 0,
            owner: sender,
            level_id: level_id.clone(),
            completed_at: ctx.timestamp,
        });

    let configured: Vec<String> = ctx
        .db
        .level_successor()
        .level_id()
        .filter(level_id.as_str())
        .map(|edge| edge.successor_id)
        .collect();
    let enabled: Vec<String> = ctx
        .db
        .player_enabled_level()
        .by_owner_level()
        .filter(sender)
        .map(|row| row.level_id)
        .collect();
    for successor in progression::successor_inserts(&configured, &enabled) {
        ctx.db.player_enabled_level().insert(PlayerEnabledLevel {
            id: 0,
            owner: sender,
            level_id: successor,
        });
    }

    if ctx
        .db
        .lootbox_def()
        .id()
        .find(&level.reward_lootbox_id)
        .is_some()
    {
        ctx.db.player_lootbox().insert(PlayerLootbox {
            id: 0,
            owner: sender,
            lootbox_id: level.reward_lootbox_id,
            granted_at: ctx.timestamp,
            opened: false,
            awarded_piece_id: None,
        });
    }
    Ok(())
}

/// Open an unopened lootbox the caller owns. Picks one unique piece
/// definition using `ctx.rng()` and weights of `drop.weight * rarity_weight`
/// of the piece's character, increments the caller's piece count (duplicates
/// allowed), persists the award on the lootbox row, and unlocks the piece's
/// character once every required unique piece is owned.
#[spacetimedb::reducer]
pub fn open_lootbox(ctx: &ReducerContext, player_lootbox_id: u64) -> Result<(), String> {
    let sender = ctx.sender();
    let lootbox = ctx
        .db
        .player_lootbox()
        .id()
        .find(player_lootbox_id)
        .ok_or("unknown lootbox")?;
    progression::ensure_owner(sender, lootbox.owner)?;
    progression::ensure_unopened(lootbox.opened)?;

    let mut weighted: Vec<(String, u64)> = Vec::new();
    for drop in ctx
        .db
        .lootbox_drop()
        .lootbox_id()
        .filter(lootbox.lootbox_id.as_str())
    {
        let piece = ctx
            .db
            .piece_def()
            .id()
            .find(&drop.piece_id)
            .ok_or_else(|| format!("drop references unknown piece '{}'", drop.piece_id))?;
        let character = ctx
            .db
            .character_def()
            .id()
            .find(&piece.character_id)
            .ok_or_else(|| {
                format!(
                    "piece references unknown character '{}'",
                    piece.character_id
                )
            })?;
        weighted.push((
            piece.id,
            drop.weight as u64 * character.rarity_weight as u64,
        ));
    }
    let total: u64 = weighted.iter().map(|(_, w)| w).sum();
    if total == 0 {
        return Err(format!(
            "lootbox '{}' has no weighted drops",
            lootbox.lootbox_id
        ));
    }
    let roll = ctx.rng().gen_range(0..total);
    let piece_id = progression::pick_weighted(&weighted, roll)
        .ok_or("weighted pick failed")?
        .to_string();

    let count = match ctx
        .db
        .player_piece()
        .by_owner_piece()
        .filter((sender, piece_id.as_str()))
        .next()
    {
        Some(existing) => {
            let count = existing.count + 1;
            ctx.db
                .player_piece()
                .id()
                .update(PlayerPiece { count, ..existing });
            count
        }
        None => {
            ctx.db.player_piece().insert(PlayerPiece {
                id: 0,
                owner: sender,
                piece_id: piece_id.clone(),
                count: 1,
            });
            1
        }
    };
    log::info!("player {sender} got piece {piece_id} (count {count})");

    ctx.db.player_lootbox().id().update(PlayerLootbox {
        opened: true,
        awarded_piece_id: Some(piece_id.clone()),
        ..lootbox
    });

    let character_id = ctx
        .db
        .piece_def()
        .id()
        .find(&piece_id)
        .map(|piece| piece.character_id)
        .unwrap_or_default();
    let required: Vec<String> = ctx
        .db
        .piece_def()
        .character_id()
        .filter(character_id.as_str())
        .map(|piece| piece.id)
        .collect();
    let owned: Vec<String> = ctx
        .db
        .player_piece()
        .by_owner_piece()
        .filter(sender)
        .filter(|row| row.count > 0)
        .map(|row| row.piece_id)
        .collect();
    if progression::unlocks_character(&required, &owned) {
        unlock_character_if_absent(ctx, sender, &character_id);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Owner-only import reducers (called by the admin CLI)
// ---------------------------------------------------------------------------

#[derive(SpacetimeType)]
pub struct LayerImport {
    pub z: u8,
    pub width: u16,
    pub height: u16,
    pub cell_width: u16,
    pub cell_height: u16,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[derive(SpacetimeType)]
pub struct DropImport {
    pub piece_id: String,
    pub weight: u32,
}

fn ensure_module_owner(ctx: &ReducerContext) -> Result<(), String> {
    let owner = ctx
        .db
        .module_owner()
        .id()
        .find(0)
        .ok_or("module owner not initialized")?;
    progression::ensure_owner(ctx.sender(), owner.owner)
}

/// Atomically overwrite one level's metadata, layers, and successor edges.
/// Imports replace the stored rows for the stable level ID; git history of
/// the committed Tiled files is the rollback mechanism.
#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn import_level(
    ctx: &ReducerContext,
    id: String,
    name: String,
    is_starting: bool,
    active: bool,
    reward_lootbox_id: String,
    successors: Vec<String>,
    layers: Vec<LayerImport>,
) -> Result<(), String> {
    ensure_module_owner(ctx)?;
    if id.is_empty() {
        return Err("level id must not be empty".to_string());
    }
    let facts: Vec<LayerFacts> = layers
        .iter()
        .map(|layer| LayerFacts {
            z: layer.z,
            width: layer.width,
            height: layer.height,
            cell_width: layer.cell_width,
            cell_height: layer.cell_height,
            parallax_x: layer.parallax_x,
            parallax_y: layer.parallax_y,
            encoding: layer.encoding.clone(),
            content_hash: layer.content_hash.clone(),
            data: layer.data.clone(),
        })
        .collect();
    validate_layers(&facts)?;

    ctx.db.level().id().delete(&id);
    ctx.db.level_layer().level_id().delete(id.as_str());
    ctx.db.level_successor().level_id().delete(id.as_str());

    ctx.db.level().insert(Level {
        id: id.clone(),
        name,
        is_starting,
        active,
        reward_lootbox_id,
    });
    for layer in layers {
        ctx.db.level_layer().insert(LevelLayer {
            id: 0,
            level_id: id.clone(),
            z: layer.z,
            width: layer.width,
            height: layer.height,
            cell_width: layer.cell_width,
            cell_height: layer.cell_height,
            parallax_x: layer.parallax_x,
            parallax_y: layer.parallax_y,
            encoding: layer.encoding,
            content_hash: layer.content_hash,
            data: layer.data,
        });
    }
    for successor_id in successors {
        if successor_id.is_empty() || successor_id == id {
            return Err(format!("invalid successor id '{successor_id}'"));
        }
        ctx.db.level_successor().insert(LevelSuccessor {
            id: 0,
            level_id: id.clone(),
            successor_id,
        });
    }
    Ok(())
}

#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn import_character(
    ctx: &ReducerContext,
    id: String,
    name: String,
    style: String,
    rarity_weight: u32,
    density: f32,
    jump_speed: f32,
    flight_time_ms: u32,
    buoyancy: f32,
    fire_resistance: f32,
    starter: bool,
) -> Result<(), String> {
    ensure_module_owner(ctx)?;
    if id.is_empty() {
        return Err("character id must not be empty".to_string());
    }
    if rarity_weight == 0 {
        return Err("rarity_weight must be positive".to_string());
    }
    let row = CharacterDef {
        id: id.clone(),
        name,
        style,
        rarity_weight,
        density,
        jump_speed,
        flight_time_ms,
        buoyancy,
        fire_resistance,
        starter,
    };
    if ctx.db.character_def().id().find(&id).is_some() {
        ctx.db.character_def().id().update(row);
    } else {
        ctx.db.character_def().insert(row);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn import_piece(
    ctx: &ReducerContext,
    id: String,
    name: String,
    character_id: String,
) -> Result<(), String> {
    ensure_module_owner(ctx)?;
    if id.is_empty() {
        return Err("piece id must not be empty".to_string());
    }
    if ctx.db.character_def().id().find(&character_id).is_none() {
        return Err(format!(
            "piece references unknown character '{character_id}'"
        ));
    }
    let row = PieceDef {
        id: id.clone(),
        name,
        character_id,
    };
    if ctx.db.piece_def().id().find(&id).is_some() {
        ctx.db.piece_def().id().update(row);
    } else {
        ctx.db.piece_def().insert(row);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn import_lootbox(
    ctx: &ReducerContext,
    id: String,
    name: String,
    drops: Vec<DropImport>,
) -> Result<(), String> {
    ensure_module_owner(ctx)?;
    if id.is_empty() {
        return Err("lootbox id must not be empty".to_string());
    }
    if drops.is_empty() {
        return Err("lootbox must configure at least one drop".to_string());
    }
    for drop in &drops {
        if drop.weight == 0 {
            return Err(format!(
                "drop '{}' must have a positive weight",
                drop.piece_id
            ));
        }
        if ctx.db.piece_def().id().find(&drop.piece_id).is_none() {
            return Err(format!("drop references unknown piece '{}'", drop.piece_id));
        }
    }
    let row = LootboxDef {
        id: id.clone(),
        name,
    };
    if ctx.db.lootbox_def().id().find(&id).is_some() {
        ctx.db.lootbox_def().id().update(row);
    } else {
        ctx.db.lootbox_def().insert(row);
    }
    ctx.db.lootbox_drop().lootbox_id().delete(id.as_str());
    for drop in drops {
        ctx.db.lootbox_drop().insert(LootboxDrop {
            id: 0,
            lootbox_id: id.clone(),
            piece_id: drop.piece_id,
            weight: drop.weight,
        });
    }
    Ok(())
}
