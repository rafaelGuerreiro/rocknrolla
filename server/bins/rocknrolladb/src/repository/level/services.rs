//! Level repository services: content import and level lookups.

use crate::{
    error::ServiceResult,
    extend::{make_service::make_service, stdb::UuidGen},
    repository::{
        backdrop::services::BackdropServicesTrait,
        component::services::ComponentServicesTrait,
        level::{
            Level, LevelPlacement, LevelSuccessor, errors::LevelError, level_placement_v1, level_successor_v1, level_v1,
            types::PlacementImportV1,
        },
    },
};
use rocknrolla_level::{PlacementFacts, Vec2, validate_level_geometry};
use spacetimedb::{Table, Uuid};

make_service!(level_services);

/// A validated request to overwrite one authored level.
pub struct LevelImport {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub is_starting: bool,
    pub active: bool,
    pub reward_lootbox_id: Option<Uuid>,
    pub successors: Vec<Uuid>,
    /// The backdrop's authored identity; resolved to its UUID at import.
    pub backdrop_slug: String,
    pub spawn: Vec2,
    pub finish: Vec2,
    pub placements: Vec<PlacementImportV1>,
}

impl LevelServicesImpl<'_> {
    /// Atomically overwrite one level's metadata, placements, and successor
    /// edges. Every placement must reference an imported component slug and
    /// spawn/finish must land inside the gameplay-plane bounds. The stable
    /// authored UUID is the replacement key; git history of the committed
    /// authored sources is the rollback mechanism.
    pub fn import_level(&self, import: LevelImport) -> ServiceResult<()> {
        let components = self.ctx.component_services();
        let mut resolved = Vec::with_capacity(import.placements.len());
        let mut facts = Vec::with_capacity(import.placements.len());
        for placement in &import.placements {
            let component = components
                .find_by_slug(&placement.component_slug)
                .ok_or_else(|| LevelError::unknown_component(&import.slug, &placement.component_slug))?;
            facts.push(PlacementFacts {
                position: placement.position,
                scale: placement.scale,
                component_width_px: component.width_px,
                component_height_px: component.height_px,
            });
            resolved.push(component.id);
        }
        validate_level_geometry(&facts, import.spawn, import.finish)?;
        let backdrop = self
            .ctx
            .backdrop_services()
            .find_by_slug(&import.backdrop_slug)
            .ok_or_else(|| LevelError::unknown_backdrop(&import.slug, &import.backdrop_slug))?;
        for successor in &import.successors {
            if *successor == import.id {
                return Err(LevelError::self_successor(&import.slug));
            }
        }
        if let Some(other) = self.db.level_v1().slug().find(&import.slug)
            && other.id != import.id
        {
            return Err(LevelError::slug_conflict(&import.slug, other.id));
        }

        self.db.level_v1().id().insert_or_update(Level {
            id: import.id,
            slug: import.slug,
            name: import.name,
            is_starting: import.is_starting,
            active: import.active,
            reward_lootbox_id: import.reward_lootbox_id,
            backdrop_id: backdrop.id,
            spawn: import.spawn,
            finish: import.finish,
        });
        self.db.level_placement_v1().level_id().delete(import.id);
        self.db.level_successor_v1().level_id().delete(import.id);
        for (order, (placement, component_id)) in import.placements.iter().zip(resolved).enumerate() {
            self.db.level_placement_v1().insert(LevelPlacement {
                id: self.ctx.generate_uuid()?,
                level_id: import.id,
                component_id,
                position: placement.position,
                flip_x: placement.flip_x,
                scale: placement.scale,
                order: order as u32,
            });
        }
        for successor_id in import.successors {
            self.db.level_successor_v1().insert(LevelSuccessor {
                id: self.ctx.generate_uuid()?,
                level_id: import.id,
                successor_id,
            });
        }
        Ok(())
    }

    pub fn find_active_level(&self, level_id: Uuid) -> ServiceResult<Level> {
        let level = self
            .db
            .level_v1()
            .id()
            .find(level_id)
            .ok_or_else(|| LevelError::unknown_level(level_id))?;
        if !level.active {
            return Err(LevelError::inactive(&level.slug));
        }
        Ok(level)
    }

    pub fn active_starting_level_ids(&self) -> Vec<Uuid> {
        self.db
            .level_v1()
            .iter()
            .filter(|l| l.active && l.is_starting)
            .map(|l| l.id)
            .collect()
    }

    pub fn successor_ids(&self, level_id: Uuid) -> Vec<Uuid> {
        self.db
            .level_successor_v1()
            .level_id()
            .filter(level_id)
            .map(|edge| edge.successor_id)
            .collect()
    }
}
