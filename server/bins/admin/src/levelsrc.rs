//! Authored level source parsing for the RocknRolla level importer.
//!
//! Levels are committed as compact JSON documents (`content/levels/*.json`):
//! a placement list over the component library, a level-owned spawn and
//! finish, and a required backdrop slug. Geometry is validated against the
//! loaded component files with the same shared checks the module applies
//! on import.

use anyhow::{Context, Result, bail};
use rocknrolla_level::{ComponentFacts, PlacementFacts, Vec2, Vec3, validate_level_geometry};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize)]
struct SourceLevel {
    id: String,
    slug: String,
    name: String,
    /// Required: the level's scenery theme, resolved against the authored
    /// backdrop library. No default — a missing assignment must fail.
    backdrop: String,
    #[serde(default)]
    starting: bool,
    #[serde(default = "default_true")]
    active: bool,
    #[serde(default)]
    reward_lootbox_id: Option<String>,
    #[serde(default)]
    successors: Vec<String>,
    spawn: SourcePoint,
    finish: SourcePoint,
    placements: Vec<SourcePlacement>,
}

#[derive(Deserialize, Clone, Copy)]
struct SourcePoint {
    x: u16,
    y: u16,
}

#[derive(Deserialize)]
struct SourcePlacement {
    component: String,
    x: u16,
    y: u16,
    #[serde(default)]
    z: i8,
    #[serde(default)]
    flip_x: bool,
    #[serde(default = "one")]
    scale: f32,
}

fn default_true() -> bool {
    true
}

fn one() -> f32 {
    1.0
}

/// One resolved placement ready for `import_level`.
#[derive(Debug)]
pub struct ImportedPlacement {
    pub component_slug: String,
    pub position: Vec3,
    pub flip_x: bool,
    pub scale: f32,
}

/// A fully validated level ready for `import_level`.
#[derive(Debug)]
pub struct ImportedLevel {
    /// Stable authored UUID.
    pub id: String,
    /// Readable operator/UI slug; never a foreign key.
    pub slug: String,
    pub name: String,
    pub is_starting: bool,
    pub active: bool,
    pub reward_lootbox_id: Option<String>,
    pub successors: Vec<String>,
    /// The backdrop's authored identity; the module resolves it to a UUID.
    pub backdrop_slug: String,
    pub spawn: Vec2,
    pub finish: Vec2,
    pub placements: Vec<ImportedPlacement>,
}

