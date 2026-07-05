//! The `rle-v1` layer codec: identifier, error type, encoder, and decoder.

/// Identifier stored next to compressed layer bytes.
pub const ENCODING_RLE_V1: &str = "rle-v1";

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::tile;

    fn grid(width: u16, height: u16, fill: u8) -> Vec<u8> {
        vec![fill; width as usize * height as usize]
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
}
