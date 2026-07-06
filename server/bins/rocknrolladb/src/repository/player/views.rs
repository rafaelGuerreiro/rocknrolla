//! Public read models for the caller's own player state.

use crate::repository::player::{
    Player, PlayerPiece, PlayerUnlockedCharacter, player__view, player_piece__view, player_unlocked_character__view,
};
use spacetimedb::{SpacetimeType, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct MyPieceView {
    pub piece_id: Uuid,
    pub count: u32,
}

#[derive(SpacetimeType)]
pub struct MyUnlockedCharacterView {
    pub character_id: Uuid,
}

/// The caller's own player row; never another player's.
#[view(accessor = vw_me, public)]
pub fn vw_me(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender())
}

/// The caller's piece collection; never another player's.
#[view(accessor = vw_my_piece, public)]
pub fn vw_my_piece(ctx: &ViewContext) -> Vec<MyPieceView> {
    ctx.db
        .player_piece()
        .by_owner_piece()
        .filter(ctx.sender())
        .map(|PlayerPiece { piece_id, count, .. }| MyPieceView { piece_id, count })
        .collect()
}

/// The caller's unlocked characters; never another player's.
#[view(accessor = vw_my_unlocked_character, public)]
pub fn vw_my_unlocked_character(ctx: &ViewContext) -> Vec<MyUnlockedCharacterView> {
    ctx.db
        .player_unlocked_character()
        .by_owner_character()
        .filter(ctx.sender())
        .map(|PlayerUnlockedCharacter { character_id, .. }| MyUnlockedCharacterView { character_id })
        .collect()
}
