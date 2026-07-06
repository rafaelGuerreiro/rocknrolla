//! `LayerFacts` and whole-layer-set validation.

use crate::{
    catalog::{GAMEPLAY_CELL_SIZE, GAMEPLAY_Z, tile},
    hash::content_hash,
    rle::{ENCODING_RLE_V1, rle_decode},
};
use rocknrolla_error::{ServiceError, ServiceResult};

/// The layer facts every importer and the module validate identically.
#[derive(Debug, Clone)]
pub struct LayerFacts {
    pub z: u8,
    pub width: u16,
    pub height: u16,
    pub cell_width: u16,
    pub cell_height: u16,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

/// Validate a whole level's layer set: exactly one gameplay layer at
/// `z = 127` with parallax (1.0, 1.0) and the fixed cell size, unique Z
/// values, known encodings, valid tile IDs, and matching content hashes.
pub fn validate_layers(layers: &[LayerFacts]) -> ServiceResult<()> {
    let mut seen_z = Vec::new();
    let mut gameplay_count = 0usize;
    for layer in layers {
        if seen_z.contains(&layer.z) {
            return Err(ServiceError::validation(format!("duplicate layer z {}", layer.z)));
        }
        seen_z.push(layer.z);
        if layer.encoding != ENCODING_RLE_V1 {
            return Err(ServiceError::validation(format!("unsupported encoding '{}'", layer.encoding)));
        }
        let tiles = rle_decode(&layer.data, layer.width, layer.height)
            .map_err(|e| ServiceError::validation(format!("layer z {}: {e}", layer.z)))?;
        if let Some(&bad) = tiles.iter().find(|&&t| t > tile::MAX) {
            return Err(ServiceError::validation(format!(
                "layer z {}: unknown tile id {bad}",
                layer.z
            )));
        }
        let hash = content_hash(layer.width, layer.height, &tiles);
        if hash != layer.content_hash {
            return Err(ServiceError::validation(format!(
                "layer z {}: content hash mismatch (computed {hash}, declared {})",
                layer.z, layer.content_hash
            )));
        }
        if layer.z == GAMEPLAY_Z {
            gameplay_count += 1;
            if layer.parallax_x != 1.0 || layer.parallax_y != 1.0 {
                return Err(ServiceError::validation("gameplay layer parallax must be (1.0, 1.0)"));
            }
            if layer.cell_width != GAMEPLAY_CELL_SIZE || layer.cell_height != GAMEPLAY_CELL_SIZE {
                return Err(ServiceError::validation(format!(
                    "gameplay layer cell size must be {GAMEPLAY_CELL_SIZE}"
                )));
            }
            if !tiles.contains(&tile::SPAWN) {
                return Err(ServiceError::validation("gameplay layer has no spawn tile"));
            }
            if !tiles.contains(&tile::FINISH) {
                return Err(ServiceError::validation("gameplay layer has no finish tile"));
            }
        }
    }
    match gameplay_count {
        0 => Err(ServiceError::validation(format!("no gameplay layer at z {GAMEPLAY_Z}"))),
        1 => Ok(()),
        n => Err(ServiceError::validation(format!("{n} gameplay layers; exactly one allowed"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rle::rle_encode;

    fn grid(width: u16, height: u16, fill: u8) -> Vec<u8> {
        vec![fill; width as usize * height as usize]
    }

    fn gameplay_layer(tiles: Vec<u8>, width: u16, height: u16) -> LayerFacts {
        LayerFacts {
            z: GAMEPLAY_Z,
            width,
            height,
            cell_width: GAMEPLAY_CELL_SIZE,
            cell_height: GAMEPLAY_CELL_SIZE,
            parallax_x: 1.0,
            parallax_y: 1.0,
            encoding: ENCODING_RLE_V1.to_string(),
            content_hash: content_hash(width, height, &tiles),
            data: rle_encode(&tiles),
        }
    }

    fn valid_gameplay_tiles(width: u16, height: u16) -> Vec<u8> {
        let mut tiles = grid(width, height, tile::EMPTY);
        tiles[0] = tile::SPAWN;
        let last = tiles.len() - 1;
        tiles[last] = tile::FINISH;
        tiles
    }

    #[test]
    fn accepts_valid_layer_set() {
        let width = 6;
        let height = 3;
        let gameplay = gameplay_layer(valid_gameplay_tiles(width, height), width, height);
        let backdrop_tiles = grid(3, 2, tile::DECOR);
        let backdrop = LayerFacts {
            z: 20,
            width: 3,
            height: 2,
            cell_width: 128,
            cell_height: 128,
            parallax_x: 0.4,
            parallax_y: 0.9,
            encoding: ENCODING_RLE_V1.to_string(),
            content_hash: content_hash(3, 2, &backdrop_tiles),
            data: rle_encode(&backdrop_tiles),
        };
        assert!(validate_layers(&[backdrop, gameplay]).is_ok());
    }

    #[test]
    fn rejects_missing_duplicate_or_misplaced_gameplay_layer() {
        let gameplay = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        assert!(validate_layers(&[]).unwrap_err().to_string().contains("no gameplay layer"));

        let mut misplaced = gameplay.clone();
        misplaced.z = 126;
        let err = validate_layers(&[misplaced]).unwrap_err().to_string();
        assert!(err.contains("no gameplay layer"), "{err}");

        let err = validate_layers(&[gameplay.clone(), gameplay]).unwrap_err().to_string();
        assert!(err.contains("duplicate layer z"), "{err}");
    }

    #[test]
    fn rejects_bad_gameplay_parallax_and_unknown_tiles() {
        let mut layer = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        layer.parallax_x = 0.5;
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("parallax"));

        let tiles = vec![tile::SPAWN, 200, tile::EMPTY, tile::FINISH];
        let bad = gameplay_layer(tiles, 4, 1);
        assert!(
            validate_layers(&[bad])
                .unwrap_err()
                .to_string()
                .contains("unknown tile id 200")
        );
    }

    #[test]
    fn rejects_wrong_gameplay_cell_size() {
        let mut layer = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        layer.cell_width = 32;
        layer.cell_height = 32;
        assert!(
            validate_layers(&[layer])
                .unwrap_err()
                .to_string()
                .contains("cell size must be 64")
        );
    }

    #[test]
    fn rejects_content_hash_mismatch() {
        let mut layer = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        layer.content_hash = "deadbeefdeadbeef".to_string();
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("hash mismatch"));
    }
}
