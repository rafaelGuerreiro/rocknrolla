import Phaser from 'phaser';
import { DPR } from './ui';

/**
 * Tiny procedural particle textures the Kenney pack does not supply.
 * Safe to call repeatedly; generated once per game instance.
 */
export function ensureParticleTextures(scene: Phaser.Scene): void {
  if (scene.textures.exists('dust')) return;
  const g = scene.add.graphics();
  g.fillStyle(0xffffff).fillCircle(6, 6, 6);
  g.generateTexture('dust', 12, 12);
  g.destroy();
}

/**
 * Procedural UI chrome textures: the radial spotlight and the lootbox
 * reveal rays. Scenery (sky, hills) is DB content — see `content.ts`.
 * Canvas textures so gradients render identically on WebGL and Canvas,
 * generated at the canvas backing resolution (logical × DPR); consumers
 * draw them with `setDisplaySize` in logical coordinates.
 */
export function ensureChromeTextures(scene: Phaser.Scene): void {
  if (scene.textures.exists('spotlight')) return;
  const { width, height } = scene.scale;

  const spot = scene.textures.createCanvas('spotlight', width, height);
  if (spot) {
    const ctx = spot.getContext();
    const grad = ctx.createRadialGradient(
      width / 2,
      height * 0.42,
      60,
      width / 2,
      height * 0.42,
      height * 0.95,
    );
    grad.addColorStop(0, '#8a4a56');
    grad.addColorStop(1, '#3a2440');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, height);
    spot.refresh();
  }

  rayTexture(scene);
}

/** Radial burst of soft wedges for the lootbox reveal. */
function rayTexture(scene: Phaser.Scene): void {
  const size = 520 * DPR;
  const g = scene.add.graphics();
  const cx = size / 2;
  for (let i = 0; i < 12; i++) {
    const a0 = (i / 12) * Math.PI * 2;
    g.fillStyle(0xffe08a, 0.16);
    g.slice(cx, cx, size / 2, a0, a0 + Math.PI / 14);
    g.fillPath();
  }
  g.generateTexture('rays', size, size);
  g.destroy();
}
