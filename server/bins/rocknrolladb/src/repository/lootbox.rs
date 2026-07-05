//! Lootboxes: definitions, weighted drops, and granted player lootboxes.

use spacetimedb::{Identity, Timestamp, Uuid};

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[spacetimedb::table(accessor = lootbox_def, private)]
pub struct LootboxDef {
    #[primary_key]
    pub id: Uuid,
    pub name: String,
}

#[spacetimedb::table(accessor = lootbox_drop, private)]
pub struct LootboxDrop {
    #[primary_key]
    pub id: Uuid,
    #[index(btree)]
    pub lootbox_id: Uuid,
    pub piece_id: Uuid,
    pub weight: u32,
}

#[spacetimedb::table(accessor = player_lootbox, private,
    index(accessor = by_owner, btree(columns = [owner])))]
pub struct PlayerLootbox {
    #[primary_key]
    pub id: Uuid,
    pub owner: Identity,
    pub lootbox_id: Uuid,
    pub granted_at: Timestamp,
    pub opened: bool,
    /// Set by `open_lootbox` before the client plays any reveal animation.
    pub awarded_piece_id: Option<Uuid>,
}
