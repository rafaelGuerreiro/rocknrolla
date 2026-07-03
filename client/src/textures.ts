import Phaser from 'phaser';
import { CELL, TILE } from './levels';

/**
 * Procedural paper-cut style textures shared by every scene. Safe to call
 * repeatedly; textures are generated once per game instance.
 */
export function ensureTextures(scene: Phaser.Scene): void {
  if (scene.textures.exists('tile-solid')) return;
  const g = scene.add.graphics();

  const tileTexture = (key: string, draw: () => void) => {
    g.clear();
    draw();
    g.generateTexture(key, CELL, CELL);
  };

  tileTexture('tile-solid', () => {
    g.fillStyle(0xb0713f).fillRect(0, 0, CELL, CELL);
    g.fillStyle(0xd6a860).fillRect(0, 0, CELL, 5);
  });
  tileTexture('tile-slope-up', () => {
    g.fillStyle(0xb0713f).fillTriangle(0, CELL, CELL, CELL, CELL, 0);
    g.lineStyle(4, 0xd6a860).lineBetween(0, CELL, CELL, 0);
  });
  tileTexture('tile-slope-down', () => {
    g.fillStyle(0xb0713f).fillTriangle(0, 0, 0, CELL, CELL, CELL);
    g.lineStyle(4, 0xd6a860).lineBetween(0, 0, CELL, CELL);
  });
  tileTexture('tile-finish', () => {
    g.fillStyle(0xf0dc82).fillRect(13, 2, 4, CELL - 2);
    g.fillStyle(0xf5c451).fillTriangle(17, 4, 31, 9, 17, 15);
  });
  tileTexture('tile-lethal', () => {
    g.fillStyle(0xd64a4a);
    g.fillTriangle(0, CELL, 5, 8, 11, CELL);
    g.fillTriangle(10, CELL, 16, 4, 22, CELL);
    g.fillTriangle(21, CELL, 27, 8, CELL, CELL);
  });
  tileTexture('tile-water', () => {
    g.fillStyle(0x4080d0, 0.62).fillRect(0, 0, CELL, CELL);
    g.fillStyle(0x96d0ff, 0.8).fillRect(0, 0, CELL, 4);
  });
  tileTexture('tile-fire', () => {
    g.fillStyle(0xe86a33).fillTriangle(3, CELL, 9, 6, 15, CELL);
    g.fillStyle(0xffc454).fillTriangle(13, CELL, 20, 2, 27, CELL);
    g.fillStyle(0xe86a33).fillRect(0, CELL - 6, CELL, 6);
  });
  tileTexture('tile-heavy', () => {
    g.fillStyle(0x585c64).fillRect(0, 0, CELL, CELL);
    g.fillStyle(0x828892).fillRect(3, 3, CELL - 6, CELL - 6);
    g.lineStyle(2, 0x585c64).strokeRect(8, 8, CELL - 16, CELL - 16);
  });
  tileTexture('tile-decor', () => {
    g.fillStyle(0x6880ac, 0.85).fillCircle(16, 22, 9);
    g.fillStyle(0x6880ac, 0.6).fillCircle(9, 26, 5);
    g.fillStyle(0x6880ac, 0.6).fillCircle(24, 26, 5);
  });
  tileTexture('tile-spawn', () => {
    g.fillStyle(0x7ed6df, 0.25).fillCircle(16, 16, 12);
    g.lineStyle(2, 0x7ed6df, 0.7).strokeCircle(16, 16, 12);
  });

  g.clear();
  g.fillStyle(0xffffff).fillCircle(6, 6, 6);
  g.generateTexture('dust', 12, 12);

  g.destroy();
}

const TILE_TEXTURES: Record<number, string> = {
  [TILE.SOLID]: 'tile-solid',
  [TILE.SLOPE_UP]: 'tile-slope-up',
  [TILE.SLOPE_DOWN]: 'tile-slope-down',
  [TILE.SPAWN]: 'tile-spawn',
  [TILE.FINISH]: 'tile-finish',
  [TILE.LETHAL]: 'tile-lethal',
  [TILE.WATER]: 'tile-water',
  [TILE.FIRE]: 'tile-fire',
  [TILE.HEAVY]: 'tile-heavy',
  [TILE.DECOR]: 'tile-decor',
};

export function tileTextureKey(tile: number): string | undefined {
  return TILE_TEXTURES[tile];
}

/** Character ball with a contrasting radius line so rolling reads clearly. */
export function ensureBallTexture(scene: Phaser.Scene, characterId: string, style: string): string {
  const key = `ball-${characterId}`;
  if (scene.textures.exists(key)) return key;
  const color = Phaser.Display.Color.HexStringToColor(style).color;
  const dark = Phaser.Display.Color.HexStringToColor(style).darken(45).color;
  const g = scene.add.graphics();
  g.fillStyle(color).fillCircle(14, 14, 13);
  g.lineStyle(2, dark, 0.9).strokeCircle(14, 14, 12);
  g.lineStyle(4, dark).lineBetween(14, 14, 26, 14);
  g.fillStyle(dark).fillCircle(20, 8, 2.5);
  g.generateTexture(key, 28, 28);
  g.destroy();
  return key;
}
