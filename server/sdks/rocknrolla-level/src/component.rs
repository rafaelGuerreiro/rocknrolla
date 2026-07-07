//! Component facts and validation for the SVG component library.
//!
//! A component's `data` is one standalone SVG document: visible art plus a
//! hidden group of collider markers (`data-t="<tile id>"` rects/polygons)
//! in component-local coordinates. Validation is substring-based — no XML
//! dependency — so the reducer stays dependency-free and deterministic.

use crate::hash::content_hash;
use rocknrolla_error::{ServiceError, ServiceResult};

/// Upper bound on one component document; generated components are tens of KB.
pub const MAX_SVG_BYTES: usize = 512 * 1024;

/// The component facts every importer and the module validate identically.
#[derive(Debug, Clone)]
pub struct ComponentFacts {
    pub slug: String,
    pub width_px: u32,
    pub height_px: u32,
    pub content_hash: String,
    pub data: Vec<u8>,
}

/// Validate one component: non-zero natural size, size cap, standalone
/// `<svg>` document, and a matching content hash.
pub fn validate_component(component: &ComponentFacts) -> ServiceResult<()> {
    let slug = &component.slug;
    if component.width_px == 0 || component.height_px == 0 {
        return Err(ServiceError::validation(format!("component '{slug}': zero pixel dimensions")));
    }
    if component.data.len() > MAX_SVG_BYTES {
        return Err(ServiceError::validation(format!(
            "component '{slug}': SVG document exceeds {MAX_SVG_BYTES} bytes"
        )));
    }
    let svg = std::str::from_utf8(&component.data)
        .map_err(|_| ServiceError::validation(format!("component '{slug}': SVG is not valid UTF-8")))?;
    let trimmed = svg.trim();
    if !trimmed.starts_with("<svg") || !trimmed.ends_with("</svg>") {
        return Err(ServiceError::validation(format!(
            "component '{slug}': data is not a standalone <svg> document"
        )));
    }
    let hash = content_hash(component.width_px, component.height_px, &component.data);
    if hash != component.content_hash {
        return Err(ServiceError::validation(format!(
            "component '{slug}': content hash mismatch (computed {hash}, declared {})",
            component.content_hash
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ComponentFacts {
        let data = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"64\"></svg>".to_vec();
        ComponentFacts {
            slug: "ground-flat".to_string(),
            width_px: 128,
            height_px: 64,
            content_hash: content_hash(128, 64, &data),
            data,
        }
    }

    #[test]
    fn accepts_a_valid_component() {
        assert!(validate_component(&sample()).is_ok());
    }

    #[test]
    fn rejects_zero_dims_non_svg_and_hash_mismatch() {
        let mut component = sample();
        component.width_px = 0;
        assert!(validate_component(&component).unwrap_err().to_string().contains("zero pixel"));

        let mut component = sample();
        component.data = b"not svg".to_vec();
        component.content_hash = content_hash(128, 64, &component.data);
        let err = validate_component(&component).unwrap_err().to_string();
        assert!(err.contains("standalone <svg>"), "{err}");

        let mut component = sample();
        component.content_hash = "deadbeefdeadbeef".to_string();
        assert!(
            validate_component(&component)
                .unwrap_err()
                .to_string()
                .contains("hash mismatch")
        );
    }

    #[test]
    fn rejects_oversized_documents() {
        let mut component = sample();
        let mut data = b"<svg ".to_vec();
        data.resize(MAX_SVG_BYTES, b' ');
        data.extend_from_slice(b"</svg>");
        component.content_hash = content_hash(128, 64, &data);
        component.data = data;
        assert!(validate_component(&component).unwrap_err().to_string().contains("exceeds"));
    }
}
