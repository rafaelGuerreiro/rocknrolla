//! Lootbox reducers: parameter/caller validation plus one service delegation.

use crate::error::ServiceResult;
use crate::extend::validate::validate_required_str;
use crate::repository::access;
use crate::repository::lootbox::services::LootboxReducerContext;
use crate::repository::lootbox::types::DropImport;
use spacetimedb::{ReducerContext, Uuid};

#[spacetimedb::reducer]
pub fn import_lootbox(
    ctx: &ReducerContext,
    id: Uuid,
    name: String,
    drops: Vec<DropImport>,
) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&name, "name", 128)?;
    ctx.lootbox_services().import_lootbox(id, name, drops)
}

#[spacetimedb::reducer]
pub fn open_lootbox(ctx: &ReducerContext, player_lootbox_id: Uuid) -> ServiceResult<()> {
    ctx.lootbox_services()
        .open_lootbox(ctx.sender(), player_lootbox_id)
}
