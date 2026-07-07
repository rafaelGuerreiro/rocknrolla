//! Level repository services: content import and level lookups.

use crate::{
    error::{ServiceError, ServiceResult},
    extend::stdb::UuidGen,
    repository::level::{
        Level, LevelLayer, LevelSuccessor, level_layer_v1, level_successor_v1, level_v1, types::LayerImportV1,
    },
};
use rocknrolla_level::{LayerFacts, validate_layers};
use spacetimedb::{ReducerContext, Table, Uuid};
use std::ops::Deref;

pub trait LevelReducerContext {
    fn level_services(&self) -> LevelServices<'_>;
}

impl LevelReducerContext for ReducerContext {
    fn level_services(&self) -> LevelServices<'_> {
        LevelServices { ctx: self }
    }
}

pub struct LevelServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for LevelServices<'_> {
    type Target = ReducerContext;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

/// A validated request to overwrite one authored level.
pub struct LevelImport {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub is_starting: bool,
    pub active: bool,
    pub reward_lootbox_id: Option<Uuid>,
    pub successors: Vec<Uuid>,
    pub layers: Vec<LayerImportV1>,
}

impl LevelServices<'_> {
    /// Atomically overwrite one level's metadata, layers, and successor
    /// edges. The stable authored UUID is the replacement key; git history of
    /// the committed authored sources is the rollback mechanism.
    pub fn import_level(&self, import: LevelImport) -> ServiceResult<()> {
        let facts: Vec<LayerFacts> = import
            .layers
            .iter()
            .map(|layer| LayerFacts {
                z: layer.z,
                width_px: layer.width_px,
                height_px: layer.height_px,
                parallax_x: layer.parallax_x,
                parallax_y: layer.parallax_y,
                encoding: layer.encoding.clone(),
                content_hash: layer.content_hash.clone(),
                data: layer.data.clone(),
            })
            .collect();
        validate_layers(&facts)?;
        for successor in &import.successors {
            if *successor == import.id {
                return Err(ServiceError::validation(format!(
                    "level '{}' lists itself as a successor",
                    import.slug
                )));
            }
        }
        if let Some(other) = self.db.level_v1().slug().find(&import.slug)
            && other.id != import.id
        {
            return Err(ServiceError::conflict(format!(
                "slug '{}' already belongs to level {}",
                import.slug, other.id
            )));
        }

        self.db.level_v1().id().insert_or_update(Level {
            id: import.id,
            slug: import.slug,
            name: import.name,
            is_starting: import.is_starting,
            active: import.active,
            reward_lootbox_id: import.reward_lootbox_id,
        });
        self.db.level_layer_v1().level_id().delete(import.id);
        self.db.level_successor_v1().level_id().delete(import.id);
        for layer in import.layers {
            self.db.level_layer_v1().insert(LevelLayer {
                id: self.ctx.generate_uuid()?,
                level_id: import.id,
                z: layer.z,
                width_px: layer.width_px,
                height_px: layer.height_px,
                parallax_x: layer.parallax_x,
                parallax_y: layer.parallax_y,
                encoding: layer.encoding,
                content_hash: layer.content_hash,
                data: layer.data,
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
            .ok_or_else(|| ServiceError::not_found(format!("unknown level '{level_id}'")))?;
        if !level.active {
            return Err(ServiceError::conflict(format!("level '{}' is not active", level.slug)));
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
