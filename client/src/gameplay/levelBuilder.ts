import Phaser from 'phaser';
import { tileTextureKey } from '../assets';
import {
  CELL,
  GAMEPLAY_Z,
  HEAVY_DENSITY,
  TILE,
  type DecodedLayer,
  type DecodedLevel,
} from '../levels';

export interface BuiltLevel {
  spawn: Phaser.Math.Vector2;
  waterRects: Phaser.Geom.Rectangle[];
}

/**
 * Render every decoded layer and build the Matter terrain, slopes, sensors,
 * water regions, and heavy dynamic bodies from semantic tile ids.
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
    if (layer.z === GAMEPLAY_Z) buildGameplayLayer(scene, layer, built);
    else buildVisualLayer(scene, layer);
  }
  return built;
}

function forEachTile(
  layer: DecodedLayer,
  fn: (tile: number, x: number, y: number) => void,
): void {
  for (let y = 0; y < layer.height; y++) {
    for (let x = 0; x < layer.width; x++) {
      const tile = layer.tiles[y * layer.width + x];
      if (tile !== TILE.EMPTY) fn(tile, x, y);
    }
  }
}

function buildVisualLayer(scene: Phaser.Scene, layer: DecodedLayer): void {
  const alpha = layer.z < GAMEPLAY_Z ? 0.8 : 0.9;
  forEachTile(layer, (tile, x, y) => {
    const key = tileTextureKey(tile);
    if (!key) return;
    scene.add
      .image(
        x * layer.cellWidth + layer.cellWidth / 2,
        y * layer.cellHeight + layer.cellHeight / 2,
        key,
      )
      .setDisplaySize(layer.cellWidth, layer.cellHeight)
      .setScrollFactor(layer.parallaxX, layer.parallaxY)
      .setDepth(layer.z)
      .setAlpha(alpha);
  });
}

function buildGameplayLayer(
  scene: Phaser.Scene,
  layer: DecodedLayer,
  built: BuiltLevel,
): void {
  // Draw every visible tile of the gameplay layer at depth 127.
  forEachTile(layer, (tile, x, y) => {
    if (tile === TILE.HEAVY) return; // rendered by its dynamic body
    const key = tileTextureKey(tile);
    if (!key) return;
    scene.add
      .image(x * CELL + CELL / 2, y * CELL + CELL / 2, key)
      .setDepth(GAMEPLAY_Z);
  });

  // Merge horizontal runs of solid tiles into single static bodies.
  for (let y = 0; y < layer.height; y++) {
    let runStart = -1;
    for (let x = 0; x <= layer.width; x++) {
      const solid =
        x < layer.width && layer.tiles[y * layer.width + x] === TILE.SOLID;
      if (solid && runStart < 0) runStart = x;
      if (!solid && runStart >= 0) {
        const cells = x - runStart;
        scene.matter.add.rectangle(
          (runStart + cells / 2) * CELL,
          y * CELL + CELL / 2,
          cells * CELL,
          CELL,
          { isStatic: true, label: 'terrain', friction: 0.9 },
        );
        runStart = -1;
      }
    }
  }

  forEachTile(layer, (tile, x, y) => {
    const originX = x * CELL;
    const originY = y * CELL;
    switch (tile) {
      case TILE.SPAWN:
        built.spawn.set(originX + CELL / 2, originY + CELL / 2);
        break;
      case TILE.SLOPE_UP:
        // Surface rises left→right, matching the mirrored Kenney ramp art.
        addTriangle(scene, originX, originY, [
          { x: 0, y: CELL },
          { x: CELL, y: CELL },
          { x: CELL, y: 0 },
        ]);
        break;
      case TILE.SLOPE_DOWN:
        // Surface falls left→right, matching terrain_grass_ramp_short_b.
        addTriangle(scene, originX, originY, [
          { x: 0, y: 0 },
          { x: CELL, y: CELL },
          { x: 0, y: CELL },
        ]);
        break;
      case TILE.LETHAL:
        addSensor(scene, originX, originY, 'lethal', 12);
        break;
      case TILE.FIRE:
        addSensor(scene, originX, originY, 'fire', 8);
        break;
      case TILE.FINISH:
        addSensor(scene, originX, originY, 'finish', 0);
        break;
      case TILE.WATER:
        built.waterRects.push(
          new Phaser.Geom.Rectangle(originX, originY, CELL, CELL),
        );
        break;
      case TILE.HEAVY: {
        const key = tileTextureKey(TILE.HEAVY);
        if (!key) break;
        const block = scene.matter.add.image(
          originX + CELL / 2,
          originY + CELL / 2,
          key,
        );
        block.setBody(
          { type: 'rectangle', width: CELL, height: CELL },
          {
            label: 'heavy',
            density: HEAVY_DENSITY,
            friction: 0.8,
            frictionStatic: 1.2,
          },
        );
        block.setDepth(GAMEPLAY_Z);
        break;
      }
    }
  });
}

function addTriangle(
  scene: Phaser.Scene,
  originX: number,
  originY: number,
  verts: { x: number; y: number }[],
): void {
  const cx = verts.reduce((sum, v) => sum + v.x, 0) / verts.length;
  const cy = verts.reduce((sum, v) => sum + v.y, 0) / verts.length;
  scene.matter.add.fromVertices(originX + cx, originY + cy, verts, {
    isStatic: true,
    label: 'terrain',
    friction: 0.9,
  });
}

function addSensor(
  scene: Phaser.Scene,
  originX: number,
  originY: number,
  label: string,
  inset: number,
): void {
  scene.matter.add.rectangle(
    originX + CELL / 2,
    originY + CELL / 2,
    CELL - inset * 2,
    CELL - inset * 2,
    { isStatic: true, isSensor: true, label },
  );
}
