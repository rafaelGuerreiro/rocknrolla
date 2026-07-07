//! Module administration access: which identity may import content.

use crate::error::{ServiceError, ServiceResult};
use spacetimedb::{Identity, ReducerContext, Table};

#[spacetimedb::table(accessor = module_owner_v1, private)]
pub struct ModuleOwner {
    #[primary_key]
    pub id: u8,
    pub owner: Identity,
}

/// Record the publishing identity as the module owner during `init`.
pub fn record_module_owner(ctx: &ReducerContext, owner: Identity) {
    ctx.db.module_owner_v1().insert(ModuleOwner { id: 0, owner });
}

/// Owner-only guard for content import reducers.
pub fn require_module_owner(ctx: &ReducerContext, sender: Identity) -> ServiceResult<()> {
    let record = ctx
        .db
        .module_owner_v1()
        .id()
        .find(0)
        .ok_or_else(|| ServiceError::internal("module owner not initialized"))?;
    if record.owner == sender {
        Ok(())
    } else {
        Err(ServiceError::forbidden(sender, "only the module owner may import content"))
    }
}