/// Parse and validate one authored level document against the component
/// library and the authored backdrop slugs.
pub fn parse_level(source: &str, components: &[ComponentFacts], backdrops: &HashSet<String>) -> Result<ImportedLevel> {
    let level: SourceLevel = serde_json::from_str(source).context("invalid JSON")?;
    if level.id.is_empty() {
        bail!("level is missing the 'id' property");
    }
    crate::uuid::validate_uuid(&level.id, "level id")?;
    if level.slug.is_empty() {
        bail!("level is missing the 'slug' property");
    }
    if level.name.is_empty() {
        bail!("level is missing the 'name' property");
    }
    if level.backdrop.is_empty() {
        bail!("level '{}' is missing the 'backdrop' property", level.slug);
    }
    if !backdrops.contains(&level.backdrop) {
        bail!("level '{}' references unknown backdrop '{}'", level.slug, level.backdrop);
    }
    let reward_lootbox_id = level
        .reward_lootbox_id
        .filter(|s| !s.is_empty())
        .map(|reward| crate::uuid::validate_uuid(&reward, "reward_lootbox_id").map(|()| reward))
        .transpose()?;
    let mut seen_successors = std::collections::HashSet::new();
    for successor in &level.successors {
        crate::uuid::validate_uuid(successor, "successor")?;
        if successor.eq_ignore_ascii_case(&level.id) {
            bail!("level '{}' lists itself as a successor", level.slug);
        }
        if !seen_successors.insert(successor.to_lowercase()) {
            bail!("level '{}' lists successor '{successor}' twice", level.slug);
        }
    }

    let sizes: HashMap<&str, (u32, u32)> = components
        .iter()
        .map(|c| (c.slug.as_str(), (c.width_px, c.height_px)))
        .collect();
    let mut placements = Vec::with_capacity(level.placements.len());
    let mut facts = Vec::with_capacity(level.placements.len());
    for placement in &level.placements {
        let Some(&(width_px, height_px)) = sizes.get(placement.component.as_str()) else {
            bail!("level '{}' places unknown component '{}'", level.slug, placement.component);
        };
        let position = Vec3 {
            x: placement.x,
            y: placement.y,
            z: placement.z,
        };
        facts.push(PlacementFacts {
            position,
            scale: placement.scale,
            component_width_px: width_px,
            component_height_px: height_px,
        });
        placements.push(ImportedPlacement {
            component_slug: placement.component.clone(),
            position,
            flip_x: placement.flip_x,
            scale: placement.scale,
        });
    }
    let spawn = Vec2 {
        x: level.spawn.x,
        y: level.spawn.y,
    };
    let finish = Vec2 {
        x: level.finish.x,
        y: level.finish.y,
    };
    validate_level_geometry(&facts, spawn, finish).map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(ImportedLevel {
        id: level.id,
        slug: level.slug,
        name: level.name,
        is_starting: level.starting,
        active: level.active,
        reward_lootbox_id,
        successors: level.successors,
        backdrop_slug: level.backdrop,
        spawn,
        finish,
        placements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL_ID: &str = "0195c8f1-0000-7000-8000-000000000001";
    const NEXT_ID: &str = "0195c8f1-0000-7000-8000-000000000002";

    fn components() -> Vec<ComponentFacts> {
        crate::svggen::starter_library()
    }

    fn backdrops() -> HashSet<String> {
        HashSet::from(["dusk".to_string()])
    }

    fn sample() -> String {
        serde_json::json!({
            "id": LEVEL_ID,
            "slug": "test-level",
            "name": "Test Level",
            "backdrop": "dusk",
            "starting": true,
            "successors": [NEXT_ID],
            "spawn": { "x": 64, "y": 32 },
            "finish": { "x": 900, "y": 80 },
            "placements": [
                { "component": "ground-flat", "x": 0, "y": 128 },
                { "component": "ground-flat", "x": 512, "y": 128 },
                { "component": "bush-cluster", "x": 100, "y": 64, "z": -40, "flip_x": true, "scale": 1.5 }
            ]
        })
        .to_string()
    }

    #[test]
    fn parses_a_valid_level() {
        let level = parse_level(&sample(), &components(), &backdrops()).unwrap();
        assert_eq!(level.id, LEVEL_ID);
        assert_eq!(level.slug, "test-level");
        assert!(level.is_starting);
        assert_eq!(level.successors, vec![NEXT_ID]);
        assert_eq!(level.backdrop_slug, "dusk");
        assert_eq!(level.spawn, Vec2 { x: 64, y: 32 });
        assert_eq!(level.placements.len(), 3);
        let decor = &level.placements[2];
        assert!(decor.flip_x);
        assert_eq!(decor.position.z, -40);
        assert_eq!(decor.scale, 1.5);
    }

    #[test]
    fn rejects_unknown_components_and_bad_ids() {
        let err = parse_level(&sample().replace(LEVEL_ID, "tutorial-hill"), &components(), &backdrops())
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a valid UUID"), "{err}");

        let err = parse_level(&sample().replace("ground-flat", "no-such"), &components(), &backdrops())
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown component"), "{err}");
    }

    #[test]
    fn rejects_missing_and_unknown_backdrops() {
        // Serde surfaces the missing required field inside the JSON context.
        let err = format!(
            "{:#}",
            parse_level(&sample().replace(r#""backdrop":"dusk","#, ""), &components(), &backdrops()).unwrap_err()
        );
        assert!(err.contains("backdrop"), "{err}");

        let err = parse_level(
            &sample().replace(r#""backdrop":"dusk""#, r#""backdrop":"noon""#),
            &components(),
            &backdrops(),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown backdrop 'noon'"), "{err}");
    }

    #[test]
    fn rejects_out_of_bounds_spawn_and_self_successor() {
        let err = parse_level(
            &sample().replace(r#""spawn":{"x":64,"y":32}"#, r#""spawn":{"x":5000,"y":32}"#),
            &components(),
            &backdrops(),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("spawn"), "{err}");

        let err = parse_level(&sample().replace(NEXT_ID, LEVEL_ID), &components(), &backdrops())
            .unwrap_err()
            .to_string();
        assert!(err.contains("itself"), "{err}");
    }
}
