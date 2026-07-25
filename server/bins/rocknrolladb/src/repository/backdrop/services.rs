//! Backdrop repository services: content import and slug lookups.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::backdrop::{Backdrop, backdrop_v1, types::BackdropImportV1},
};

make_service!(backdrop_services);

impl BackdropServicesImpl<'_> {
    /// Atomically overwrite one backdrop by slug. The slug is the authored
    /// identity (filename prefix); the UUID is generated on first import and
    /// kept stable across overwrites so level references never dangle.
    pub fn import_backdrop(&self, import: BackdropImportV1) -> ServiceResult<()> {
        let id = match self.find_by_slug(&import.slug) {
            Some(existing) => existing.id,
            None => self.ctx.generate_uuid()?,
        };
        self.db.backdrop_v1().id().insert_or_update(Backdrop {
            id,
            slug: import.slug,
            sky: import.sky,
            far: import.far,
            mid: import.mid,
        });
        Ok(())
    }

    pub fn find_by_slug(&self, slug: &str) -> Option<Backdrop> {
        self.db.backdrop_v1().slug().filter(slug).next()
    }
}
