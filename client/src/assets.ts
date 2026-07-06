import { TILE } from './levels';

/**
 * Canonical Kenney sprite manifest. Tiled and the client consume the same
 * committed files under `levels/tiled/Sprites/`; keys are the relative paths
 * inside that tree without the `.png` extension
 * (e.g. `Enemies/Default/frog_idle`).
 */
const SPRITE_URLS = import.meta.glob('../../levels/tiled/Sprites/**/*.png', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>;

const PREFIX = '../../levels/tiled/Sprites/';

export function spriteUrl(key: string): string {
  const url = SPRITE_URLS[`${PREFIX}${key}.png`];
  if (!url)
    throw new Error(`sprite '${key}' is not in the committed asset tree`);
  return url;
}

/** Semantic tile id → sprite key, mirroring `levels/tiled/catalog.tsj`. */
export const TILE_SPRITES: Record<number, string> = {
  [TILE.SOLID]: 'Tiles/Default/terrain_grass_block',
  [TILE.SLOPE_UP]: 'Tiles/Default/terrain_grass_ramp_short_b_mirror',
  [TILE.SLOPE_DOWN]: 'Tiles/Default/terrain_grass_ramp_short_b',
  [TILE.SPAWN]: 'Tiles/Default/sign_right',
  [TILE.FINISH]: 'Tiles/Default/flag_green_a',
  [TILE.LETHAL]: 'Tiles/Default/spikes',
  [TILE.WATER]: 'Tiles/Default/water',
  [TILE.FIRE]: 'Tiles/Default/lava_top',
  [TILE.HEAVY]: 'Tiles/Default/weight',
  [TILE.DECOR]: 'Tiles/Default/bush',
};

export function tileTextureKey(tile: number): string | undefined {
  return TILE_SPRITES[tile];
}

/** Sprite key for a playable character's backend style key (e.g. `frog`). */
export function characterSpriteKey(style: string): string {
  return `Enemies/Default/${style}_idle`;
}

/**
 * Queue the given sprites on a scene's loader under their canonical keys.
 * Throws for keys outside the committed tree.
 */
export function queueSprites(
  load: Phaser.Loader.LoaderPlugin,
  keys: string[],
): void {
  for (const key of keys) {
    if (!load.textureManager.exists(key)) load.image(key, spriteUrl(key));
  }
}
