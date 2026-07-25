import type { DbConnection } from './module_bindings';
// Explicit extension so this module chain also loads under `node --test`.
import { TUNING } from './tuning.ts';

/** Fire hazards burn characters below this fire resistance. */
export const FIRE_RESISTANCE_THRESHOLD = 0.5;
/** Matter density of the heavy pushable obstacles. */
export const HEAVY_DENSITY = 0.0035;
/** Drawn width of the finish pole/flag in world pixels. */
export const FINISH_WIDTH = 64;

/**
 * The depth where physics happens; other depths are scenery. Plane images
 * draw at depth = z, so dynamic objects slot between 0 and the first
 * foreground plane (z >= 1).
 */
export const GAMEPLAY_PLANE_Z = 0;
export const DEPTH = {
  /** Effects (dust, puffs) above the gameplay plane, below the player. */
  EFFECTS: 0.6,
  /** Heavy pushable blocks. */
  HEAVY: 0.8,
  PLAYER: 1 as number,
  /** The upright face rig, always over its player body. */
  FACE: 1.5,
} as const;

/** Semantic ids used by the `data-t` collider markers inside component SVGs. */
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

/** Scroll-speed depth cue derived from z; the gameplay plane scrolls 1:1. */
export function planeParallax(z: number): number {
  const parallax = 1 + z * TUNING.PARALLAX_PER_Z;
  return Math.min(Math.max(parallax, 0.05), 4);
}

/**
 * Resolve a Matter body to its collision root. Concave character hulls
 * decompose into compound bodies and Matter reports collision pairs against
 * the *parts*, never the compound parent — comparing a pair body to the
 * player body by identity silently fails then. Top-level bodies are their
 * own parent.
 */
export function collisionRoot<T extends { parent: T }>(body: T): T {
  let root = body;
  while (root.parent !== root) root = root.parent;
  return root;
}

/** One collider marker in component-local or world coordinates. */
export interface LevelMarker {
  t: number;
  x: number;
  y: number;
  width: number;
  height: number;
  /** Present for polygon markers (slopes). */
  points?: { x: number; y: number }[];
  /** For dynamic markers (heavy): the component-art texture to draw. */
  textureKey?: string;
}

/** How one placement maps component-local coordinates into the world. */
export interface PlacementTransform {
  x: number;
  y: number;
  flipX: boolean;
  scale: number;
  /** The component's natural width; flips mirror around it. */
  componentWidth: number;
}

/** A component texture to load (content-addressed, shared across levels). */
export interface ComponentTexture {
  key: string;
  svg: string;
}

/**
 * One image to draw: a component placement in world space. Levels can span
 * tens of thousands of pixels, far past GPU texture limits, so each
 * placement draws its own small component texture instead of composing
 * per-depth mega-textures.
 */
export interface RenderPlacement {
  textureKey: string;
  x: number;
  y: number;
  z: number;
  flipX: boolean;
  scale: number;
}

export interface DecodedLevel {
  id: string;
  name: string;
  backdropId: string;
  spawn: { x: number; y: number };
  finish: { x: number; y: number };
  renderPlacements: RenderPlacement[];
  textures: ComponentTexture[];
  /** World-space collider markers from gameplay-plane placements. */
  markers: LevelMarker[];
  widthPx: number;
  heightPx: number;
}

