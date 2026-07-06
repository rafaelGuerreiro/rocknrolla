/**
 * Texture key for a playable character's backend style key. Roller body
 * textures are generated from the design SVGs in `rollers.ts` and preloaded
 * by `BootScene` for every style the server reports.
 */
export function characterSpriteKey(style: string): string {
  return `roller_${style}`;
}
