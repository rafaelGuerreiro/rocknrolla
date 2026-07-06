//! The typed service error/result model, shared with sdk crates via
//! `rocknrolla-error` so both the module and its dependencies use the same
//! failure categories.

pub use rocknrolla_error::{ServiceError, ServiceResult};
