//! Shared geometric SpacetimeTypes for the RocknRolla module, admin shell,
//! and level validation.
//!
//! Unversioned by decision: geometric primitives that will never change
//! shape, a deliberate exception to the `_v1`/`V1` suffix rule. `x`/`y` are
//! world pixels (origin top-left, no negative offsets); `z` is signed depth
//! centered on the gameplay plane (negative = background, positive =
//! foreground).

use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    pub x: u16,
    pub y: u16,
}

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3 {
    pub x: u16,
    pub y: u16,
    pub z: i8,
}

impl Vec2 {
    pub fn with_z(self, z: i8) -> Vec3 {
        Vec3 { x: self.x, y: self.y, z }
    }
}

/// Drops the depth component.
impl From<Vec3> for Vec2 {
    fn from(v: Vec3) -> Self {
        Vec2 { x: v.x, y: v.y }
    }
}

impl From<(Vec2, i8)> for Vec3 {
    fn from((v, z): (Vec2, i8)) -> Self {
        v.with_z(z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_vec2_and_vec3() {
        let v3 = Vec2 { x: 3, y: 4 }.with_z(-7);
        assert_eq!(v3, Vec3 { x: 3, y: 4, z: -7 });
        assert_eq!(Vec2::from(v3), Vec2 { x: 3, y: 4 });
        assert_eq!(Vec3::from((Vec2 { x: 1, y: 2 }, 5)), Vec3 { x: 1, y: 2, z: 5 });
    }
}
