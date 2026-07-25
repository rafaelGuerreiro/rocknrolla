//! Progression domain error variants, mapped onto the shared `ServiceError` categories.

use crate::error::ServiceError;
use spacetimedb::Identity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProgressionError {
    #[error("level '{slug}' is not enabled for this player")]
    LevelNotEnabled { owner: Identity, slug: String },
}

impl ProgressionError {
    pub fn level_not_enabled(owner: Identity, slug: &str) -> ServiceError {
        ProgressionError::LevelNotEnabled {
            owner,
            slug: slug.to_string(),
        }
        .into()
    }
}

impl From<ProgressionError> for ServiceError {
    fn from(err: ProgressionError) -> Self {
        let message = err.to_string();
        match err {
            ProgressionError::LevelNotEnabled { owner, .. } => ServiceError::forbidden(owner, message),
        }
    }
}
