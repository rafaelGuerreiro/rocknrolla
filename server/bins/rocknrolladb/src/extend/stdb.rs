//! Reusable SpacetimeDB helpers.

use crate::error::{ServiceError, ServiceResult};
use spacetimedb::{ReducerContext, Uuid};

/// UUID generation with failures surfaced as typed service errors.
pub trait UuidGen {
    fn generate_uuid(&self) -> ServiceResult<Uuid>;
}

impl UuidGen for ReducerContext {
    fn generate_uuid(&self) -> ServiceResult<Uuid> {
        self.new_uuid_v7()
            .map_err(|e| ServiceError::internal(format!("uuid generation failed: {e}")))
    }
}

/// Full-range bound covering every UUID, for index scans in views
/// (view handles expose indexes but cannot iterate whole tables).
pub fn all_uuids() -> std::ops::RangeFrom<Uuid> {
    Uuid::from_u128(0)..
}
