//! Semantic tile IDs and shared gameplay-layer constants.
//!
//! Stored level data references these stable semantic IDs; art assets and
//! Tiled global IDs map onto them but never replace them.

/// Logical cell size in pixels for the gameplay layer, matching the native
/// Kenney `Default` tile size.
pub const GAMEPLAY_CELL_SIZE: u16 = 64;
/// The gameplay/collision layer always sits at this Z.
pub const GAMEPLAY_Z: u8 = 127;

pub mod tile {
    pub const EMPTY: u8 = 0;
    pub const SOLID: u8 = 1;
    /// Floor rises left-to-right (45 degrees).
    pub const SLOPE_UP: u8 = 2;
    /// Floor falls left-to-right (45 degrees).
    pub const SLOPE_DOWN: u8 = 3;
    pub const SPAWN: u8 = 4;
    pub const FINISH: u8 = 5;
    /// Lethal hazard; touching it fails the run.
    pub const LETHAL: u8 = 6;
    /// Water sensor; applies buoyancy while inside.
    pub const WATER: u8 = 7;
    /// Fire hazard; lethal unless character fire resistance meets the threshold.
    pub const FIRE: u8 = 8;
    /// Heavy pushable obstacle; only dense characters move it.
    pub const HEAVY: u8 = 9;
    /// Non-colliding decoration.
    pub const DECOR: u8 = 10;
    /// Highest tile ID the client catalog knows.
    pub const MAX: u8 = DECOR;
}
