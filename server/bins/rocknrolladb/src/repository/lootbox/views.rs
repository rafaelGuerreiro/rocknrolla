//! Public read model for the caller's own lootboxes.

use crate::repository::lootbox::{lootbox_def_v1__view, player_lootbox_v1__view};
use spacetimedb::{SpacetimeType, Uuid, ViewContext, view};

#[derive(SpacetimeType)]
pub struct MyLootboxViewV1 {
    pub id: Uuid,
    pub lootbox_id: Uuid,
    pub name: String,
    pub opened: bool,
    pub awarded_piece_id: Option<Uuid>,
}

/// The caller's granted lootboxes with their display names; never another
/// player's.
#[view(accessor = vw_my_lootbox_v1, name = "vw_my_lootbox_v1", public)]
pub fn vw_my_lootbox_v1(ctx: &ViewContext) -> Vec<MyLootboxViewV1> {
    ctx.db
        .player_lootbox_v1()
        .by_owner()
        .filter(ctx.sender())
        .filter_map(|granted| {
            let def = ctx.db.lootbox_def_v1().id().find(granted.lootbox_id)?;
            Some(MyLootboxViewV1 {
                id: granted.id,
                lootbox_id: granted.lootbox_id,
                name: def.name,
                opened: granted.opened,
                awarded_piece_id: granted.awarded_piece_id,
            })
        })
        .collect()
}
