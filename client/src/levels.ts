import type { DbConnection } from './module_bindings';

/** Gameplay grid cell size in pixels (authoring unit; markers may merge cells). */
export const CELL = 64;
export const GAMEPLAY_Z = 127;
/** Fire hazards burn characters below this fire resistance. */
export const FIRE_RESISTANCE_THRESHOLD = 0.5;
/** Matter density of the heavy pushable obstacles. */
export const HEAVY_DENSITY = 0.0035;

/** Semantic ids used by the `data-t` collider markers inside layer SVGs. */
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
  widthPx: number;
  heightPx: number;
  parallaxX: number;
  parallaxY: number;
  /** The standalone SVG scene document for this layer. */
  svg: string;
  contentHash: string;
}

/** One collider marker parsed from the gameplay layer's hidden group. */
export interface LevelMarker {
  t: number;
  x: number;
  y: number;
  width: number;
  height: number;
  /** Present for polygon markers (slopes). */
  points?: { x: number; y: number }[];
}

export interface DecodedLevel {
  id: string;
  name: string;
  layers: DecodedLayer[];
  gameplay: DecodedLayer;
  markers: LevelMarker[];
  widthPx: number;
  heightPx: number;
}

/** Phaser texture key for one decoded layer's scene SVG. */
export function layerTextureKey(layer: DecodedLayer): string {
  return `level_layer_${layer.contentHash}`;
}

function fnv1a64(widthPx: number, heightPx: number, data: Uint8Array): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  const eat = (byte: number) => {
    hash ^= BigInt(byte);
    hash = (hash * prime) & mask;
  };
  for (let shift = 0; shift < 32; shift += 8) eat((widthPx >> shift) & 0xff);
  for (let shift = 0; shift < 32; shift += 8) eat((heightPx >> shift) & 0xff);
  for (const byte of data) eat(byte);
  return hash.toString(16).padStart(16, '0');
}

/** Parse the `data-t` collider markers out of a gameplay layer document. */
function parseMarkers(svg: string, z: number): LevelMarker[] {
  const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
  if (doc.querySelector('parsererror')) {
    throw new Error(`layer z ${z}: stored SVG does not parse`);
  }
  const markers: LevelMarker[] = [];
  for (const el of doc.querySelectorAll('[data-t]')) {
    const t = Number(el.getAttribute('data-t'));
    if (!Number.isInteger(t)) continue;
    const points = el.getAttribute('points');
    if (points) {
      const list = points
        .trim()
        .split(/\s+/)
        .map((pair) => {
          const [x, y] = pair.split(',').map(Number);
          return { x, y };
        });
      if (list.some((p) => !Number.isFinite(p.x) || !Number.isFinite(p.y))) {
        throw new Error(`layer z ${z}: marker has malformed points`);
      }
      const xs = list.map((p) => p.x);
      const ys = list.map((p) => p.y);
      const x = Math.min(...xs);
      const y = Math.min(...ys);
      markers.push({
        t,
        x,
        y,
        width: Math.max(...xs) - x,
        height: Math.max(...ys) - y,
        points: list,
      });
      continue;
    }
    const x = Number(el.getAttribute('x'));
    const y = Number(el.getAttribute('y'));
    const width = Number(el.getAttribute('width'));
    const height = Number(el.getAttribute('height'));
    if (![x, y, width, height].every(Number.isFinite)) {
      throw new Error(`layer z ${z}: marker has malformed bounds`);
    }
    markers.push({ t, x, y, width, height });
  }
  return markers;
}

const cache = new Map<string, { key: string; level: DecodedLevel }>();

/**
 * Decode the level from subscribed rows, verifying encoding and content
 * hash. Decoded levels are cached in memory for instant retries; an
 * overwritten import changes the content hashes and is picked up on the
 * next load.
 */
export function loadLevel(conn: DbConnection, levelId: string): DecodedLevel {
  const meta = [...conn.db.vw_level_v1.iter()].find(
    (row) => row.id.toString() === levelId,
  );
  if (!meta) throw new Error(`level '${levelId}' is not available`);
  const rows = [...conn.db.vw_level_layer_v1.iter()].filter(
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
      if (row.encoding !== 'svg-v1') {
        throw new Error(
          `layer z ${row.z}: unsupported encoding '${row.encoding}'`,
        );
      }
      const hash = fnv1a64(row.widthPx, row.heightPx, row.data);
      if (hash !== row.contentHash) {
        throw new Error(`layer z ${row.z}: content hash mismatch`);
      }
      return {
        z: row.z,
        widthPx: row.widthPx,
        heightPx: row.heightPx,
        parallaxX: row.parallaxX,
        parallaxY: row.parallaxY,
        svg: new TextDecoder().decode(row.data),
        contentHash: row.contentHash,
      };
    })
    .sort((a, b) => a.z - b.z);

  const gameplay = layers.find((layer) => layer.z === GAMEPLAY_Z);
  if (!gameplay) throw new Error(`level '${levelId}' has no gameplay layer`);
  const markers = parseMarkers(gameplay.svg, gameplay.z);
  if (!markers.some((m) => m.t === TILE.SPAWN)) {
    throw new Error(`level '${levelId}' has no spawn marker`);
  }

  const level: DecodedLevel = {
    id: meta.id.toString(),
    name: meta.name,
    layers,
    gameplay,
    markers,
    widthPx: gameplay.widthPx,
    heightPx: gameplay.heightPx,
  };
  cache.set(levelId, { key, level });
  return level;
}
