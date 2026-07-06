import Phaser from 'phaser';
import {
  CELL,
  HEAVY_DENSITY,
  TILE,
  layerTextureKey,
  type DecodedLevel,
  type LevelMarker,
} from '../levels';

/** Sensor rects shrink by this much per side so grazes feel fair. */
const LETHAL_INSET = 12;
const FIRE_INSET = 8;

export interface BuiltLevel {
  spawn: Phaser.Math.Vector2;
  waterRects: Phaser.Geom.Rectangle[];
}

/**
 * Draw every layer's scene SVG (already loaded as a texture) and build the
 * Matter terrain, slopes, sensors, water regions, and heavy dynamic bodies
 * from the gameplay layer's collider markers.
 */
export function buildLevel(
  scene: Phaser.Scene,
  level: DecodedLevel,
): BuiltLevel {
  const built: BuiltLevel = {
    spawn: new Phaser.Math.Vector2(CELL * 2, CELL * 2),
    waterRects: [],
  };
  for (const layer of level.layers) {
    scene.add
      .image(0, 0, layerTextureKey(layer))
      .setOrigin(0, 0)
      .setScrollFactor(layer.parallaxX, layer.parallaxY)
      .setDepth(layer.z);
  }
  for (const marker of level.markers) {
    buildMarker(scene, marker, built);
  }
  return built;
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
      const cx = points.reduce((sum, p) => sum + p.x, 0) / points.length;
      const cy = points.reduce((sum, p) => sum + p.y, 0) / points.length;
      scene.matter.add.fromVertices(
        cx,
        cy,
        points.map((p) => ({ x: p.x - marker.x, y: p.y - marker.y })),
        { isStatic: true, label: 'terrain', friction: 0.9 },
      );
      break;
    }
    case TILE.SPAWN:
      built.spawn.set(
        marker.x + marker.width / 2,
        marker.y + marker.height / 2,
      );
      break;
    case TILE.FINISH:
      addSensor(scene, marker, 'finish', 0);
      break;
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
      // Marker-only in the scene SVG: drawn as a dynamic sprite instead.
      const block = scene.matter.add.image(
        marker.x + marker.width / 2,
        marker.y + marker.height / 2,
        'tile_heavy',
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
      // One above the gameplay scene image so the block reads on top.
      block.setDepth(128);
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
