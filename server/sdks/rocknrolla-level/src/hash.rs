//! Deterministic layer content hashing.

/// FNV-1a 64-bit hash of the pixel dimensions and the raw layer bytes.
/// Rendered as 16 lowercase hex characters.
pub fn content_hash(width_px: u32, height_px: u32, data: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    let mut eat = |byte: u8| {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    };
    for byte in width_px.to_le_bytes() {
        eat(byte);
    }
    for byte in height_px.to_le_bytes() {
        eat(byte);
    }
    for &byte in data {
        eat(byte);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_stable_and_dimension_sensitive() {
        let data = b"<svg></svg>".to_vec();
        assert_eq!(content_hash(256, 128, &data), content_hash(256, 128, &data.clone()));
        assert_ne!(content_hash(256, 128, &data), content_hash(128, 256, &data));
    }
}
