//! Authored backdrop loading for the RocknRolla importer.
//!
//! A backdrop is three standalone SVG files in `content/backdrops/` named
//! `<slug>.<role>.svg` with roles exactly `sky`, `far`, and `mid`. Each
//! layer file follows the component file rules (root integer
//! `width`/`height`, standalone document, content hash).

use crate::componentsrc;
use anyhow::{Context, Result, bail};
use rocknrolla_level::ComponentFacts;
use std::{collections::BTreeMap, path::Path};

pub const ROLES: [&str; 3] = ["sky", "far", "mid"];

/// One backdrop's validated layers, ready for import.
#[derive(Debug)]
pub struct BackdropFacts {
    pub slug: String,
    pub sky: ComponentFacts,
    pub far: ComponentFacts,
    pub mid: ComponentFacts,
}

/// Group already-parsed layer files (slugged `<slug>.<role>`) into complete
/// backdrops, rejecting unknown roles, duplicates, and missing layers.
pub fn group_layers(layers: Vec<ComponentFacts>) -> Result<Vec<BackdropFacts>> {
    let mut grouped: BTreeMap<String, BTreeMap<String, ComponentFacts>> = BTreeMap::new();
    for layer in layers {
        let Some((slug, role)) = layer.slug.rsplit_once('.').map(|(s, r)| (s.to_string(), r.to_string())) else {
            bail!(
                "backdrop file '{}.svg' is not named '<slug>.<role>.svg' (roles: {})",
                layer.slug,
                ROLES.join(", ")
            );
        };
        if !ROLES.contains(&role.as_str()) {
            bail!(
                "backdrop '{slug}' has unknown layer role '{role}' (roles: {})",
                ROLES.join(", ")
            );
        }
        if grouped.entry(slug.clone()).or_default().insert(role.clone(), layer).is_some() {
            bail!("backdrop '{slug}' has duplicate layer role '{role}'");
        }
    }
    let mut backdrops = Vec::with_capacity(grouped.len());
    for (slug, mut roles) in grouped {
        for role in ROLES {
            if !roles.contains_key(role) {
                bail!("backdrop '{slug}' is missing layer '{role}'");
            }
        }
        backdrops.push(BackdropFacts {
            slug,
            sky: roles.remove("sky").expect("checked above"),
            far: roles.remove("far").expect("checked above"),
            mid: roles.remove("mid").expect("checked above"),
        });
    }
    Ok(backdrops)
}

/// Load every `<slug>.<role>.svg` in a directory as complete backdrops.
pub fn load_backdrops(dir: &Path) -> Result<Vec<BackdropFacts>> {
    let layers = componentsrc::load_components(dir).context("cannot load backdrops")?;
    group_layers(layers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(slug: &str) -> ComponentFacts {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="150"></svg>"#;
        componentsrc::parse_component(slug, svg).unwrap()
    }

    #[test]
    fn groups_a_complete_backdrop() {
        let backdrops = group_layers(vec![layer("dusk.sky"), layer("dusk.far"), layer("dusk.mid")]).unwrap();
        assert_eq!(backdrops.len(), 1);
        assert_eq!(backdrops[0].slug, "dusk");
        assert_eq!(backdrops[0].sky.width_px, 512);
    }

    #[test]
    fn rejects_missing_unknown_and_duplicate_roles() {
        let err = group_layers(vec![layer("dusk.sky"), layer("dusk.far")])
            .unwrap_err()
            .to_string();
        assert!(err.contains("missing layer 'mid'"), "{err}");

        let err = group_layers(vec![layer("dusk.stars")]).unwrap_err().to_string();
        assert!(err.contains("unknown layer role 'stars'"), "{err}");

        let err = group_layers(vec![layer("dusk.sky"), layer("dusk.sky")])
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate layer role 'sky'"), "{err}");

        let err = group_layers(vec![layer("plainname")]).unwrap_err().to_string();
        assert!(err.contains("not named"), "{err}");
    }
}
