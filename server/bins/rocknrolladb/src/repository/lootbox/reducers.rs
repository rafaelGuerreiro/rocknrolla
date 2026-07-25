//! Lootbox reducers: parameter/caller validation plus one service delegation.

use crate::{
    error::ServiceResult,
    extend::validate::validate_required_str,
    repository::{
        access,
        lootbox::{services::LootboxServicesTrait, types::DropImportV1},
    },
};
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer(name = "import_lootbox_v1")]
pub fn import_lootbox_v1(ctx: &ReducerContext, id: Uuid, name: String, drops: Vec<DropImportV1>) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&name, "name", 128)?;
    ctx.lootbox_services().import_lootbox(id, name, drops)
}

#[spacetimedb::reducer(name = "open_lootbox_v1")]
pub fn open_lootbox_v1(ctx: &ReducerContext, player_lootbox_id: Uuid) -> ServiceResult<()> {
    ctx.lootbox_services().open_lootbox(ctx.sender(), player_lootbox_id)
}
