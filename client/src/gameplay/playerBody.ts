import Phaser from 'phaser';
import { GAMEPLAY_Z } from '../levels';

/**
 * On-screen character size in world pixels. Matches the original 64px
 * sprites so body mass and level proportions stay unchanged while the
 * SVG textures raster larger for crispness.
 */
const PLAYER_DISPLAY_PX = 64;
/** Pixels with alpha at or above this are part of the collision silhouette. */
const ALPHA_THRESHOLD = 16;
/** Douglas-Peucker tolerance in pixels for simplifying the traced contour. */
const SIMPLIFY_EPSILON = 2;

export interface BodyStats {
  density: number;
}

interface CachedHull {
  /** Simplified outer contour in image pixel coordinates. */
  contour: Phaser.Types.Math.Vector2Like[];
  /** Area centroid of the contour (Matter's center of mass) in image pixels. */
  centroid: Phaser.Math.Vector2;
  width: number;
  height: number;
}

const hullCache = new Map<string, CachedHull>();

/**
 * Create the playable character as a Matter image whose collision polygon is
 * derived from the texture's alpha channel. The hull is traced and cached
 * once per texture load; transparent padding is excluded. Throws when the
 * texture is missing or has no opaque pixels.
 */
export function createPlayerBody(
  scene: Phaser.Scene,
  textureKey: string,
  spawn: Phaser.Types.Math.Vector2Like,
  stats: BodyStats,
): Phaser.Physics.Matter.Image {
  const hull = hullForTexture(scene, textureKey);
  const player = scene.matter.add.image(spawn.x ?? 0, spawn.y ?? 0, textureKey);
  const body = scene.matter.bodies.fromVertices(
    0,
    0,
    [hull.contour as MatterJS.Vector[]],
    {
      label: 'player',
      density: stats.density,
      friction: 0.9,
      frictionAir: 0.012,
      restitution: 0.08,
    },
  );
  player.setExistingBody(body);
  // Matter's setScale rescales the attached body with the sprite, mapping
  // the texture-resolution hull down to the world-pixel character size.
  player.setScale(PLAYER_DISPLAY_PX / hull.width);
  // Draw the sprite so the pixel at the hull centroid sits on the body's
  // center of mass; rotation then keeps sprite and hull aligned.
  player.setOrigin(hull.centroid.x / hull.width, hull.centroid.y / hull.height);
  player.setPosition(spawn.x ?? 0, spawn.y ?? 0);
  player.setDepth(GAMEPLAY_Z + 1);
  return player;
}

function hullForTexture(scene: Phaser.Scene, textureKey: string): CachedHull {
  const cached = hullCache.get(textureKey);
  if (cached) return cached;

  if (!scene.textures.exists(textureKey)) {
    throw new Error(`character texture '${textureKey}' is not loaded`);
  }
  const source = scene.textures.get(textureKey).getSourceImage();
  if (
    !('width' in source) ||
    source instanceof Phaser.GameObjects.RenderTexture
  ) {
    throw new Error(
      `character texture '${textureKey}' has no readable image source`,
    );
  }
  const width = source.width;
  const height = source.height;
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context)
    throw new Error('2d canvas context unavailable for alpha tracing');
  context.drawImage(source as CanvasImageSource, 0, 0);
  const alpha = context.getImageData(0, 0, width, height).data;
  const opaque = (x: number, y: number): boolean =>
    x >= 0 &&
    y >= 0 &&
    x < width &&
    y < height &&
    alpha[(y * width + x) * 4 + 3] >= ALPHA_THRESHOLD;

  const boundary = traceBoundary(opaque, width, height);
  if (!boundary) {
    throw new Error(
      `character texture '${textureKey}' has no opaque pixels to trace`,
    );
  }
  const contour = simplifyClosed(boundary, SIMPLIFY_EPSILON);
  if (contour.length < 3) {
    throw new Error(
      `character texture '${textureKey}' produced a degenerate collision hull`,
    );
  }
  const hull: CachedHull = {
    contour,
    centroid: polygonCentroid(contour),
    width,
    height,
  };
  hullCache.set(textureKey, hull);
  return hull;
}

/** Clockwise 8-neighborhood offsets starting from west. */
const NEIGHBORS: ReadonlyArray<readonly [number, number]> = [
  [-1, 0],
  [-1, -1],
  [0, -1],
  [1, -1],
  [1, 0],
  [1, 1],
  [0, 1],
  [-1, 1],
];

/**
 * Moore-neighbor boundary tracing with Jacob's stopping criterion. Returns
 * the outer boundary pixel centers of the first opaque region, or undefined
 * when the image is fully transparent.
 */
