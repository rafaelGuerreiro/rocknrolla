//! `LayerFacts` and whole-layer-set validation for `svg-v1` scene layers.
//!
//! A layer's `data` is one standalone SVG document: visible scene art plus,
//! on the gameplay layer, a hidden group of collider markers
//! (`data-t="<tile id>"` rects/polygons) the client turns into physics
//! bodies. The module validates bytes and markers without an XML parser —
//! substring checks keep the reducer dependency-free and deterministic.

use crate::{catalog::GAMEPLAY_Z, hash::content_hash};
use rocknrolla_error::{ServiceError, ServiceResult};

pub const ENCODING_SVG_V1: &str = "svg-v1";
/// Upper bound on one layer document; generated scenes are tens of KB.
pub const MAX_SVG_BYTES: usize = 512 * 1024;

/// The layer facts every importer and the module validate identically.
#[derive(Debug, Clone)]
pub struct LayerFacts {
    pub z: u8,
    pub width_px: u32,
    pub height_px: u32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub encoding: String,
    pub content_hash: String,
    pub data: Vec<u8>,
}

/// True when the SVG document contains a collider marker for the tile id.
fn has_marker(svg: &str, tile_id: u8) -> bool {
    svg.contains(&format!("data-t=\"{tile_id}\""))
}

/// Validate a whole level's layer set: exactly one gameplay layer at
/// `z = 127` with parallax (1.0, 1.0), unique Z values, well-formed SVG
/// documents, spawn/finish markers on the gameplay layer, and matching
/// content hashes.
pub fn validate_layers(layers: &[LayerFacts]) -> ServiceResult<()> {
    let mut seen_z = Vec::new();
    let mut gameplay_count = 0usize;
    for layer in layers {
        if seen_z.contains(&layer.z) {
            return Err(ServiceError::validation(format!("duplicate layer z {}", layer.z)));
        }
        seen_z.push(layer.z);
        if layer.encoding != ENCODING_SVG_V1 {
            return Err(ServiceError::validation(format!("unsupported encoding '{}'", layer.encoding)));
        }
        if layer.width_px == 0 || layer.height_px == 0 {
            return Err(ServiceError::validation(format!(
                "layer z {}: zero pixel dimensions",
                layer.z
            )));
        }
        if layer.data.len() > MAX_SVG_BYTES {
            return Err(ServiceError::validation(format!(
                "layer z {}: SVG document exceeds {MAX_SVG_BYTES} bytes",
                layer.z
            )));
        }
        let svg = std::str::from_utf8(&layer.data)
            .map_err(|_| ServiceError::validation(format!("layer z {}: SVG is not valid UTF-8", layer.z)))?;
        let trimmed = svg.trim();
        if !trimmed.starts_with("<svg") || !trimmed.ends_with("</svg>") {
            return Err(ServiceError::validation(format!(
                "layer z {}: data is not a standalone <svg> document",
                layer.z
            )));
        }
        let hash = content_hash(layer.width_px, layer.height_px, &layer.data);
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
            if !has_marker(svg, crate::catalog::tile::SPAWN) {
                return Err(ServiceError::validation("gameplay layer has no spawn marker"));
            }
            if !has_marker(svg, crate::catalog::tile::FINISH) {
                return Err(ServiceError::validation("gameplay layer has no finish marker"));
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
    use crate::catalog::tile;

    fn gameplay_svg() -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"256\" height=\"128\">\
             <rect x=\"0\" y=\"64\" width=\"256\" height=\"64\" fill=\"#b06038\"/>\
             <g visibility=\"hidden\">\
             <rect data-t=\"{}\" x=\"0\" y=\"0\" width=\"64\" height=\"64\"/>\
             <rect data-t=\"{}\" x=\"192\" y=\"0\" width=\"64\" height=\"64\"/>\
             </g></svg>",
            tile::SPAWN,
            tile::FINISH
        )
    }

    fn gameplay_layer() -> LayerFacts {
        let data = gameplay_svg().into_bytes();
        LayerFacts {
            z: GAMEPLAY_Z,
            width_px: 256,
            height_px: 128,
            parallax_x: 1.0,
            parallax_y: 1.0,
            encoding: ENCODING_SVG_V1.to_string(),
            content_hash: content_hash(256, 128, &data),
            data,
        }
    }

    #[test]
    fn accepts_valid_layer_set() {
        let backdrop_data = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"64\"></svg>".to_vec();
        let backdrop = LayerFacts {
            z: 20,
            width_px: 128,
            height_px: 64,
            parallax_x: 0.4,
            parallax_y: 0.9,
            encoding: ENCODING_SVG_V1.to_string(),
            content_hash: content_hash(128, 64, &backdrop_data),
            data: backdrop_data,
        };
        assert!(validate_layers(&[backdrop, gameplay_layer()]).is_ok());
    }

    #[test]
    fn rejects_missing_duplicate_or_misplaced_gameplay_layer() {
        assert!(validate_layers(&[]).unwrap_err().to_string().contains("no gameplay layer"));

        let mut misplaced = gameplay_layer();
        misplaced.z = 126;
        let err = validate_layers(&[misplaced]).unwrap_err().to_string();
        assert!(err.contains("no gameplay layer"), "{err}");

        let err = validate_layers(&[gameplay_layer(), gameplay_layer()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate layer z"), "{err}");
    }

    #[test]
    fn rejects_bad_parallax_missing_markers_and_non_svg() {
        let mut layer = gameplay_layer();
        layer.parallax_x = 0.5;
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("parallax"));

        let data = gameplay_svg().replace("data-t=\"4\"", "data-t=\"10\"").into_bytes();
        let mut layer = gameplay_layer();
        layer.content_hash = content_hash(256, 128, &data);
        layer.data = data;
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("spawn marker"));

        let mut layer = gameplay_layer();
        layer.data = b"not svg".to_vec();
        layer.content_hash = content_hash(256, 128, &layer.data);
        let err = validate_layers(&[layer]).unwrap_err().to_string();
        assert!(err.contains("standalone <svg>"), "{err}");
    }

    #[test]
    fn rejects_content_hash_mismatch_and_zero_dims() {
        let mut layer = gameplay_layer();
        layer.content_hash = "deadbeefdeadbeef".to_string();
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("hash mismatch"));

        let mut layer = gameplay_layer();
        layer.width_px = 0;
        assert!(validate_layers(&[layer]).unwrap_err().to_string().contains("zero pixel"));
    }
}
