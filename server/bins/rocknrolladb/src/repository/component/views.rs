//! Public read model for the component library. Only components placed in
//! active levels are exposed and internal fields stay private.

use crate::repository::{
    component::component_v1__view,
    level::{level_placement_v1__view, level_v1__view},
};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct ComponentViewV1 {
    pub id: Uuid,
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

#[view(accessor = vw_component_v1, name = "vw_component_v1", public)]
pub fn vw_component_v1(ctx: &AnonymousViewContext) -> Vec<ComponentViewV1> {
    let mut ids: Vec<Uuid> = ctx
        .db
        .level_v1()
        .active()
        .filter(true)
        .flat_map(|level| ctx.db.level_placement_v1().level_id().filter(level.id))
        .map(|placement| placement.component_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids.into_iter()
        .filter_map(|id| ctx.db.component_v1().id().find(id))
        .map(|component| ComponentViewV1 {
            id: component.id,
            slug: component.slug,
            width_px: component.width_px,
            height_px: component.height_px,
            content_hash: component.content_hash,
            data: component.data,
        })
        .collect()
}
