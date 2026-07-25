//! Backdrop content: named per-level scenery themes (sky + far + mid
//! parallax strips) referenced by levels as a whole — never placed, never
//! colliding.

use crate::repository::backdrop::types::BackdropLayerV1;
use spacetimedb::Uuid;

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[spacetimedb::table(accessor = backdrop_v1, name = "backdrop_v1", private)]
pub struct Backdrop {
    #[primary_key]
    pub id: Uuid,
    /// Authored identity: the layer filenames' prefix in
    /// `content/backdrops/` (`<slug>.{sky,far,mid}.svg`). One row per slug,
    /// enforced by the import upsert (btree, not unique, so views can
    /// range-scan the whole set).
    #[index(btree)]
    pub slug: String,
    pub sky: BackdropLayerV1,
    pub far: BackdropLayerV1,
    pub mid: BackdropLayerV1,
}
