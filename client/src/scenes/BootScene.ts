import Phaser from 'phaser';
import { characterSpriteKey } from '../assets';
import { connect } from '../db';
import {
  FACE_SVG,
  ROLLER_BODY_SVG,
  addRoller,
  rollerBodyDataUrl,
  rollerSilhouetteDataUrl,
  silhouetteTextureKey,
} from '../rollers';
import { ensureBackdropTextures } from '../textures';
import { svgDataUrl, TILE_SVG } from '../tiles';
import {
  CREAM_TEXT,
  MONO_FONT,
  setupCamera,
  UI_FONT,
  VIEW_H,
  VIEW_W,
} from '../ui';

/** Fonts the theme needs before any text is painted (Phaser never reflows). */
const FONT_FACES = [
  '600 24px Fredoka',
  '700 34px Fredoka',
  '600 18px Nunito',
  '400 14px "Space Mono"',
];

/** SVG raster sizes: 2× their max on-screen size so they stay retina-crisp. */
const TILE_RASTER = 128;
const ROLLER_RASTER = 320;
const FACE_RASTER_W = 176;
const FACE_RASTER_H = 110;

const BAR_WIDTH = 320;
const BAR_HEIGHT = 18;

export class BootScene extends Phaser.Scene {
  private failedFiles: string[] = [];
  private status?: Phaser.GameObjects.Text;
  private barFill?: Phaser.GameObjects.Graphics;
  private barRider?: Phaser.GameObjects.Image;

  constructor() {
    super('boot');
  }

  preload(): void {
    this.failedFiles = [];
    for (const [key, svg] of Object.entries(TILE_SVG)) {
      this.load.svg(key, svgDataUrl(svg), {
        width: TILE_RASTER,
        height: TILE_RASTER,
      });
    }
    for (const style of Object.keys(ROLLER_BODY_SVG)) {
      this.load.svg(characterSpriteKey(style), rollerBodyDataUrl(style), {
        width: ROLLER_RASTER,
        height: ROLLER_RASTER,
      });
      this.load.svg(
        silhouetteTextureKey(style),
        rollerSilhouetteDataUrl(style),
        { width: 128, height: 128 },
      );
    }
    for (const [name, svg] of Object.entries(FACE_SVG)) {
      this.load.svg(`face_${name}`, svgDataUrl(svg), {
        width: FACE_RASTER_W,
        height: FACE_RASTER_H,
      });
    }
    this.load.on(
      Phaser.Loader.Events.FILE_LOAD_ERROR,
      (file: Phaser.Loader.File) => {
        this.failedFiles.push(file.key);
      },
    );
  }

  create(): void {
    setupCamera(this);
    this.drawBackdrop();
    void Promise.all(FONT_FACES.map((face) => document.fonts.load(face)))
      .catch(() => undefined) // offline: fall back to system fonts
      .then(() => this.buildUiAndConnect());
  }

  private drawBackdrop(): void {
    ensureBackdropTextures(this);
    this.add
      .image(VIEW_W / 2, VIEW_H / 2, 'dusk-sky')
      .setDisplaySize(VIEW_W, VIEW_H);
    this.add
      .image(VIEW_W / 2, VIEW_H - 60, 'hill-far')
      .setDisplaySize(VIEW_W, 150)
      .setAlpha(0.9);
    this.add
      .image(VIEW_W / 2, VIEW_H - 30, 'hill-mid')
      .setDisplaySize(VIEW_W, 110);

    const rocco = addRoller(this, VIEW_W / 2, VIEW_H / 2 - 96, 92, 'rock');
    this.tweens.add({
      targets: rocco,
      y: rocco.y - 9,
      duration: 1500,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
    });
  }

