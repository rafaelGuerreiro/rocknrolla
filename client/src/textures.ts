import Phaser from 'phaser';

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
