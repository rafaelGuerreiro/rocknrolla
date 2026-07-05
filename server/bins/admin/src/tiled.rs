//! Tiled JSON parsing and validation for the RocknRolla level importer.

use rocknrolla_level::{
    ENCODING_RLE_V1, GAMEPLAY_Z, LayerFacts, content_hash, rle_encode, tile, validate_layers,
};
use serde::Deserialize;

const FLIP_FLAGS: u32 = 0xf000_0000;

#[derive(Deserialize)]
struct TiledMap {
    #[serde(rename = "type")]
    kind: String,
    orientation: String,
    #[serde(default)]
    infinite: bool,
    tilewidth: u16,
    tileheight: u16,
    #[serde(default)]
    properties: Vec<TiledProperty>,
    #[serde(default)]
    tilesets: Vec<TiledTileset>,
    layers: Vec<TiledLayer>,
}

#[derive(Deserialize)]
struct TiledProperty {
    name: String,
    #[serde(default)]
    value: serde_json::Value,
}

#[derive(Deserialize)]
struct TiledTileset {
    firstgid: u32,
}

#[derive(Deserialize)]
struct TiledLayer {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    #[serde(default)]
    width: u16,
    #[serde(default)]
    height: u16,
    #[serde(default)]
    data: Vec<u32>,
    #[serde(default = "one")]
    parallaxx: f32,
    #[serde(default = "one")]
    parallaxy: f32,
    #[serde(default)]
    properties: Vec<TiledProperty>,
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

fn prop<'a>(props: &'a [TiledProperty], name: &str) -> Option<&'a serde_json::Value> {
    props.iter().find(|p| p.name == name).map(|p| &p.value)
}

fn string_prop(props: &[TiledProperty], name: &str) -> Result<Option<String>, String> {
    match prop(props, name) {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(format!("property '{name}' must be a string, got {other}")),
    }
}

fn bool_prop(props: &[TiledProperty], name: &str) -> Result<Option<bool>, String> {
    match prop(props, name) {
        None => Ok(None),
        Some(serde_json::Value::Bool(b)) => Ok(Some(*b)),
        Some(other) => Err(format!("property '{name}' must be a bool, got {other}")),
    }
}

fn u16_prop(props: &[TiledProperty], name: &str) -> Result<Option<u16>, String> {
    match prop(props, name) {
        None => Ok(None),
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .map(Some)
            .ok_or_else(|| format!("property '{name}' out of range")),
        Some(other) => Err(format!("property '{name}' must be an integer, got {other}")),
    }
}

