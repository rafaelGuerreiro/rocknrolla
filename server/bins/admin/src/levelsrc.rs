//! Authored level source parsing for the RocknRolla level importer.
//!
//! Levels are committed as compact JSON documents (`levels/src/*.json`)
//! whose layers are ASCII tile grids. The importer renders each layer into
//! a standalone `svg-v1` scene document (see [`crate::svggen`]) so the
//! database stores the exact bytes the client draws.

use anyhow::{Context, Result, bail};
use rocknrolla_level::{GAMEPLAY_CELL_SIZE, GAMEPLAY_Z, LayerFacts, tile, validate_layers};
use serde::Deserialize;

#[derive(Deserialize)]
struct SourceLevel {
    id: String,
    slug: String,
    name: String,
    #[serde(default)]
    starting: bool,
    #[serde(default = "default_true")]
    active: bool,
    #[serde(default)]
    reward_lootbox_id: Option<String>,
    #[serde(default)]
    successors: Vec<String>,
    layers: Vec<SourceLayer>,
}

#[derive(Deserialize)]
struct SourceLayer {
    z: u8,
    /// Cell size in pixels; gameplay layers must use the default 64.
    #[serde(default = "default_cell")]
    cell: u32,
    #[serde(default = "one")]
    parallax_x: f32,
    #[serde(default = "one")]
    parallax_y: f32,
    rows: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_cell() -> u32 {
    GAMEPLAY_CELL_SIZE as u32
}

fn one() -> f32 {
    1.0
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
    pub layers: Vec<LayerFacts>,
}

/// Map one authoring character onto a semantic tile id.
fn tile_for(ch: char) -> Result<u8> {
    Ok(match ch {
        '.' => tile::EMPTY,
        '#' => tile::SOLID,
        '/' => tile::SLOPE_UP,
        '\\' => tile::SLOPE_DOWN,
        'S' => tile::SPAWN,
        'F' => tile::FINISH,
        '^' => tile::LETHAL,
        '~' => tile::WATER,
        'f' => tile::FIRE,
        'H' => tile::HEAVY,
        'd' => tile::DECOR,
        other => bail!("unknown tile character '{other}'"),
    })
}

/// Parse the ASCII rows of one layer into a row-major tile grid.
fn parse_grid(layer: &SourceLayer) -> Result<(Vec<u8>, u32, u32)> {
    let rows = layer.rows.len() as u32;
    if rows == 0 {
        bail!("layer z {} has no rows", layer.z);
    }
    let cols = layer.rows[0].chars().count() as u32;
    if cols == 0 {
        bail!("layer z {} has empty rows", layer.z);
    }
    let mut tiles = Vec::with_capacity((cols * rows) as usize);
    for (y, row) in layer.rows.iter().enumerate() {
        if row.chars().count() as u32 != cols {
            bail!(
                "layer z {} row {y} has {} tiles, expected {cols}",
                layer.z,
                row.chars().count()
            );
        }
        for (x, ch) in row.chars().enumerate() {
            let id = tile_for(ch).with_context(|| format!("layer z {} row {y} column {x}", layer.z))?;
            tiles.push(id);
        }
    }
    Ok((tiles, cols, rows))
}

/// Parse and validate one authored level document into an importable level.
pub fn parse_level(source: &str) -> Result<ImportedLevel> {
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

    let mut layers = Vec::new();
    for layer in &level.layers {
        let (tiles, cols, rows) = parse_grid(layer)?;
        if layer.z == GAMEPLAY_Z && layer.cell != GAMEPLAY_CELL_SIZE as u32 {
            bail!("gameplay layer cell size must be {GAMEPLAY_CELL_SIZE}");
        }
        if layer.cell == 0 {
            bail!("layer z {} has zero cell size", layer.z);
        }
        layers.push(crate::svggen::render_layer(&crate::svggen::LayerScene {
            z: layer.z,
            parallax_x: layer.parallax_x,
            parallax_y: layer.parallax_y,
            cell: layer.cell,
            cols,
            rows,
            tiles,
        }));
    }

    // Full shared validation: single gameplay layer at 127, unique z,
    // parallax rules, spawn/finish markers, and content hashes.
    validate_layers(&layers)?;

    Ok(ImportedLevel {
        id: level.id,
        slug: level.slug,
        name: level.name,
        is_starting: level.starting,
        active: level.active,
        reward_lootbox_id,
        successors: level.successors,
        layers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL_ID: &str = "0195c8f1-0000-7000-8000-000000000001";
    const NEXT_ID: &str = "0195c8f1-0000-7000-8000-000000000002";

    fn sample(gameplay_z: u8) -> String {
        serde_json::json!({
            "id": LEVEL_ID,
            "slug": "test-level",
            "name": "Test Level",
            "starting": true,
            "successors": [NEXT_ID],
            "layers": [{
                "z": gameplay_z,
                "rows": [
                    "S..^..F",
                    "##/~\\##"
                ]
            }]
        })
        .to_string()
    }

    #[test]
    fn parses_a_valid_level() {
        let level = parse_level(&sample(127)).unwrap();
        assert_eq!(level.id, LEVEL_ID);
        assert_eq!(level.slug, "test-level");
        assert!(level.is_starting);
        assert_eq!(level.successors, vec![NEXT_ID]);
        assert_eq!(level.layers.len(), 1);
        let layer = &level.layers[0];
        assert_eq!(layer.z, 127);
        assert_eq!(layer.width_px, 7 * 64);
        assert_eq!(layer.height_px, 2 * 64);
        let svg = std::str::from_utf8(&layer.data).unwrap();
        assert!(svg.contains("data-t=\"4\""), "spawn marker missing");
        assert!(svg.contains("data-t=\"5\""), "finish marker missing");
        assert!(svg.contains("data-t=\"2\""), "slope marker missing");
    }

    #[test]
    fn rejects_bad_grids_and_ids() {
        let err = parse_level(&sample(127).replace(LEVEL_ID, "tutorial-hill"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a valid UUID"), "{err}");

        let err = parse_level(&sample(127).replace("S..^..F", "S..^.."))
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected"), "{err}");

        let err = format!("{:#}", parse_level(&sample(127).replace('^', "X")).unwrap_err());
        assert!(err.contains("unknown tile character"), "{err}");
    }

    #[test]
    fn rejects_missing_gameplay_layer() {
        let err = parse_level(&sample(50)).unwrap_err().to_string();
        assert!(err.contains("no gameplay layer"), "{err}");
    }

    #[test]
    fn rejects_self_successor() {
        let err = parse_level(&sample(127).replace(NEXT_ID, LEVEL_ID)).unwrap_err().to_string();
        assert!(err.contains("itself"), "{err}");
    }
}
