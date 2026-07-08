//! Generic validation for standalone SVG content assets (components,
//! character bodies, silhouettes, faces, backdrop layers), shared by the
//! module and the admin importer. Substring-based — no XML dependency — so
//! reducers stay dependency-free and deterministic.

use crate::{component::MAX_SVG_BYTES, hash::content_hash};
use rocknrolla_error::{ServiceError, ServiceResult};

/// Validate one SVG asset: non-zero natural size, size cap, standalone
/// `<svg>` document, and a matching content hash. `what` names the asset
/// kind in error messages (e.g. "component", "face").
pub fn validate_svg_asset(
    what: &str,
    slug: &str,
    width_px: u32,
    height_px: u32,
    declared_hash: &str,
    data: &[u8],
) -> ServiceResult<()> {
    if width_px == 0 || height_px == 0 {
        return Err(ServiceError::validation(format!("{what} '{slug}': zero pixel dimensions")));
    }
    if data.len() > MAX_SVG_BYTES {
        return Err(ServiceError::validation(format!(
            "{what} '{slug}': SVG document exceeds {MAX_SVG_BYTES} bytes"
        )));
    }
    let svg =
        std::str::from_utf8(data).map_err(|_| ServiceError::validation(format!("{what} '{slug}': SVG is not valid UTF-8")))?;
    let trimmed = svg.trim();
    if !trimmed.starts_with("<svg") || !trimmed.ends_with("</svg>") {
        return Err(ServiceError::validation(format!(
            "{what} '{slug}': data is not a standalone <svg> document"
        )));
    }
    let hash = content_hash(width_px, height_px, data);
    if hash != declared_hash {
        return Err(ServiceError::validation(format!(
            "{what} '{slug}': content hash mismatch (computed {hash}, declared {declared_hash})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svg_bytes() -> Vec<u8> {
        b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80\" height=\"50\"></svg>".to_vec()
    }

    #[test]
    fn accepts_a_valid_asset() {
        let data = svg_bytes();
        let hash = content_hash(80, 50, &data);
        assert!(validate_svg_asset("face", "happy", 80, 50, &hash, &data).is_ok());
    }

    #[test]
    fn names_the_asset_kind_in_errors() {
        let data = svg_bytes();
        let hash = content_hash(80, 50, &data);
        let err = validate_svg_asset("face", "happy", 0, 50, &hash, &data)
            .unwrap_err()
            .to_string();
        assert!(err.contains("face 'happy'"), "{err}");

        let err = validate_svg_asset("backdrop layer", "dusk.sky", 80, 50, "deadbeefdeadbeef", &data)
            .unwrap_err()
            .to_string();
        assert!(err.contains("backdrop layer 'dusk.sky'"), "{err}");
        assert!(err.contains("hash mismatch"), "{err}");
    }
}