/// Parse and validate one Tiled JSON document into an importable level.
pub fn parse_level(source: &str) -> Result<ImportedLevel, String> {
    let map: TiledMap = serde_json::from_str(source).map_err(|e| format!("invalid JSON: {e}"))?;
    if map.kind != "map" {
        return Err(format!("expected a Tiled map, got type '{}'", map.kind));
    }
    if map.orientation != "orthogonal" {
        return Err(format!("unsupported orientation '{}'", map.orientation));
    }
    if map.infinite {
        return Err("infinite maps are not supported".to_string());
    }
    let firstgid = match map.tilesets.as_slice() {
        [only] => only.firstgid,
        [] => return Err("map has no tileset".to_string()),
        _ => return Err("map must use exactly one tileset".to_string()),
    };

    let id = string_prop(&map.properties, "id")?
        .filter(|s| !s.is_empty())
        .ok_or("map is missing the 'id' string property")?;
    crate::uuid::validate_uuid(&id, "level id")?;
    let slug = string_prop(&map.properties, "slug")?
        .filter(|s| !s.is_empty())
        .ok_or("map is missing the 'slug' string property")?;
    let name = string_prop(&map.properties, "name")?
        .filter(|s| !s.is_empty())
        .ok_or("map is missing the 'name' string property")?;
    let is_starting = bool_prop(&map.properties, "starting")?.unwrap_or(false);
    let active = bool_prop(&map.properties, "active")?.unwrap_or(true);
    let reward_lootbox_id = string_prop(&map.properties, "reward_lootbox_id")?
        .filter(|s| !s.is_empty())
        .map(|reward| crate::uuid::validate_uuid(&reward, "reward_lootbox_id").map(|()| reward))
        .transpose()?;
    let successors: Vec<String> = string_prop(&map.properties, "successors")?
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let mut seen_successors = std::collections::HashSet::new();
    for successor in &successors {
        crate::uuid::validate_uuid(successor, "successor")?;
        if successor.eq_ignore_ascii_case(&id) {
            return Err(format!("level '{slug}' lists itself as a successor"));
        }
        if !seen_successors.insert(successor.to_lowercase()) {
            return Err(format!(
                "level '{slug}' lists successor '{successor}' twice"
            ));
        }
    }

    let mut layers = Vec::new();
    for layer in &map.layers {
        if layer.kind != "tilelayer" {
            return Err(format!(
                "layer '{}' has unsupported type '{}'; only tile layers are allowed",
                layer.name, layer.kind
            ));
        }
        let z_value = u16_prop(&layer.properties, "z")?
            .ok_or_else(|| format!("layer '{}' is missing the 'z' int property", layer.name))?;
        let z = u8::try_from(z_value)
            .map_err(|_| format!("layer '{}' z {z_value} is out of 0..=255", layer.name))?;
        if let Some(role) = string_prop(&layer.properties, "role")? {
            let expected = if z == GAMEPLAY_Z {
                "gameplay"
            } else {
                "visual"
            };
            if role != expected {
                return Err(format!(
                    "layer '{}' role '{role}' contradicts z {z} (expected '{expected}')",
                    layer.name
                ));
            }
        }
        if layer.width == 0 || layer.height == 0 {
            return Err(format!("layer '{}' has zero dimensions", layer.name));
        }
        let expected_len = layer.width as usize * layer.height as usize;
        if layer.data.len() != expected_len {
            return Err(format!(
                "layer '{}' has {} tiles, expected {expected_len}",
                layer.name,
                layer.data.len()
            ));
        }
        let mut tiles = Vec::with_capacity(expected_len);
        for (index, &gid) in layer.data.iter().enumerate() {
            if gid & FLIP_FLAGS != 0 {
                return Err(format!(
                    "layer '{}' tile {index} uses unsupported flip/rotation flags",
                    layer.name
                ));
            }
            let tile_id = if gid == 0 {
                tile::EMPTY
            } else {
                let local = gid.checked_sub(firstgid).ok_or_else(|| {
                    format!(
                        "layer '{}' tile {index} gid {gid} below firstgid",
                        layer.name
                    )
                })?;
                u8::try_from(local)
                    .ok()
                    .filter(|&t| t <= tile::MAX)
                    .ok_or_else(|| {
                        format!(
                            "layer '{}' tile {index} has unknown tile id {local}",
                            layer.name
                        )
                    })?
            };
            tiles.push(tile_id);
        }
        let cell_width = u16_prop(&layer.properties, "cell_width")?.unwrap_or(map.tilewidth);
        let cell_height = u16_prop(&layer.properties, "cell_height")?.unwrap_or(map.tileheight);
        layers.push(LayerFacts {
            z,
            width: layer.width,
            height: layer.height,
            cell_width,
            cell_height,
            parallax_x: layer.parallaxx,
            parallax_y: layer.parallaxy,
            encoding: ENCODING_RLE_V1.to_string(),
            content_hash: content_hash(layer.width, layer.height, &tiles),
            data: rle_encode(&tiles),
        });
    }

    // Full shared validation: single gameplay layer at 127, unique z,
    // parallax rules, gameplay cell size, tile ids, and the RLE round trip
    // via decode + hash.
    validate_layers(&layers)?;

    Ok(ImportedLevel {
        id,
        slug,
        name,
        is_starting,
        active,
        reward_lootbox_id,
        successors,
        layers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL_ID: &str = "0195c8f1-0000-7000-8000-000000000001";
    const NEXT_ID: &str = "0195c8f1-0000-7000-8000-000000000002";
    const OTHER_ID: &str = "0195c8f1-0000-7000-8000-000000000003";

    fn sample(gameplay_z: u64, extra_flags: u32) -> String {
        let data: Vec<u32> = {
            let mut d = vec![0u32; 12];
            d[0] = 1 + tile::SPAWN as u32;
            d[4] = (1 + tile::SOLID as u32) | extra_flags;
            d[11] = 1 + tile::FINISH as u32;
            d
        };
        serde_json::json!({
            "type": "map", "orientation": "orthogonal", "infinite": false,
            "width": 4, "height": 3, "tilewidth": 64, "tileheight": 64,
            "tilesets": [{"firstgid": 1}],
            "properties": [
                {"name": "id", "type": "string", "value": LEVEL_ID},
                {"name": "slug", "type": "string", "value": "test-level"},
                {"name": "name", "type": "string", "value": "Test Level"},
                {"name": "starting", "type": "bool", "value": true},
                {"name": "successors", "type": "string",
                 "value": format!("{NEXT_ID}, {OTHER_ID}")}
            ],
            "layers": [{
                "type": "tilelayer", "name": "gameplay",
                "width": 4, "height": 3, "data": data,
                "properties": [{"name": "z", "type": "int", "value": gameplay_z}]
            }]
        })
        .to_string()
    }

    #[test]
    fn parses_a_valid_map() {
        let level = parse_level(&sample(127, 0)).unwrap();
        assert_eq!(level.id, LEVEL_ID);
        assert_eq!(level.slug, "test-level");
        assert!(level.is_starting);
        assert_eq!(level.reward_lootbox_id, None);
        assert_eq!(level.successors, vec![NEXT_ID, OTHER_ID]);
        assert_eq!(level.layers.len(), 1);
        assert_eq!(level.layers[0].z, GAMEPLAY_Z);
    }

    #[test]
    fn rejects_non_uuid_ids_and_references() {
        let err = parse_level(&sample(127, 0).replace(LEVEL_ID, "tutorial-hill")).unwrap_err();
        assert!(err.contains("not a valid UUID"), "{err}");
        let err = parse_level(&sample(127, 0).replace(NEXT_ID, "next-level")).unwrap_err();
        assert!(err.contains("not a valid UUID"), "{err}");
    }

    #[test]
    fn rejects_self_and_duplicate_successors() {
        let err = parse_level(&sample(127, 0).replace(NEXT_ID, LEVEL_ID)).unwrap_err();
        assert!(err.contains("itself"), "{err}");
        let err = parse_level(&sample(127, 0).replace(OTHER_ID, NEXT_ID)).unwrap_err();
        assert!(err.contains("twice"), "{err}");
    }

    #[test]
    fn rejects_flip_flags() {
        let err = parse_level(&sample(127, 0x8000_0000)).unwrap_err();
        assert!(err.contains("flip/rotation"), "{err}");
    }

    #[test]
    fn rejects_missing_gameplay_layer() {
        let err = parse_level(&sample(50, 0)).unwrap_err();
        assert!(err.contains("no gameplay layer"), "{err}");
    }
}
