//! The authenticated player: profile, unlocked characters, and owned pieces.

use spacetimedb::{Identity, Uuid};

pub mod reducers;
pub mod services;
pub mod views;

#[spacetimedb::table(accessor = player_v1, name = "player_v1", private)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub selected_character_id: Option<Uuid>,
}

#[spacetimedb::table(accessor = player_unlocked_character_v1, name = "player_unlocked_character_v1", private,
    index(accessor = by_owner_character, btree(columns = [owner, character_id])))]
pub struct PlayerUnlockedCharacter {
    #[primary_key]
    pub id: Uuid,
    pub owner: Identity,
    pub character_id: Uuid,
}

#[spacetimedb::table(accessor = player_piece_v1, name = "player_piece_v1", private,
    index(accessor = by_owner_piece, btree(columns = [owner, piece_id])))]
pub struct PlayerPiece {
    #[primary_key]
    pub id: Uuid,
    pub owner: Identity,
    pub piece_id: Uuid,
    pub count: u32,
}
