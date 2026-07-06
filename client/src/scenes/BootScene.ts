import Phaser from 'phaser';
import { characterSpriteKey, queueSprites, TILE_SPRITES } from '../assets';
import { connect } from '../db';
import { UI_FONT } from '../ui';

export class BootScene extends Phaser.Scene {
  private failedFiles: string[] = [];

  constructor() {
    super('boot');
  }

  preload(): void {
    this.failedFiles = [];
    queueSprites(this.load, Object.values(TILE_SPRITES));
    this.load.on(
      Phaser.Loader.Events.FILE_LOAD_ERROR,
      (file: Phaser.Loader.File) => {
        this.failedFiles.push(file.key);
      },
    );
  }

  create(): void {
    const status = this.add
      .text(this.scale.width / 2, this.scale.height / 2, 'Connecting…', {
        fontFamily: UI_FONT,
        fontSize: '26px',
        color: '#e8ecf5',
        align: 'center',
      })
      .setOrigin(0.5);

    if (this.failedFiles.length > 0) {
      status.setText(
        `Failed to load game art:\n${this.failedFiles.join(', ')}\nTap to retry.`,
      );
      this.retryOnTap();
      return;
    }

    connect()
      .then((conn) => {
        if (
          conn.db.vw_level.count() === 0n ||
          conn.db.vw_character.count() === 0n
        ) {
          status.setText(
            'Connected, but no content is imported.\nRun task server:admin and import, then tap to retry.',
          );
          this.retryOnTap();
          return;
        }
        // Character sprites come from the same canonical Kenney tree.
        const styles = [...conn.db.vw_character.iter()].map((row) => row.style);
        queueSprites(this.load, styles.map(characterSpriteKey));
        this.load.once(Phaser.Loader.Events.COMPLETE, () => {
          if (this.failedFiles.length > 0) {
            status.setText(
              `Failed to load character art:\n${this.failedFiles.join(', ')}\nTap to retry.`,
            );
            this.retryOnTap();
            return;
          }
          this.scene.start('level-select');
        });
        this.load.start();
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
