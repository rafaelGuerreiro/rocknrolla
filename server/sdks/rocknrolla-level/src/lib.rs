//! Shared level encoding for the RocknRolla module, admin CLI, and tests.
//!
//! Tile catalog, the `rle-v1` layer codec, the FNV-1a 64 content hash, and
//! layer-set validation live here so the WASM module and the host CLI agree.

/// Logical cell size in pixels for the gameplay layer.
pub const GAMEPLAY_CELL_SIZE: u16 = 32;
/// The gameplay/collision layer always sits at this Z.
pub const GAMEPLAY_Z: u8 = 127;
/// Identifier stored next to compressed layer bytes.
pub const ENCODING_RLE_V1: &str = "rle-v1";

pub mod tile {
    pub const EMPTY: u8 = 0;
    pub const SOLID: u8 = 1;
    /// Floor rises left-to-right (45 degrees).
    pub const SLOPE_UP: u8 = 2;
    /// Floor falls left-to-right (45 degrees).
    pub const SLOPE_DOWN: u8 = 3;
    pub const SPAWN: u8 = 4;
    pub const FINISH: u8 = 5;
    /// Lethal hazard; touching it fails the run.
    pub const LETHAL: u8 = 6;
    /// Water sensor; applies buoyancy while inside.
    pub const WATER: u8 = 7;
    /// Fire hazard; lethal unless character fire resistance meets the threshold.
    pub const FIRE: u8 = 8;
    /// Heavy pushable obstacle; only dense characters move it.
    pub const HEAVY: u8 = 9;
    /// Non-colliding decoration.
    pub const DECOR: u8 = 10;
    /// Highest tile ID the client catalog knows.
    pub const MAX: u8 = DECOR;
}

#[derive(Debug, PartialEq)]
pub enum CodecError {
    ZeroRunLength { pair_index: usize },
    TrailingByte,
    LengthMismatch { decoded: usize, expected: usize },
    EmptyLayer,
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodecError::ZeroRunLength { pair_index } => {
                write!(f, "rle-v1 pair {pair_index} has zero run length")
            }
            CodecError::TrailingByte => write!(f, "rle-v1 data has a trailing unpaired byte"),
            CodecError::LengthMismatch { decoded, expected } => {
                write!(f, "rle-v1 decoded {decoded} tiles, expected {expected}")
            }
            CodecError::EmptyLayer => write!(f, "layer has zero width or height"),
        }
    }
}

/// Encode row-major tiles as `rle-v1`: repeated `[run_length, tile_id]` pairs.
/// Runs longer than 255 are split.
pub fn rle_encode(tiles: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut iter = tiles.iter().copied();
    let Some(mut current) = iter.next() else {
        return out;
    };
    let mut run: u32 = 1;
    for tile in iter {
        if tile == current && run < 255 {
            run += 1;
        } else {
            out.push(run as u8);
            out.push(current);
            current = tile;
            run = 1;
        }
    }
    out.push(run as u8);
    out.push(current);
    out
}

/// Decode `rle-v1` bytes, enforcing the exact `width * height` tile count.
pub fn rle_decode(data: &[u8], width: u16, height: u16) -> Result<Vec<u8>, CodecError> {
    if width == 0 || height == 0 {
        return Err(CodecError::EmptyLayer);
    }
    if !data.len().is_multiple_of(2) {
        return Err(CodecError::TrailingByte);
    }
    let expected = width as usize * height as usize;
    let mut tiles = Vec::with_capacity(expected);
    for (pair_index, pair) in data.chunks_exact(2).enumerate() {
        let (run, tile) = (pair[0], pair[1]);
        if run == 0 {
            return Err(CodecError::ZeroRunLength { pair_index });
        }
        if tiles.len() + run as usize > expected {
            return Err(CodecError::LengthMismatch {
                decoded: tiles.len() + run as usize,
                expected,
            });
        }
        tiles.extend(core::iter::repeat_n(tile, run as usize));
    }
    if tiles.len() != expected {
        return Err(CodecError::LengthMismatch {
            decoded: tiles.len(),
            expected,
        });
    }
    Ok(tiles)
}

