import Phaser from 'phaser';
import {
  DEPTH,
  FINISH_WIDTH,
  HEAVY_DENSITY,
  TILE,
  planeParallax,
  type DecodedLevel,
  type LevelMarker,
} from '../levels';
import { polygonCentroid } from './playerBody';

/** Sensor rects shrink by this much per side so grazes feel fair. */
const LETHAL_INSET = 12;
const FIRE_INSET = 8;

/**
 * The finish sensor spans this far above and below the drawn pole so the
 * run ends even when the player sails over it or clips under it.
 */
const FINISH_SENSOR_HEIGHT_PX = 100_000;

export interface BuiltLevel {
  spawn: Phaser.Math.Vector2;
  waterRects: Phaser.Geom.Rectangle[];
}

/**
 * Draw every placement as its own component image (small textures, loaded
 * once per content hash) with z-derived parallax and depth, place the
 * level-owned spawn and finish, and build the Matter terrain, slopes,
 * sensors, water regions, and heavy dynamic bodies from the world-space
 * collider markers.
 */
export function buildLevel(
  scene: Phaser.Scene,
  level: DecodedLevel,
): BuiltLevel {
  const built: BuiltLevel = {
    spawn: new Phaser.Math.Vector2(level.spawn.x, level.spawn.y),
    waterRects: [],
  };
  for (const placement of level.renderPlacements) {
    const parallax = planeParallax(placement.z);
    scene.add
      .image(placement.x, placement.y, placement.textureKey)
      .setOrigin(0, 0)
      .setScale(placement.scale)
      .setFlipX(placement.flipX)
      .setScrollFactor(parallax, parallax)
      .setDepth(placement.z);
  }
  for (const marker of level.markers) {
    buildMarker(scene, marker, built);
  }
  buildFinish(scene, level);
  return built;
}

/** The finish pole: a sensor column plus the flag visual. */
function buildFinish(scene: Phaser.Scene, level: DecodedLevel): void {
  scene.matter.add.rectangle(
    level.finish.x,
    level.finish.y,
    FINISH_WIDTH,
    FINISH_SENSOR_HEIGHT_PX,
    { isStatic: true, isSensor: true, label: 'finish' },
  );
  scene.add
    .image(level.finish.x, level.finish.y + FINISH_WIDTH / 2, 'tile_finish')
    .setOrigin(0.5, 1)
    .setDisplaySize(FINISH_WIDTH, FINISH_WIDTH)
    .setDepth(DEPTH.EFFECTS);
}

function buildMarker(
  scene: Phaser.Scene,
  marker: LevelMarker,
  built: BuiltLevel,
): void {
  switch (marker.t) {
    case TILE.SOLID:
      scene.matter.add.rectangle(
        marker.x + marker.width / 2,
        marker.y + marker.height / 2,
        marker.width,
        marker.height,
        { isStatic: true, label: 'terrain', friction: 0.9 },
      );
      break;
    case TILE.SLOPE_UP:
    case TILE.SLOPE_DOWN: {
      const points = marker.points;
      if (!points || points.length < 3) break;
      // fromVertices puts the shape's centre of MASS at the given position,
      // so anchor at the polygon's area centroid or the shape drifts
      // (vertex mean only coincides with it for triangles).
      const centre = polygonCentroid(points);
      scene.matter.add.fromVertices(
        centre.x,
        centre.y,
        points.map((p) => ({ x: p.x - marker.x, y: p.y - marker.y })),
        { isStatic: true, label: 'terrain', friction: 0.9 },
      );
      break;
    }
    case TILE.LETHAL:
      addSensor(scene, marker, 'lethal', LETHAL_INSET);
      break;
    case TILE.FIRE:
      addSensor(scene, marker, 'fire', FIRE_INSET);
      break;
    case TILE.WATER:
      built.waterRects.push(
        new Phaser.Geom.Rectangle(
          marker.x,
          marker.y,
          marker.width,
          marker.height,
        ),
      );
      break;
    case TILE.HEAVY: {
      // Dynamic: drawn as a sprite from the component's own art so it can
      // move with the physics body (never baked into the composed plane).
      if (!marker.textureKey) break;
      const block = scene.matter.add.image(
        marker.x + marker.width / 2,
        marker.y + marker.height / 2,
        marker.textureKey,
      );
      block.setDisplaySize(marker.width, marker.height);
      block.setBody(
        { type: 'rectangle', width: marker.width, height: marker.height },
        {
          label: 'heavy',
          density: HEAVY_DENSITY,
          friction: 0.8,
          frictionStatic: 1.2,
        },
      );
      block.setDepth(DEPTH.HEAVY);
      break;
    }
    default:
      break;
  }
}

function addSensor(
  scene: Phaser.Scene,
  marker: LevelMarker,
  label: string,
  inset: number,
): void {
  scene.matter.add.rectangle(
    marker.x + marker.width / 2,
    marker.y + marker.height / 2,
    marker.width - inset * 2,
    marker.height - inset * 2,
    { isStatic: true, isSensor: true, label },
  );
}
