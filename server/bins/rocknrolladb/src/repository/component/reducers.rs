//! Component reducers: parameter/caller validation plus one service delegation.

use crate::{
    error::ServiceResult,
    extend::validate::validate_required_str,
    repository::{
        access,
        component::{services::ComponentReducerContext, types::ComponentImportV1},
    },
};
use spacetimedb::ReducerContext;

#[spacetimedb::reducer]
pub fn import_component(ctx: &ReducerContext, component: ComponentImportV1) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&component.slug, "slug", 64)?;
    ctx.component_services().import_component(component)
}