  private buildUiAndConnect(): void {
    const centerX = VIEW_W / 2;

    this.add
      .text(centerX, VIEW_H / 2 - 18, 'RocknRolla', {
        fontFamily: UI_FONT,
        fontSize: '52px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 4, 'rgba(36,29,22,0.6)', 6);
    this.add
      .text(centerX, VIEW_H / 2 + 24, 'A DOWNHILL PHYSICS ROLLER', {
        fontFamily: MONO_FONT,
        fontSize: '13px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(4);

    const barY = VIEW_H / 2 + 78;
    const track = this.add.graphics();
    track.fillStyle(0x1e101a, 0.4);
    track.fillRoundedRect(
      centerX - BAR_WIDTH / 2,
      barY - BAR_HEIGHT / 2,
      BAR_WIDTH,
      BAR_HEIGHT,
      BAR_HEIGHT / 2,
    );
    this.barFill = this.add.graphics();
    this.barRider = this.add
      .image(centerX - BAR_WIDTH / 2, barY - 18, characterSpriteKey('rock'))
      .setDisplaySize(26, 26);
    this.status = this.add
      .text(centerX, barY + 32, 'CONNECTING TO BASECAMP…', {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: '#ffe0a3',
        align: 'center',
      })
      .setOrigin(0.5)
      .setLetterSpacing(3);
    this.setProgress(0.15);

    if (this.failedFiles.length > 0) {
      this.fail(`FAILED TO LOAD ART: ${this.failedFiles.join(', ')}`);
      return;
    }
    this.connectAndLoad();
  }

  private setProgress(t: number): void {
    if (!this.barFill || !this.barRider) return;
    const barY = VIEW_H / 2 + 78;
    const w = Math.max(BAR_HEIGHT, BAR_WIDTH * Phaser.Math.Clamp(t, 0, 1));
    this.barFill.clear();
    this.barFill.fillGradientStyle(0xffce7a, 0xf2932f, 0xffce7a, 0xf2932f, 1);
    this.barFill.fillRoundedRect(
      VIEW_W / 2 - BAR_WIDTH / 2,
      barY - BAR_HEIGHT / 2,
      w,
      BAR_HEIGHT,
      BAR_HEIGHT / 2,
    );
    this.barRider.setX(VIEW_W / 2 - BAR_WIDTH / 2 + w);
  }

  private connectAndLoad(): void {
    connect()
      .then((conn) => {
        this.setProgress(0.7);
        if (
          conn.db.vw_level_v1.count() === 0n ||
          conn.db.vw_character_v1.count() === 0n
        ) {
          this.fail(
            'CONNECTED, BUT NO CONTENT IS IMPORTED.\nRUN task server:admin AND IMPORT, THEN TAP TO RETRY.',
          );
          return;
        }
        // Roller textures for every server style (unknown styles fall back).
        const styles = [...conn.db.vw_character_v1.iter()].map(
          (row) => row.style,
        );
        for (const style of styles) {
          if (!this.textures.exists(characterSpriteKey(style))) {
            this.load.svg(characterSpriteKey(style), rollerBodyDataUrl(style), {
              width: ROLLER_RASTER,
              height: ROLLER_RASTER,
            });
            this.load.svg(
              silhouetteTextureKey(style),
              rollerSilhouetteDataUrl(style),
              { width: 128, height: 128 },
            );
          }
        }
        this.load.on(Phaser.Loader.Events.PROGRESS, (t: number) =>
          this.setProgress(0.7 + 0.3 * t),
        );
        this.load.once(Phaser.Loader.Events.COMPLETE, () => {
          if (this.failedFiles.length > 0) {
            this.fail(
              `FAILED TO LOAD CHARACTER ART: ${this.failedFiles.join(', ')}`,
            );
            return;
          }
          this.setProgress(1);
          this.scene.start('level-select');
        });
        this.load.start();
      })
      .catch((error: Error) => {
        this.fail(`CONNECTION FAILED: ${error.message}`);
      });
  }

  private fail(message: string): void {
    this.status?.setText(`${message}\nTAP TO RETRY.`);
    this.input.once('pointerdown', () => this.scene.restart());
  }
}
