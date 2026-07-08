//! Public read model for backdrops. All backdrops are exposed: the set is
//! tiny and the menus need the default backdrop even when no active level
//! references it.

use crate::{
    extend::stdb::all_slugs,
    repository::backdrop::{backdrop_v1__view, types::BackdropLayerV1},
};
use spacetimedb::{AnonymousViewContext, SpacetimeType, Uuid, view};

#[derive(SpacetimeType)]
pub struct BackdropViewV1 {
    pub id: Uuid,
    pub slug: String,
    pub sky: BackdropLayerV1,
    pub far: BackdropLayerV1,
    pub mid: BackdropLayerV1,
}

#[view(accessor = vw_backdrop_v1, name = "vw_backdrop_v1", public)]
pub fn vw_backdrop_v1(ctx: &AnonymousViewContext) -> Vec<BackdropViewV1> {
    ctx.db
        .backdrop_v1()
        .slug()
        .filter(all_slugs())
        .map(|b| BackdropViewV1 {
            id: b.id,
            slug: b.slug,
            sky: b.sky,
            far: b.far,
            mid: b.mid,
        })
        .collect()
}
