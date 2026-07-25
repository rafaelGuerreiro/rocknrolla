//! Level domain error variants, mapped onto the shared `ServiceError` categories.

use crate::error::ServiceError;
use spacetimedb::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LevelError {
    #[error("level '{level_slug}' places unknown component '{component_slug}'")]
    UnknownComponent { level_slug: String, component_slug: String },
    #[error("level '{level_slug}' references unknown backdrop '{backdrop_slug}'")]
    UnknownBackdrop { level_slug: String, backdrop_slug: String },
    #[error("level '{level_slug}' lists itself as a successor")]
    SelfSuccessor { level_slug: String },
    #[error("slug '{slug}' already belongs to level {existing_id}")]
    SlugConflict { slug: String, existing_id: Uuid },
    #[error("unknown level '{level_id}'")]
    UnknownLevel { level_id: Uuid },
    #[error("level '{slug}' is not active")]
    Inactive { slug: String },
}

impl LevelError {
    pub fn unknown_component(level_slug: &str, component_slug: &str) -> ServiceError {
        LevelError::UnknownComponent {
            level_slug: level_slug.to_string(),
            component_slug: component_slug.to_string(),
        }
        .into()
    }

    pub fn unknown_backdrop(level_slug: &str, backdrop_slug: &str) -> ServiceError {
        LevelError::UnknownBackdrop {
            level_slug: level_slug.to_string(),
            backdrop_slug: backdrop_slug.to_string(),
        }
        .into()
    }

    pub fn self_successor(level_slug: &str) -> ServiceError {
        LevelError::SelfSuccessor {
            level_slug: level_slug.to_string(),
        }
        .into()
    }

    pub fn slug_conflict(slug: &str, existing_id: Uuid) -> ServiceError {
        LevelError::SlugConflict {
            slug: slug.to_string(),
            existing_id,
        }
        .into()
    }

    pub fn unknown_level(level_id: Uuid) -> ServiceError {
        LevelError::UnknownLevel { level_id }.into()
    }

    pub fn inactive(slug: &str) -> ServiceError {
        LevelError::Inactive { slug: slug.to_string() }.into()
    }
}

impl From<LevelError> for ServiceError {
    fn from(err: LevelError) -> Self {
        let message = err.to_string();
        match err {
            LevelError::UnknownComponent { .. } | LevelError::UnknownBackdrop { .. } | LevelError::SelfSuccessor { .. } => {
                ServiceError::validation(message)
            },
            LevelError::SlugConflict { .. } | LevelError::Inactive { .. } => ServiceError::conflict(message),
            LevelError::UnknownLevel { .. } => ServiceError::not_found(message),
        }
    }
}
