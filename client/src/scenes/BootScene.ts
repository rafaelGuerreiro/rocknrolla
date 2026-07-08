import Phaser from 'phaser';
import { buildContentIndex, rasterSize } from '../content';
import { connect } from '../db';
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

/** Raster size for the client-owned finish flag (2× on-screen size). */
const TILE_RASTER = 128;

const BAR_WIDTH = 320;
const BAR_HEIGHT = 18;

/**
 * Plain solid-color loading screen: no art exists before the subscription
 * applies — every texture (characters, faces, backdrops, components) comes
 * from the database. The scene connects, indexes the content rows, and
 * rasterizes the content textures before handing off to the menu.
 */
export class BootScene extends Phaser.Scene {
  private failedFiles: string[] = [];
  private status?: Phaser.GameObjects.Text;
  private barFill?: Phaser.GameObjects.Graphics;

  constructor() {
    super('boot');
  }

  preload(): void {
    this.failedFiles = [];
    // The finish flag is level-owned client chrome, not authored content.
    for (const [key, svg] of Object.entries(TILE_SVG)) {
      this.load.svg(key, svgDataUrl(svg), {
        width: TILE_RASTER,
        height: TILE_RASTER,
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
    void Promise.all(FONT_FACES.map((face) => document.fonts.load(face)))
      .catch(() => undefined) // offline: fall back to system fonts
      .then(() => this.buildUiAndConnect());
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
    if (!this.barFill) return;
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
  }

  private connectAndLoad(): void {
    connect()
      .then((conn) => {
        this.setProgress(0.7);
        if (
          conn.db.vw_level_v1.count() === 0n ||
          conn.db.vw_character_v1.count() === 0n ||
          conn.db.vw_character_art_v1.count() === 0n ||
          conn.db.vw_face_v1.count() === 0n ||
          conn.db.vw_backdrop_v1.count() === 0n
        ) {
          this.fail(
            'CONNECTED, BUT NO CONTENT IS IMPORTED.\nRUN task server:admin AND IMPORT, THEN TAP TO RETRY.',
          );
          return;
        }
        let textures;
        try {
          textures = buildContentIndex(conn).textures;
        } catch (error) {
          this.fail(
            `BAD CONTENT: ${error instanceof Error ? error.message : String(error)}`,
          );
          return;
        }
        const missing = textures.filter((t) => !this.textures.exists(t.key));
        if (missing.length === 0) {
          this.finishLoading();
          return;
        }
        for (const texture of missing) {
          this.load.svg(texture.key, svgDataUrl(texture.svg), {
            ...rasterSize(texture),
          });
        }
        this.load.on(Phaser.Loader.Events.PROGRESS, (t: number) =>
          this.setProgress(0.7 + 0.3 * t),
        );
        this.load.once(Phaser.Loader.Events.COMPLETE, () =>
          this.finishLoading(),
        );
        this.load.start();
      })
      .catch((error: Error) => {
        this.fail(`CONNECTION FAILED: ${error.message}`);
      });
  }

  private finishLoading(): void {
    if (this.failedFiles.length > 0) {
      this.fail(`FAILED TO LOAD CONTENT ART: ${this.failedFiles.join(', ')}`);
      return;
    }
    this.setProgress(1);
    this.scene.start('level-select');
  }

  private fail(message: string): void {
    this.status?.setText(`${message}\nTAP TO RETRY.`);
    this.input.once('pointerdown', () => this.scene.restart());
  }
}
