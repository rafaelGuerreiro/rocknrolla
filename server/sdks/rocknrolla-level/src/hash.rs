//! Deterministic layer content hashing.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::tile;

    #[test]
    fn content_hash_is_stable_and_dimension_sensitive() {
        let tiles = vec![tile::SOLID; 8];
        assert_eq!(content_hash(4, 2, &tiles), content_hash(4, 2, &tiles.clone()));
        assert_ne!(content_hash(4, 2, &tiles), content_hash(2, 4, &tiles));
    }
}
