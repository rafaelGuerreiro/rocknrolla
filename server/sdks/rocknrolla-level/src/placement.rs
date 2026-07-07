//! Placement facts and whole-level geometry validation.
//!
//! A level is a flat list of component placements plus a level-owned spawn
//! and finish. Physics only exists on the gameplay plane (`z = 0`); other
//! depths are scenery. World bounds are derived from the gameplay-plane
//! extent, and spawn/finish must land inside them.

use rocknrolla_error::{ServiceError, ServiceResult};
use rocknrolla_geometry::{Vec2, Vec3};

/// The depth where physics happens; all other depths are scenery.
pub const GAMEPLAY_PLANE_Z: i8 = 0;
/// Uniform placement scale ceiling; keeps composed documents bounded.
pub const MAX_PLACEMENT_SCALE: f32 = 8.0;

/// The placement facts every importer and the module validate identically.
#[derive(Debug, Clone)]
pub struct PlacementFacts {
    pub position: Vec3,
    pub scale: f32,
    pub component_width_px: u32,
    pub component_height_px: u32,
}

/// Scaled world-pixel extent (right or bottom edge) of one placement axis.
fn extent(origin: u16, natural_px: u32, scale: f32) -> u32 {
    origin as u32 + (natural_px as f32 * scale).ceil() as u32
}

/// Validate a level's placements and spawn/finish, returning the world
/// `(width_px, height_px)` derived from the gameplay-plane extent.
pub fn validate_level_geometry(placements: &[PlacementFacts], spawn: Vec2, finish: Vec2) -> ServiceResult<(u32, u32)> {
    let mut width = 0u32;
    let mut height = 0u32;
    let mut gameplay = 0usize;
    for (index, placement) in placements.iter().enumerate() {
        if !placement.scale.is_finite() || placement.scale <= 0.0 || placement.scale > MAX_PLACEMENT_SCALE {
            return Err(ServiceError::validation(format!(
                "placement {index}: scale must be in (0, {MAX_PLACEMENT_SCALE}]"
            )));
        }
        let right = extent(placement.position.x, placement.component_width_px, placement.scale);
        let bottom = extent(placement.position.y, placement.component_height_px, placement.scale);
        if right > u16::MAX as u32 || bottom > u16::MAX as u32 {
            return Err(ServiceError::validation(format!(
                "placement {index}: extends past the {} world coordinate limit",
                u16::MAX
            )));
        }
        if placement.position.z == GAMEPLAY_PLANE_Z {
            gameplay += 1;
            width = width.max(right);
            height = height.max(bottom);
        }
    }
    if gameplay == 0 {
        return Err(ServiceError::validation("level has no gameplay-plane (z = 0) placement"));
    }
    for (name, point) in [("spawn", spawn), ("finish", finish)] {
        if point.x as u32 >= width || point.y as u32 >= height {
            return Err(ServiceError::validation(format!(
                "{name} ({}, {}) is outside the level bounds {width}x{height}",
                point.x, point.y
            )));
        }
    }
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v2(x: u16, y: u16) -> Vec2 {
        Vec2 { x, y }
    }

    fn ground(x: u16, y: u16, z: i8) -> PlacementFacts {
        PlacementFacts {
            position: Vec3 { x, y, z },
            scale: 1.0,
            component_width_px: 512,
            component_height_px: 128,
        }
    }

    #[test]
    fn derives_bounds_from_the_gameplay_plane_only() {
        let placements = [ground(0, 640, 0), ground(512, 640, 0), ground(2000, 2000, -40)];
        let (width, height) = validate_level_geometry(&placements, v2(64, 600), v2(900, 700)).unwrap();
        assert_eq!((width, height), (1024, 768));
    }

    #[test]
    fn scale_grows_the_extent() {
        let mut scaled = ground(0, 0, 0);
        scaled.scale = 1.5;
        let (width, height) = validate_level_geometry(&[scaled], v2(0, 0), v2(700, 100)).unwrap();
        assert_eq!((width, height), (768, 192));
    }

    #[test]
    fn rejects_missing_gameplay_plane_and_out_of_bounds_points() {
        let err = validate_level_geometry(&[ground(0, 0, -1)], v2(0, 0), v2(0, 0))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no gameplay-plane"), "{err}");

        let err = validate_level_geometry(&[ground(0, 0, 0)], v2(600, 0), v2(0, 0))
            .unwrap_err()
            .to_string();
        assert!(err.contains("spawn"), "{err}");

        let err = validate_level_geometry(&[ground(0, 0, 0)], v2(0, 0), v2(0, 500))
            .unwrap_err()
            .to_string();
        assert!(err.contains("finish"), "{err}");
    }

    #[test]
    fn rejects_bad_scales_and_world_overflow() {
        for scale in [0.0, -1.0, f32::NAN, MAX_PLACEMENT_SCALE + 0.1] {
            let mut placement = ground(0, 0, 0);
            placement.scale = scale;
            assert!(
                validate_level_geometry(&[placement], v2(0, 0), v2(0, 0)).is_err(),
                "accepted scale {scale}"
            );
        }

        let far = ground(65500, 0, 0);
        let err = validate_level_geometry(&[far], v2(0, 0), v2(0, 0)).unwrap_err().to_string();
        assert!(err.contains("coordinate limit"), "{err}");
    }
}
