//! Typed service error/result model shared by the SpacetimeDB module and any
//! sdk crate whose failures need to surface as one of these categories rather
//! than a plain string.

use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

/// Broad error categories returned by repository services and reducers.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// The caller is not allowed to perform this operation.
    #[error("E403: {0}")]
    Forbidden(String),

    /// A referenced record does not exist.
    #[error("E404: {0}")]
    NotFound(String),

    /// The operation conflicts with the current state.
    #[error("E409: {0}")]
    Conflict(String),

    /// A request parameter failed validation.
    #[error("E422: {0}")]
    Validation(String),

    /// The server encountered an unexpected condition.
    #[error("E500: {0}")]
    Internal(String),
}

impl ServiceError {
    /// `sender` only needs to be `Display`, not `spacetimedb::Identity`, so
    /// this crate stays free of a SpacetimeDB dependency.
    pub fn forbidden(sender: impl std::fmt::Display, reason: impl Into<String>) -> Self {
        ServiceError::Forbidden(format!("sender {sender}: {}", reason.into()))
    }

    pub fn not_found(what: impl Into<String>) -> Self {
        ServiceError::NotFound(what.into())
    }

    pub fn conflict(reason: impl Into<String>) -> Self {
        ServiceError::Conflict(reason.into())
    }

    pub fn validation(reason: impl Into<String>) -> Self {
        ServiceError::Validation(reason.into())
    }

    pub fn internal(reason: impl Into<String>) -> Self {
        ServiceError::Internal(reason.into())
    }
}