function traceBoundary(
  opaque: (x: number, y: number) => boolean,
  width: number,
  height: number,
): Phaser.Math.Vector2[] | undefined {
  let startX = -1;
  let startY = -1;
  outer: for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      if (opaque(x, y)) {
        startX = x;
        startY = y;
        break outer;
      }
    }
  }
  if (startX < 0) return undefined;

  const points: Phaser.Math.Vector2[] = [
    new Phaser.Math.Vector2(startX + 0.5, startY + 0.5),
  ];
  // The start pixel is the first in scan order, so its west neighbor is empty.
  let currentX = startX;
  let currentY = startY;
  let backtrack = 0; // index in NEIGHBORS pointing at the empty pixel we came from
  const startBacktrack = backtrack;
  const maxSteps = 4 * width * height;

  for (let step = 0; step < maxSteps; step++) {
    let found = -1;
    for (let i = 1; i <= NEIGHBORS.length; i++) {
      const index = (backtrack + i) % NEIGHBORS.length;
      const [dx, dy] = NEIGHBORS[index];
      if (opaque(currentX + dx, currentY + dy)) {
        found = index;
        break;
      }
    }
    if (found < 0) return points; // isolated single pixel

    // Reposition backtrack to the empty neighbor just before the found pixel,
    // expressed relative to the new current pixel.
    const previousIndex = (found + NEIGHBORS.length - 1) % NEIGHBORS.length;
    const [px, py] = NEIGHBORS[previousIndex];
    const [dx, dy] = NEIGHBORS[found];
    currentX += dx;
    currentY += dy;
    backtrack = neighborIndex(px - dx, py - dy);

    if (
      currentX === startX &&
      currentY === startY &&
      backtrack === startBacktrack
    ) {
      return points;
    }
    points.push(new Phaser.Math.Vector2(currentX + 0.5, currentY + 0.5));
  }
  return points;
}

function neighborIndex(dx: number, dy: number): number {
  const index = NEIGHBORS.findIndex(([nx, ny]) => nx === dx && ny === dy);
  if (index < 0) throw new Error(`not an 8-neighborhood offset: ${dx},${dy}`);
  return index;
}

/** Ramer-Douglas-Peucker simplification of a closed contour. */
function simplifyClosed(
  points: Phaser.Math.Vector2[],
  epsilon: number,
): Phaser.Math.Vector2[] {
  if (points.length < 4) return points;
  // Split the loop at the two mutually farthest anchor points so both chains
  // have stable endpoints.
  let far = 1;
  let farDistance = 0;
  for (let i = 1; i < points.length; i++) {
    const d = Phaser.Math.Distance.BetweenPointsSquared(points[0], points[i]);
    if (d > farDistance) {
      farDistance = d;
      far = i;
    }
  }
  const first = rdp(points.slice(0, far + 1), epsilon);
  const second = rdp([...points.slice(far), points[0]], epsilon);
  return [...first.slice(0, -1), ...second.slice(0, -1)];
}

function rdp(
  points: Phaser.Math.Vector2[],
  epsilon: number,
): Phaser.Math.Vector2[] {
  if (points.length < 3) return points;
  const first = points[0];
  const last = points[points.length - 1];
  const line = new Phaser.Geom.Line(first.x, first.y, last.x, last.y);
  let index = -1;
  let maxDistance = 0;
  for (let i = 1; i < points.length - 1; i++) {
    const d = distanceToSegment(points[i], line);
    if (d > maxDistance) {
      maxDistance = d;
      index = i;
    }
  }
  if (maxDistance <= epsilon) return [first, last];
  const left = rdp(points.slice(0, index + 1), epsilon);
  const right = rdp(points.slice(index), epsilon);
  return [...left.slice(0, -1), ...right];
}

function distanceToSegment(
  point: Phaser.Math.Vector2,
  line: Phaser.Geom.Line,
): number {
  const closest = Phaser.Geom.Line.GetNearestPoint(line, point);
  const t =
    Math.abs(line.x2 - line.x1) > Math.abs(line.y2 - line.y1)
      ? (closest.x - line.x1) / (line.x2 - line.x1 || 1)
      : (closest.y - line.y1) / (line.y2 - line.y1 || 1);
  if (Number.isNaN(t) || t < 0)
    return Phaser.Math.Distance.BetweenPoints(point, line.getPointA());
  if (t > 1) return Phaser.Math.Distance.BetweenPoints(point, line.getPointB());
  return Phaser.Math.Distance.BetweenPoints(point, closest);
}

/** Area centroid of a simple polygon, matching Matter's Vertices.centre. */
function polygonCentroid(
  points: Phaser.Types.Math.Vector2Like[],
): Phaser.Math.Vector2 {
  let area = 0;
  let cx = 0;
  let cy = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    const cross = (a.x ?? 0) * (b.y ?? 0) - (b.x ?? 0) * (a.y ?? 0);
    area += cross;
    cx += ((a.x ?? 0) + (b.x ?? 0)) * cross;
    cy += ((a.y ?? 0) + (b.y ?? 0)) * cross;
  }
  area *= 3; // 6 * (area / 2)
  return new Phaser.Math.Vector2(cx / area, cy / area);
}
