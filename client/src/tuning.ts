/**
 * Gameplay feel — every knob in one place. Edit, save, and Vite reloads;
 * or tweak live in the browser console via `TUNING.<KNOB> = …` and hit the
 * HUD restart (↻) — values are applied when a run starts.
 *
 * Rules of thumb:
 * - Faster downhill: raise GRAVITY_Y, lower BODY_FRICTION_AIR.
 * - Lower/flatter jumps: raise GRAVITY_Y or lower JUMP_SCALE.
 * - Floatier hold-jump: raise LIFT_GRAVITY_FACTOR (0..1) — the fraction of
 *   gravity cancelled while the pointer is held (within flight time).
 */
export const TUNING = {
  /** World gravity. Higher = stronger slope acceleration, flatter jumps. */
  GRAVITY_Y: 1.7,

  /** Multiplies every character's server-side `jumpSpeed`. */
  JUMP_SCALE: 0.8,
  /** Fraction of gravity cancelled while holding a jump (0..1). */
  LIFT_GRAVITY_FACTOR: 0.82,
  /** Multiplies each character's server-side `flightTimeMs` hold budget. */
  FLIGHT_TIME_SCALE: 0.5,
  /** Upward velocity kept when releasing a jump early (0..1). */
  RELEASE_CUT: 0.45,
  /** Total jumps per airtime (2 = one double jump). */
  MAX_JUMPS: 2,
  /** Grace period after leaving ground where a jump still counts (ms). */
  COYOTE_MS: 50,
  /** How early a tap may land before touching ground and still jump (ms). */
  JUMP_BUFFER_MS: 50,

  /**
   * How strongly the body's spin follows its horizontal speed
   * (1 = wheel-perfect rolling, 0 = the body never visually rolls).
   * Contact friction is zero, so this is the only source of spin.
   */
  ROLL_SPIN: 1.0,
  /** Air drag per step: momentum bleed on flats and in flight. Keep tiny. */
  BODY_FRICTION_AIR: 0.0025,
  /** Bounciness on impact (0..1). */
  BODY_RESTITUTION: 0.08,

  /** Per-frame velocity retention inside water (1 = no drag). */
  WATER_DRAG: 0.995,

  /**
   * Parallax per depth unit: a plane at z scrolls at 1 + z × this
   * (clamped), so negative z drifts slower (background) and positive z
   * faster (foreground).
   */
  PARALLAX_PER_Z: 0.005,

  /** Landing speed that triggers the dust burst. */
  HARD_LANDING_SPEED: 9,
};

declare global {
  interface Window {
    /** Exposed for live feel-tuning from the browser console. */
    TUNING: typeof TUNING;
  }
}
// Guarded so pure-logic modules importing TUNING stay runnable under
// `node --test` (no DOM).
if (typeof window !== 'undefined') window.TUNING = TUNING;
