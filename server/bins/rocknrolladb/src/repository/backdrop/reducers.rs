//! Backdrop reducers: parameter/caller validation plus one service delegation.

use crate::{
    error::ServiceResult,
    extend::validate::validate_required_str,
    repository::{
        access,
        backdrop::{
            services::BackdropServicesTrait,
            types::{BackdropImportV1, BackdropLayerV1},
        },
    },
};
use rocknrolla_level::validate_svg_asset;
use spacetimedb::ReducerContext;

fn validate_layer(slug: &str, role: &str, layer: &BackdropLayerV1) -> ServiceResult<()> {
    validate_svg_asset(
        "backdrop layer",
        &format!("{slug}.{role}"),
        layer.width_px,
        layer.height_px,
        &layer.content_hash,
        &layer.data,
    )
}

#[spacetimedb::reducer(name = "import_backdrop_v1")]
pub fn import_backdrop_v1(ctx: &ReducerContext, backdrop: BackdropImportV1) -> ServiceResult<()> {
    access::require_module_owner(ctx, ctx.sender())?;
    validate_required_str(&backdrop.slug, "slug", 64)?;
    validate_layer(&backdrop.slug, "sky", &backdrop.sky)?;
    validate_layer(&backdrop.slug, "far", &backdrop.far)?;
    validate_layer(&backdrop.slug, "mid", &backdrop.mid)?;
    ctx.backdrop_services().import_backdrop(backdrop)
}
