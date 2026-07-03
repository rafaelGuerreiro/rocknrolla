import Phaser from 'phaser';
import { connect } from '../db';
import { ensureTextures } from '../textures';
import { UI_FONT } from '../ui';

export class BootScene extends Phaser.Scene {
  constructor() {
    super('boot');
  }

  create(): void {
    ensureTextures(this);
    const status = this.add
      .text(this.scale.width / 2, this.scale.height / 2, 'Connecting…', {
        fontFamily: UI_FONT,
        fontSize: '26px',
        color: '#e8ecf5',
        align: 'center',
      })
      .setOrigin(0.5);

    connect()
      .then((conn) => {
        if (conn.db.level.count() === 0n || conn.db.character_def.count() === 0n) {
          status.setText('Connected, but no content is imported.\nRun task server:levels-import, then tap to retry.');
          this.retryOnTap();
          return;
        }
        this.scene.start('level-select');
      })
      .catch((error: Error) => {
        status.setText(`Connection failed:\n${error.message}\nTap to retry.`);
        this.retryOnTap();
      });
  }

  private retryOnTap(): void {
    this.input.once('pointerdown', () => this.scene.restart());
  }
}
