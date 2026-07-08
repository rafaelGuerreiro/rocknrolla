import Phaser from 'phaser';
import { characterBodyKey, faceKey, type FaceName } from './content';

export type { FaceName };

/**
 * Two-layer roller composition. Bodies are faceless and rotate with the
 * physics; faces are a shared set of expressions layered on top, kept
 * upright and swapped on events. All art comes from the database
 * (`vw_character_art_v1` / `vw_face_v1`), rasterized once by BootScene.
 */

/** Face proportions relative to the body size (from the design mockup). */
export const FACE_WIDTH_RATIO = 0.5;
export const FACE_ASPECT = 110 / 176;
export const FACE_OFFSET_Y_RATIO = 0.04;

/**
 * Compose a body + upright face at a given display size. The returned
 * container can bob/scale; for physics-driven rollers keep the layers as
 * separate scene objects instead (see GameScene) so only the body spins.
 */
export function addRoller(
  scene: Phaser.Scene,
  x: number,
  y: number,
  size: number,
  characterId: string,
  expr: FaceName = 'happy',
): Phaser.GameObjects.Container {
  const bodyImage = scene.add
    .image(0, 0, characterBodyKey(characterId))
    .setDisplaySize(size, size);
  const faceWidth = size * FACE_WIDTH_RATIO;
  const faceImage = scene.add
    .image(0, size * FACE_OFFSET_Y_RATIO, faceKey(expr))
    .setDisplaySize(faceWidth, faceWidth * FACE_ASPECT);
  const container = scene.add.container(x, y, [bodyImage, faceImage]);
  container.setSize(size, size);
  return container;
}
