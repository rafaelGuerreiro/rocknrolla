//! Authored character art loading and silhouette derivation.
//!
//! Character bodies are committed as standalone SVG files
//! (`content/characters/<style>.svg`); the filename is the character's
//! `style` from the seed and the root tag's `width`/`height` attributes are
//! the natural pixel size. The locked-card silhouette is never authored: it
//! is derived here at import time by injecting a flatten-to-ink color
//! filter, so silhouettes can never drift from bodies.

use crate::{componentsrc, seed::SeedCharacter};
use anyhow::{Context, Result, bail};
use rocknrolla_level::{ComponentFacts, content_hash};
use std::path::Path;

/// Flatten every color to dark ink while keeping the alpha silhouette.
/// Mirrors the client's original locked-card treatment.
const SILHOUETTE_FILTER: &str =
    r#"<filter id="sil"><feColorMatrix type="matrix" values="0 0 0 0 0.08 0 0 0 0 0.04 0 0 0 0 0.07 0 0 0 1 0"/></filter>"#;

/// One character's validated art payloads, ready for import.
#[derive(Debug)]
pub struct ImportedCharacterArt {
    pub style: String,
    /// The character's stable authored UUID from the seed.
    pub character_id: String,
    pub body: ComponentFacts,
    pub silhouette: ComponentFacts,
}

/// Derive the silhouette document from a body document by injecting the
/// ink filter into its `<defs>` and wrapping the art in a filtered group.
///
/// The string surgery requires the authored contract: exactly one plain
/// `<defs>` block (no attributes) before the art, and no nested `<svg>` —
/// anything else fails here instead of producing a corrupt document.
pub fn derive_silhouette(style: &str, body_svg: &str) -> Result<String> {
    if !body_svg.contains("<defs>") || !body_svg.contains("</defs>") {
        bail!("character '{style}': body SVG needs a plain <defs> block for silhouette derivation");
    }
    if !body_svg.trim_end().ends_with("</svg>") {
        bail!("character '{style}': body SVG is not a standalone document");
    }
    if body_svg.matches("</svg>").count() != 1 || body_svg.matches("</defs>").count() != 1 {
        bail!("character '{style}': body SVG must have exactly one <defs> block and no nested <svg>");
    }
    Ok(body_svg
        .replacen("<defs>", &format!("<defs>{SILHOUETTE_FILTER}"), 1)
        .replacen("</defs>", r#"</defs><g filter="url(#sil)">"#, 1)
        .replacen("</svg>", "</g></svg>", 1))
}

/// Pair loaded body files with their seed characters and derive each
/// silhouette. Every seed character must have exactly one art file and
/// every file must belong to a seed character — either gap fails loudly.
pub fn pair_art(bodies: Vec<ComponentFacts>, characters: &[SeedCharacter]) -> Result<Vec<ImportedCharacterArt>> {
    let mut art = Vec::with_capacity(bodies.len());
    for body in bodies {
        let style = body.slug.clone();
        let Some(character) = characters.iter().find(|c| c.style == style) else {
            bail!("character art '{style}.svg' matches no seed character style");
        };
        let silhouette_svg = derive_silhouette(&style, std::str::from_utf8(&body.data).expect("validated as UTF-8"))?;
        let data = silhouette_svg.into_bytes();
        let silhouette = ComponentFacts {
            slug: format!("{style}-silhouette"),
            width_px: body.width_px,
            height_px: body.height_px,
            content_hash: content_hash(body.width_px, body.height_px, &data),
            data,
        };
        art.push(ImportedCharacterArt {
            style,
            character_id: character.id.clone(),
            body,
            silhouette,
        });
    }
    for character in characters {
        if !art.iter().any(|a| a.style == character.style) {
            bail!(
                "seed character '{}' (style '{}') has no art file",
                character.name,
                character.style
            );
        }
    }
    Ok(art)
}

/// Load every `<style>.svg` in a directory and pair it with the seed.
pub fn load_character_art(dir: &Path, characters: &[SeedCharacter]) -> Result<Vec<ImportedCharacterArt>> {
    let bodies = componentsrc::load_components(dir).context("cannot load character art")?;
    pair_art(bodies, characters)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120"><defs><radialGradient id="g"/></defs><circle cx="60" cy="60" r="50" fill="url(#g)"/></svg>"#;

    fn seed_character(style: &str) -> SeedCharacter {
        SeedCharacter {
            id: "0195c8f1-0000-7000-8000-0000000000c1".to_string(),
            name: "Rocky".to_string(),
            style: style.to_string(),
            rarity_weight: 100,
            density: 0.004,
            jump_speed: 10.0,
            flight_time_ms: 300,
            buoyancy: 0.3,
            fire_resistance: 0.5,
            starter: true,
        }
    }

    fn body_facts(style: &str) -> ComponentFacts {
        componentsrc::parse_component(style, BODY).unwrap()
    }

    #[test]
    fn derives_a_filtered_silhouette() {
        let silhouette = derive_silhouette("rock", BODY).unwrap();
        assert!(silhouette.contains(r#"<filter id="sil">"#));
        assert!(silhouette.contains(r#"</defs><g filter="url(#sil)">"#));
        assert!(silhouette.trim_end().ends_with("</g></svg>"));
    }

    #[test]
    fn rejects_bodies_without_defs() {
        let bare = BODY.replace(r#"<defs><radialGradient id="g"/></defs>"#, "");
        let err = derive_silhouette("rock", &bare).unwrap_err().to_string();
        assert!(err.contains("<defs>"), "{err}");
    }

    #[test]
    fn rejects_nested_svg_and_multiple_defs() {
        let nested = BODY.replace("<circle", "<svg></svg><circle");
        let err = derive_silhouette("rock", &nested).unwrap_err().to_string();
        assert!(err.contains("nested"), "{err}");

        let doubled = BODY.replace("<circle", "<defs></defs><circle");
        let err = derive_silhouette("rock", &doubled).unwrap_err().to_string();
        assert!(err.contains("exactly one"), "{err}");
    }

    #[test]
    fn pairs_art_with_seed_styles_and_hashes_the_silhouette() {
        let art = pair_art(vec![body_facts("rock")], &[seed_character("rock")]).unwrap();
        assert_eq!(art.len(), 1);
        assert_eq!(art[0].style, "rock");
        assert_eq!(art[0].character_id, "0195c8f1-0000-7000-8000-0000000000c1");
        assert_eq!(art[0].silhouette.width_px, 120);
        assert_ne!(art[0].silhouette.content_hash, art[0].body.content_hash);
        assert_eq!(
            art[0].silhouette.content_hash,
            content_hash(120, 120, &art[0].silhouette.data)
        );
    }

    #[test]
    fn rejects_orphan_files_and_missing_art() {
        let err = pair_art(vec![body_facts("rock")], &[seed_character("pebble")])
            .unwrap_err()
            .to_string();
        assert!(err.contains("matches no seed character"), "{err}");

        let err = pair_art(
            vec![body_facts("pebble")],
            &[seed_character("pebble"), seed_character("rock")],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("has no art file"), "{err}");
    }
}
