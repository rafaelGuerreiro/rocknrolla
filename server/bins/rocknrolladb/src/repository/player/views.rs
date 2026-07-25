//! Public read models for the caller's own player state.

use crate::repository::player::{
    Player, PlayerPiece, PlayerUnlockedCharacter, player_piece_v1__view, player_unlocked_character_v1__view, player_v1__view,
};
use spacetimedb::{Identity, SpacetimeType, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct MeViewV1 {
    pub identity: Identity,
    pub selected_character_id: Option<Uuid>,
}

#[derive(SpacetimeType)]
pub struct MyPieceViewV1 {
    pub piece_id: Uuid,
    pub count: u32,
}

#[derive(SpacetimeType)]
pub struct MyUnlockedCharacterViewV1 {
    pub character_id: Uuid,
}

/// The caller's own player row; never another player's. Mapped through a
/// dedicated view struct so the private table type never enters bindings.
#[view(accessor = vw_me_v1, name = "vw_me_v1", public)]
pub fn vw_me_v1(ctx: &ViewContext) -> Option<MeViewV1> {
    ctx.db.player_v1().identity().find(ctx.sender()).map(
        |Player {
             identity,
             selected_character_id,
         }| MeViewV1 {
            identity,
            selected_character_id,
        },
    )
}

/// The caller's piece collection; never another player's.
#[view(accessor = vw_my_piece_v1, name = "vw_my_piece_v1", public)]
pub fn vw_my_piece_v1(ctx: &ViewContext) -> Vec<MyPieceViewV1> {
    ctx.db
        .player_piece_v1()
        .by_owner_piece()
        .filter(ctx.sender())
        .map(|PlayerPiece { piece_id, count, .. }| MyPieceViewV1 { piece_id, count })
        .collect()
}

/// The caller's unlocked characters; never another player's.
#[view(accessor = vw_my_unlocked_character_v1, name = "vw_my_unlocked_character_v1", public)]
pub fn vw_my_unlocked_character_v1(ctx: &ViewContext) -> Vec<MyUnlockedCharacterViewV1> {
    ctx.db
        .player_unlocked_character_v1()
        .by_owner_character()
        .filter(ctx.sender())
        .map(|PlayerUnlockedCharacter { character_id, .. }| MyUnlockedCharacterViewV1 { character_id })
        .collect()
}