/** FNV-1a64 over dimensions + bytes; mirrors the server's content hash. */
export function contentHash(
  widthPx: number,
  heightPx: number,
  data: Uint8Array,
): string {
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

/**
 * Map one component-local marker into world coordinates: uniform scale,
 * then horizontal mirror around the component's own width, then translate.
 * Applies identically to art (via SVG transforms) and colliders.
 */
export function transformMarker(
  marker: LevelMarker,
  transform: PlacementTransform,
): LevelMarker {
  const { x, y, flipX, scale, componentWidth } = transform;
  if (marker.points) {
    const points = marker.points.map((p) => ({
      x: x + scale * (flipX ? componentWidth - p.x : p.x),
      y: y + scale * p.y,
    }));
    const xs = points.map((p) => p.x);
    const ys = points.map((p) => p.y);
    const minX = Math.min(...xs);
    const minY = Math.min(...ys);
    return {
      t: marker.t,
      x: minX,
      y: minY,
      width: Math.max(...xs) - minX,
      height: Math.max(...ys) - minY,
      points,
    };
  }
  return {
    t: marker.t,
    x:
      x + scale * (flipX ? componentWidth - marker.x - marker.width : marker.x),
    y: y + scale * marker.y,
    width: scale * marker.width,
    height: scale * marker.height,
  };
}

/** Parse the `data-t` collider markers out of a component document. */
export function parseMarkers(svg: string, slug: string): LevelMarker[] {
  const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
  if (doc.querySelector('parsererror')) {
    throw new Error(`component '${slug}': stored SVG does not parse`);
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
        throw new Error(`component '${slug}': marker has malformed points`);
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
      throw new Error(`component '${slug}': marker has malformed bounds`);
    }
    markers.push({ t, x, y, width, height });
  }
  return markers;
}

interface ComponentDef {
  id: string;
  slug: string;
  widthPx: number;
  heightPx: number;
  contentHash: string;
  svg: string;
}

interface DecodedPlacement {
  component: ComponentDef;
  x: number;
  y: number;
  z: number;
  flipX: boolean;
  scale: number;
  order: number;
}

const cache = new Map<string, { key: string; level: DecodedLevel }>();

/** How long to wait for the selected level's placements to arrive. */
const PLACEMENTS_TIMEOUT_MS = 6000;

interface PlacementRowLike {
  levelId: { toString(): string };
}

/** Structural table shape so the wait is `node --test`-able. */
interface PlacementTableLike {
  iter(): Iterable<PlacementRowLike>;
  onInsert(cb: (ctx: unknown, row: PlacementRowLike) => void): void;
  removeOnInsert(cb: (ctx: unknown, row: PlacementRowLike) => void): void;
}

/**
 * Resolve once `vw_level_placement_v1` holds rows for `levelId`. The view is
 * gated by the server-side level selection, so the first play of a level
 * races `selectLevelV1` against the subscription update — the rows only
 * arrive a moment after the reducer commits.
 */
export function awaitLevelPlacements(
  table: PlacementTableLike,
  levelId: string,
  timeoutMs = PLACEMENTS_TIMEOUT_MS,
): Promise<void> {
  const present = [...table.iter()].some(
    (row) => row.levelId.toString() === levelId,
  );
  if (present) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const settle = (fn: () => void) => {
      table.removeOnInsert(onInsert);
      clearTimeout(timer);
      fn();
    };
    const onInsert = (_ctx: unknown, row: PlacementRowLike) => {
      if (row.levelId.toString() !== levelId) return;
      settle(resolve);
    };
    const timer = setTimeout(
      () =>
        settle(() =>
          reject(
            new Error(
              `level '${levelId}': placements did not arrive — check the connection`,
            ),
          ),
        ),
      timeoutMs,
    );
    table.onInsert(onInsert);
  });
}

/**
 * Decode the level from subscribed rows: verify component hashes, map
 * placements onto per-component textures, and transform gameplay-plane
 * collider markers into world space. Decoded levels are cached in memory
 * for instant retries; an overwritten import changes the content hashes
 * and is picked up on the next load.
 */
