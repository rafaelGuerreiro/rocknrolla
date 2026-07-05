import type { DbConnection } from './module_bindings';

/** Gameplay layer logical cell size in pixels (Kenney Default tile size). */
export const CELL = 64;
export const GAMEPLAY_Z = 127;
/** Fire hazards burn characters below this fire resistance. */
export const FIRE_RESISTANCE_THRESHOLD = 0.5;
/** Matter density of the heavy pushable obstacle tiles. */
export const HEAVY_DENSITY = 0.0035;

export const TILE = {
  EMPTY: 0,
  SOLID: 1,
  SLOPE_UP: 2,
  SLOPE_DOWN: 3,
  SPAWN: 4,
  FINISH: 5,
  LETHAL: 6,
  WATER: 7,
  FIRE: 8,
  HEAVY: 9,
  DECOR: 10,
} as const;

export interface DecodedLayer {
  z: number;
  width: number;
  height: number;
  cellWidth: number;
  cellHeight: number;
  parallaxX: number;
  parallaxY: number;
  tiles: Uint8Array;
}

export interface DecodedLevel {
  id: string;
  name: string;
  layers: DecodedLayer[];
  gameplay: DecodedLayer;
  widthPx: number;
  heightPx: number;
}

function fnv1a64(width: number, height: number, tiles: Uint8Array): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  const eat = (byte: number) => {
    hash ^= BigInt(byte);
    hash = (hash * prime) & mask;
  };
  eat(width & 0xff);
  eat((width >> 8) & 0xff);
  eat(height & 0xff);
  eat((height >> 8) & 0xff);
  for (const byte of tiles) eat(byte);
  return hash.toString(16).padStart(16, '0');
}

/** Decode `rle-v1` bytes, enforcing the exact `width * height` tile count. */
export function rleDecode(data: Uint8Array, width: number, height: number): Uint8Array {
  if (data.length % 2 !== 0) throw new Error('rle-v1: unpaired trailing byte');
  const expected = width * height;
  const tiles = new Uint8Array(expected);
  let cursor = 0;
  for (let i = 0; i < data.length; i += 2) {
    const run = data[i];
    const tile = data[i + 1];
    if (run === 0) throw new Error('rle-v1: zero run length');
    if (cursor + run > expected) throw new Error('rle-v1: decoded length too long');
    tiles.fill(tile, cursor, cursor + run);
    cursor += run;
  }
  if (cursor !== expected) throw new Error(`rle-v1: decoded ${cursor} tiles, expected ${expected}`);
  return tiles;
}

const cache = new Map<string, { key: string; level: DecodedLevel }>();

/**
 * Decode the level from subscribed rows, verifying encoding and content
 * hash. Decoded levels are cached in memory for instant retries; an
 * overwritten import changes the content hashes and is picked up on the
 * next load.
 */
export function loadLevel(conn: DbConnection, levelId: string): DecodedLevel {
  const meta = [...conn.db.vw_level.iter()].find((row) => row.id.toString() === levelId);
  if (!meta) throw new Error(`level '${levelId}' is not available`);
  const rows = [...conn.db.vw_level_layer.iter()].filter(
    (row) => row.levelId.toString() === levelId,
  );
  if (rows.length === 0) throw new Error(`level '${levelId}' has no layers`);

  const key = rows
    .map((row) => `${row.z}:${row.contentHash}`)
    .sort()
    .join('|');
  const cached = cache.get(levelId);
  if (cached && cached.key === key) return cached.level;

  const layers: DecodedLayer[] = rows
    .map((row) => {
      if (row.encoding !== 'rle-v1') {
        throw new Error(`layer z ${row.z}: unsupported encoding '${row.encoding}'`);
      }
      const tiles = rleDecode(row.data, row.width, row.height);
      const hash = fnv1a64(row.width, row.height, tiles);
      if (hash !== row.contentHash) {
        throw new Error(`layer z ${row.z}: content hash mismatch`);
      }
      return {
        z: row.z,
        width: row.width,
        height: row.height,
        cellWidth: row.cellWidth,
        cellHeight: row.cellHeight,
        parallaxX: row.parallaxX,
        parallaxY: row.parallaxY,
        tiles,
      };
    })
    .sort((a, b) => a.z - b.z);

  const gameplay = layers.find((layer) => layer.z === GAMEPLAY_Z);
  if (!gameplay) throw new Error(`level '${levelId}' has no gameplay layer`);

  const level: DecodedLevel = {
    id: meta.id.toString(),
    name: meta.name,
    layers,
    gameplay,
    widthPx: gameplay.width * CELL,
    heightPx: gameplay.height * CELL,
  };
  cache.set(levelId, { key, level });
  return level;
}
