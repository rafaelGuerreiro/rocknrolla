//! Component repository services: library import and lookups.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::component::{Component, component_v1, types::ComponentImportV1},
};
use rocknrolla_level::{ComponentFacts, validate_component};

make_service!(component_services);

impl ComponentServicesImpl<'_> {
    /// Atomically overwrite one component by slug. The slug is the authored
    /// identity (filename); the UUID is generated on first import and kept
    /// stable across overwrites so placements never dangle.
    pub fn import_component(&self, import: ComponentImportV1) -> ServiceResult<()> {
        validate_component(&ComponentFacts {
            slug: import.slug.clone(),
            width_px: import.width_px,
            height_px: import.height_px,
            content_hash: import.content_hash.clone(),
            data: import.data.clone(),
        })?;
        let id = match self.db.component_v1().slug().find(&import.slug) {
            Some(existing) => existing.id,
            None => self.ctx.generate_uuid()?,
        };
        self.db.component_v1().id().insert_or_update(Component {
            id,
            slug: import.slug,
            width_px: import.width_px,
            height_px: import.height_px,
            content_hash: import.content_hash,
            data: import.data,
        });
        Ok(())
    }

    pub fn find_by_slug(&self, slug: &String) -> Option<Component> {
        self.db.component_v1().slug().find(slug)
    }
}