export function loadLevel(conn: DbConnection, levelId: string): DecodedLevel {
  const meta = [...conn.db.vw_level_v1.iter()].find(
    (row) => row.id.toString() === levelId,
  );
  if (!meta) throw new Error(`level '${levelId}' is not available`);
  const rows = [...conn.db.vw_level_placement_v1.iter()]
    .filter((row) => row.levelId.toString() === levelId)
    .sort((a, b) => a.order - b.order);
  if (rows.length === 0)
    throw new Error(`level '${levelId}' has no placements`);

  const components = new Map<string, ComponentDef>();
  for (const row of conn.db.vw_component_v1.iter()) {
    const hash = contentHash(row.widthPx, row.heightPx, row.data);
    if (hash !== row.contentHash) {
      throw new Error(`component '${row.slug}': content hash mismatch`);
    }
    components.set(row.id.toString(), {
      id: row.id.toString(),
      slug: row.slug,
      widthPx: row.widthPx,
      heightPx: row.heightPx,
      contentHash: row.contentHash,
      svg: new TextDecoder().decode(row.data),
    });
  }

  const placements: DecodedPlacement[] = rows.map((row) => {
    const component = components.get(row.componentId.toString());
    if (!component) {
      throw new Error(
        `level '${levelId}' places unknown component '${row.componentId}'`,
      );
    }
    return {
      component,
      x: row.position.x,
      y: row.position.y,
      z: row.position.z,
      flipX: row.flipX,
      scale: row.scale,
      order: row.order,
    };
  });

  const key = placements
    .map(
      (p) =>
        `${p.component.slug}:${p.x}:${p.y}:${p.z}:${p.flipX}:${p.scale}:${p.order}`,
    )
    .join('|');
  const cached = cache.get(levelId);
  if (cached && cached.key === key) return cached.level;

  const componentMarkers = new Map<string, LevelMarker[]>();
  const markersOf = (component: ComponentDef): LevelMarker[] => {
    let local = componentMarkers.get(component.slug);
    if (!local) {
      local = parseMarkers(component.svg, component.slug);
      componentMarkers.set(component.slug, local);
    }
    return local;
  };
  // A component carrying a HEAVY marker is a dynamic object: its art moves
  // with the physics body, so no static image is drawn for it.
  // ponytail: the whole component is treated as dynamic; keep heavy
  // components to just the block.
  const isDynamic = (component: ComponentDef): boolean =>
    markersOf(component).some((marker) => marker.t === TILE.HEAVY);
  const textureKeyOf = (component: ComponentDef): string =>
    `component_${component.contentHash}`;

  const textures = new Map<string, ComponentTexture>();
  const renderPlacements: RenderPlacement[] = [];
  let widthPx = 1;
  let heightPx = 1;
  for (const placement of placements) {
    const { component } = placement;
    textures.set(textureKeyOf(component), {
      key: textureKeyOf(component),
      svg: component.svg,
    });
    if (placement.z === GAMEPLAY_PLANE_Z) {
      widthPx = Math.max(
        widthPx,
        Math.ceil(placement.x + placement.scale * component.widthPx),
      );
      heightPx = Math.max(
        heightPx,
        Math.ceil(placement.y + placement.scale * component.heightPx),
      );
    }
    if (isDynamic(component)) continue;
    renderPlacements.push({
      textureKey: textureKeyOf(component),
      x: placement.x,
      y: placement.y,
      z: placement.z,
      flipX: placement.flipX,
      scale: placement.scale,
    });
  }

  const markers: LevelMarker[] = [];
  for (const placement of placements) {
    if (placement.z !== GAMEPLAY_PLANE_Z) continue;
    const { component } = placement;
    for (const marker of markersOf(component)) {
      const world = transformMarker(marker, {
        x: placement.x,
        y: placement.y,
        flipX: placement.flipX,
        scale: placement.scale,
        componentWidth: component.widthPx,
      });
      if (marker.t === TILE.HEAVY) {
        world.textureKey = textureKeyOf(component);
      }
      markers.push(world);
    }
  }

  if (!placements.some((p) => p.z === GAMEPLAY_PLANE_Z)) {
    throw new Error(`level '${levelId}' has no gameplay-plane placement`);
  }

  const level: DecodedLevel = {
    id: meta.id.toString(),
    name: meta.name,
    backdropId: meta.backdropId.toString(),
    spawn: { x: meta.spawn.x, y: meta.spawn.y },
    finish: { x: meta.finish.x, y: meta.finish.y },
    renderPlacements,
    textures: [...textures.values()],
    markers,
    widthPx,
    heightPx,
  };
  cache.set(levelId, { key, level });
  return level;
}
