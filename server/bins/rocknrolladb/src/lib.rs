//! RocknRolla SpacetimeDB module: module wiring and lifecycle coordination.
//!
//! Domain behavior lives in [`repository`]; shared checks in [`extend`];
//! the typed service error model in [`error`].

use crate::error::ServiceResult;
use spacetimedb::ReducerContext;

pub mod error;
pub mod extend;
pub mod repository;

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    repository::init(ctx);
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    repository::identity_connected(ctx)
}
