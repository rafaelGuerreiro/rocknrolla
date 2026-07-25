//! Authored component loading for the RocknRolla importer.
//!
//! Components are committed as standalone SVG files (`content/components/
//! <slug>.svg`); the filename is the slug and the root tag's `width`/
//! `height` attributes are the natural pixel size. Loading validates each
//! file with the same shared checks the module applies on import.

use anyhow::{Context, Result, bail};
use rocknrolla_level::{ComponentFacts, content_hash, validate_component};
use std::path::Path;

/// Extract an integer attribute from the root `<svg>` tag without an XML
/// dependency, mirroring the module's substring-based validation.
fn parse_px_attr(svg: &str, name: &str) -> Result<u32> {
    let end = svg.find('>').context("no root tag")?;
    let root = &svg[..end];
    let marker = format!(" {name}=\"");
    let start = root
        .find(&marker)
        .with_context(|| format!("root <svg> tag has no {name} attribute"))?
        + marker.len();
    let value = &root[start..];
    let quote = value.find('"').with_context(|| format!("unterminated {name} attribute"))?;
    value[..quote]
        .parse::<u32>()
        .with_context(|| format!("{name} attribute is not a whole pixel count"))
}

/// Load and validate one authored component file.
pub fn parse_component(slug: &str, source: &str) -> Result<ComponentFacts> {
    if slug.is_empty() {
        bail!("component file has an empty name");
    }
    let width_px = parse_px_attr(source, "width")?;
    let height_px = parse_px_attr(source, "height")?;
    let data = source.as_bytes().to_vec();
    let facts = ComponentFacts {
        slug: slug.to_string(),
        width_px,
        height_px,
        content_hash: content_hash(width_px, height_px, &data),
        data,
    };
    validate_component(&facts).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(facts)
}

/// Load every `*.svg` in a directory, sorted by filename.
pub fn load_components(dir: &Path) -> Result<Vec<ComponentFacts>> {
    if !dir.is_dir() {
        bail!("component path not found: {}", dir.display());
    }
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("cannot read directory {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "svg"))
        .collect();
    files.sort();
    if files.is_empty() {
        bail!("no SVG files found in {}", dir.display());
    }
    let mut components = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
        let slug = file
            .file_stem()
            .and_then(|s| s.to_str())
            .context("component filename is not UTF-8")?;
        let component = parse_component(slug, &source).with_context(|| file.display().to_string())?;
        components.push(component);
    }
    Ok(components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_generated_component() {
        let facts = crate::svggen::starter_library().into_iter().next().unwrap();
        let source = String::from_utf8(facts.data.clone()).unwrap();
        let parsed = parse_component(&facts.slug, &source).unwrap();
        assert_eq!(parsed.width_px, facts.width_px);
        assert_eq!(parsed.height_px, facts.height_px);
        assert_eq!(parsed.content_hash, facts.content_hash);
    }

    #[test]
    fn rejects_files_without_dimensions_or_svg_wrapper() {
        let err = parse_component("x", "<svg xmlns=\"a\"></svg>").unwrap_err().to_string();
        assert!(err.contains("width"), "{err}");

        let err = parse_component("x", "<div width=\"1\" height=\"1\"></div>")
            .unwrap_err()
            .to_string();
        assert!(err.contains("standalone <svg>"), "{err}");
    }
}
