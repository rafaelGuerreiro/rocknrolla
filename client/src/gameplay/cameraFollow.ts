import Phaser from 'phaser';

/** Horizontal screen anchor for the player: one third of the viewport width. */
const ANCHOR_X_FRACTION = 1 / 3;
/** Vertical screen anchor for the player: the viewport center. */
const ANCHOR_Y_FRACTION = 1 / 2;

/**
 * Keep the target's body center exactly at (width/3, height/2) of the
 * viewport with no lerp, deadzone, or look-ahead. The scroll is computed
 * from the live camera size each frame, so resizes and orientation changes
 * are handled automatically. Camera shake still works: it offsets the view
 * matrix, not the scroll. Deliberately no scroll clamping — keeping the
 * anchor from spawn through finish takes priority over hiding the void
 * beyond level edges.
 */
export class CameraFollow {
  constructor(
    private readonly scene: Phaser.Scene,
    private readonly target: Phaser.GameObjects.Components.Transform,
  ) {
    scene.events.on(Phaser.Scenes.Events.UPDATE, this.follow);
    scene.events.once(Phaser.Scenes.Events.SHUTDOWN, this.destroy);
    this.follow();
  }

  private follow = (): void => {
    const camera = this.scene.cameras.main;
    camera.setScroll(
      this.target.x - camera.width * ANCHOR_X_FRACTION,
      this.target.y - camera.height * ANCHOR_Y_FRACTION,
    );
  };

  destroy = (): void => {
    this.scene.events.off(Phaser.Scenes.Events.UPDATE, this.follow);
  };
}