/// FNV-1a 64-bit hash of width, height, and the decoded row-major tiles.
/// Rendered as 16 lowercase hex characters.
pub fn content_hash(width: u16, height: u16, tiles: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    let mut eat = |byte: u8| {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    };
    for byte in width.to_le_bytes() {
        eat(byte);
    }
    for byte in height.to_le_bytes() {
        eat(byte);
    }
    for &byte in tiles {
        eat(byte);
    }
    format!("{hash:016x}")
}

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
pub fn validate_layers(layers: &[LayerFacts]) -> Result<(), String> {
    let mut seen_z = Vec::new();
    let mut gameplay_count = 0usize;
    for layer in layers {
        if seen_z.contains(&layer.z) {
            return Err(format!("duplicate layer z {}", layer.z));
        }
        seen_z.push(layer.z);
        if layer.encoding != ENCODING_RLE_V1 {
            return Err(format!("unsupported encoding '{}'", layer.encoding));
        }
        let tiles = rle_decode(&layer.data, layer.width, layer.height)
            .map_err(|e| format!("layer z {}: {e}", layer.z))?;
        if let Some(&bad) = tiles.iter().find(|&&t| t > tile::MAX) {
            return Err(format!("layer z {}: unknown tile id {bad}", layer.z));
        }
        let hash = content_hash(layer.width, layer.height, &tiles);
        if hash != layer.content_hash {
            return Err(format!(
                "layer z {}: content hash mismatch (computed {hash}, declared {})",
                layer.z, layer.content_hash
            ));
        }
        if layer.z == GAMEPLAY_Z {
            gameplay_count += 1;
            if layer.parallax_x != 1.0 || layer.parallax_y != 1.0 {
                return Err("gameplay layer parallax must be (1.0, 1.0)".to_string());
            }
            if layer.cell_width != GAMEPLAY_CELL_SIZE || layer.cell_height != GAMEPLAY_CELL_SIZE {
                return Err(format!(
                    "gameplay layer cell size must be {GAMEPLAY_CELL_SIZE}"
                ));
            }
            if !tiles.contains(&tile::SPAWN) {
                return Err("gameplay layer has no spawn tile".to_string());
            }
            if !tiles.contains(&tile::FINISH) {
                return Err("gameplay layer has no finish tile".to_string());
            }
        }
    }
    match gameplay_count {
        0 => Err(format!("no gameplay layer at z {GAMEPLAY_Z}")),
        1 => Ok(()),
        n => Err(format!("{n} gameplay layers; exactly one allowed")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn round_trips_mixed_tiles() {
        let mut tiles = grid(8, 4, tile::EMPTY);
        tiles[3] = tile::SOLID;
        tiles[4] = tile::SOLID;
        tiles[9] = tile::WATER;
        tiles[31] = tile::FINISH;
        let encoded = rle_encode(&tiles);
        assert_eq!(rle_decode(&encoded, 8, 4).unwrap(), tiles);
    }

    #[test]
    fn splits_runs_longer_than_255() {
        let tiles = grid(20, 20, tile::SOLID); // 400 > 255
        let encoded = rle_encode(&tiles);
        assert_eq!(encoded, vec![255, tile::SOLID, 145, tile::SOLID]);
        assert_eq!(rle_decode(&encoded, 20, 20).unwrap(), tiles);
    }

    #[test]
    fn rejects_zero_run_length() {
        assert_eq!(
            rle_decode(&[0, tile::SOLID], 1, 1),
            Err(CodecError::ZeroRunLength { pair_index: 0 })
        );
    }

    #[test]
    fn rejects_trailing_byte() {
        assert_eq!(
            rle_decode(&[1, tile::SOLID, 9], 1, 1),
            Err(CodecError::TrailingByte)
        );
    }

    #[test]
    fn rejects_short_and_long_decodes() {
        assert_eq!(
            rle_decode(&[2, tile::SOLID], 2, 2),
            Err(CodecError::LengthMismatch {
                decoded: 2,
                expected: 4
            })
        );
        assert_eq!(
            rle_decode(&[5, tile::SOLID], 2, 2),
            Err(CodecError::LengthMismatch {
                decoded: 5,
                expected: 4
            })
        );
    }

    #[test]
    fn rejects_empty_dimensions() {
        assert_eq!(rle_decode(&[1, 0], 0, 4), Err(CodecError::EmptyLayer));
    }

    #[test]
    fn content_hash_is_stable_and_dimension_sensitive() {
        let tiles = grid(4, 2, tile::SOLID);
        assert_eq!(
            content_hash(4, 2, &tiles),
            content_hash(4, 2, &tiles.clone())
        );
        assert_ne!(content_hash(4, 2, &tiles), content_hash(2, 4, &tiles));
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
            cell_width: 64,
            cell_height: 64,
            parallax_x: 0.4,
            parallax_y: 0.9,
            encoding: ENCODING_RLE_V1.to_string(),
            content_hash: content_hash(3, 2, &backdrop_tiles),
            data: rle_encode(&backdrop_tiles),
        };
        assert_eq!(validate_layers(&[backdrop, gameplay]), Ok(()));
    }

    #[test]
    fn rejects_missing_duplicate_or_misplaced_gameplay_layer() {
        let gameplay = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        assert!(
            validate_layers(&[])
                .unwrap_err()
                .contains("no gameplay layer")
        );

        let mut misplaced = gameplay.clone();
        misplaced.z = 126;
        let err = validate_layers(&[misplaced]).unwrap_err();
        assert!(err.contains("no gameplay layer"), "{err}");

        let err = validate_layers(&[gameplay.clone(), gameplay]).unwrap_err();
        assert!(err.contains("duplicate layer z"), "{err}");
    }

    #[test]
    fn rejects_bad_gameplay_parallax_and_unknown_tiles() {
        let mut layer = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        layer.parallax_x = 0.5;
        assert!(validate_layers(&[layer]).unwrap_err().contains("parallax"));

        let tiles = vec![tile::SPAWN, 200, tile::EMPTY, tile::FINISH];
        let bad = gameplay_layer(tiles, 4, 1);
        assert!(
            validate_layers(&[bad])
                .unwrap_err()
                .contains("unknown tile id 200")
        );
    }

    #[test]
    fn rejects_content_hash_mismatch() {
        let mut layer = gameplay_layer(valid_gameplay_tiles(4, 2), 4, 2);
        layer.content_hash = "deadbeefdeadbeef".to_string();
        assert!(
            validate_layers(&[layer])
                .unwrap_err()
                .contains("hash mismatch")
        );
    }
}
