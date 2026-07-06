import Phaser from 'phaser';
import { addRoller } from '../rollers';
import type { GameScene } from './GameScene';
import {
  INK,
  MONO_FONT,
  NOTE_ON_CREAM,
  UI_FONT,
  button,
  pill,
  setupCamera,
  VIEW_W,
} from '../ui';

interface HudData {
  levelName: string;
  rollerName: string;
  style: string;
}

/**
 * HUD overlay running on top of the game scene. A separate scene keeps the
 * HUD on a static camera: the game camera's DPR zoom and follow scroll make
 * scrollFactor-0 objects unusable for fixed UI (zoom pivots them around the
 * canvas center, and input hit-testing drifts from the drawn position).
 */
export class GameHudScene extends Phaser.Scene {
  private hud!: HudData;

  constructor() {
    super('hud');
  }

  init(data: HudData): void {
    this.hud = data;
  }

  create(): void {
    setupCamera(this);
    const game = this.scene.get('game') as GameScene;

    const name = this.add
      .text(0, -9, this.hud.levelName, {
        fontFamily: UI_FONT,
        fontSize: '18px',
        fontStyle: '600',
        color: INK,
      })
      .setOrigin(0, 0.5);
    const roller = this.add
      .text(0, 11, this.hud.rollerName.toUpperCase(), {
        fontFamily: MONO_FONT,
        fontSize: '10px',
        color: NOTE_ON_CREAM,
      })
      .setOrigin(0, 0.5)
      .setLetterSpacing(2);
    const pillWidth = Math.max(name.width, roller.width) + 92;
    const textX = -pillWidth / 2 + 56;
    name.setX(textX);
    roller.setX(textX);
    const badge = pill(this, 18 + pillWidth / 2, 40, pillWidth, 52);
    badge.add(
      addRoller(this, -pillWidth / 2 + 30, 0, 38, this.hud.style, 'determined'),
    );
    badge.add([name, roller]);

    // Consume taps so the game scene below does not read them as jumps.
    const guard = (btn: Phaser.GameObjects.Container) => {
      btn.list
        .filter((child) => child.input)
        .forEach((child) =>
          child.on('pointerdown', () => this.input.stopPropagation()),
        );
    };
    guard(
      button(this, VIEW_W - 52, 44, '↻', () => game.scene.restart(), {
        width: 56,
        small: true,
        variant: 'cream',
      }),
    );
    guard(
      button(this, VIEW_W - 120, 44, '❚❚', () => game.togglePause(), {
        width: 56,
        small: true,
        variant: 'cream',
      }),
    );
  }
}
