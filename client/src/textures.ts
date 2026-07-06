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

const SKY_STOPS: [number, string][] = [
  [0, '#33203c'],
  [0.38, '#6d3a4e'],
  [0.72, '#c66240'],
  [1, '#f2a63c'],
];

/**
 * Procedural "Claymation Dusk" backdrop textures: the 4-stop sky gradient,
 * two hill silhouettes for parallax, and a radial spotlight. Canvas
 * textures so gradients render identically on WebGL and Canvas. All are
 * generated at the canvas backing resolution (logical × DPR); consumers
 * draw them with `setDisplaySize` in logical coordinates.
 */
export function ensureBackdropTextures(scene: Phaser.Scene): void {
  if (scene.textures.exists('dusk-sky')) return;
  const { width, height } = scene.scale;

  const sky = scene.textures.createCanvas('dusk-sky', width, height);
  if (sky) {
    const ctx = sky.getContext();
    const grad = ctx.createLinearGradient(0, 0, 0, height);
    for (const [offset, color] of SKY_STOPS) grad.addColorStop(offset, color);
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, height);
    sky.refresh();
  }

  hillTexture(scene, 'hill-far', width, 150 * DPR, 0x5a3550, 3);
  hillTexture(scene, 'hill-mid', width, 110 * DPR, 0x6b4a3a, 5);

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

/** Rounded hill silhouette strip drawn with overlapping ellipses. */
function hillTexture(
  scene: Phaser.Scene,
  key: string,
  width: number,
  height: number,
  color: number,
  bumps: number,
): void {
  const g = scene.add.graphics();
  g.fillStyle(color, 1);
  g.fillRect(0, height * 0.55, width, height * 0.45);
  for (let i = 0; i <= bumps; i++) {
    const cx = (i / bumps) * width;
    const rx = width / bumps / 1.3;
    const ry = height * (0.45 + 0.25 * Math.sin(i * 2.7));
    g.fillEllipse(cx, height * 0.6, rx * 2, ry * 2);
  }
  g.generateTexture(key, width, height);
  g.destroy();
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
