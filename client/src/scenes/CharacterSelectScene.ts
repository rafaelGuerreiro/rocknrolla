import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
import { addRoller } from '../rollers';
import { db } from '../db';
import { ensureBackdropTextures } from '../textures';
import {
  BODY_FONT,
  button,
  characterStatBars,
  characterTrait,
  CREAM_TEXT,
  INK,
  MONO_FONT,
  NOTE_ON_CREAM,
  note,
  setupCamera,
  statBars,
  UI_FONT,
  VIEW_H,
  VIEW_W,
} from '../ui';

interface CharacterRow {
  id: { toString(): string };
  name: string;
  style: string;
  density: number;
  jumpSpeed: number;
  flightTimeMs: number;
  buoyancy: number;
  fireResistance: number;
}

export class CharacterSelectScene extends Phaser.Scene {
  private levelId!: string;
  private selectedId?: string;

  constructor() {
    super('character-select');
  }

  init(data: { levelId: string; selectedId?: string }): void {
    this.levelId = data.levelId;
    this.selectedId = data.selectedId;
  }

  create(): void {
    const width = VIEW_W;
    const height = VIEW_H;
    setupCamera(this);
    const conn = db();
    ensureBackdropTextures(this);
    this.add
      .image(width / 2, height / 2, 'spotlight')
      .setDisplaySize(width, height);

    const unlocked = new Set(
      [...conn.db.vw_my_unlocked_character.iter()].map((row) =>
        row.characterId.toString(),
      ),
    );
    const characters = [...conn.db.vw_character.iter()].sort((a, b) =>
      a.id.toString().localeCompare(b.id.toString()),
    );
    const eligible = characters.filter((character) =>
      unlocked.has(character.id.toString()),
    );

    // One eligible character is not a choice: proceed with it directly.
    if (eligible.length === 1) {
      this.startRun(eligible[0].id.toString());
      return;
    }
    if (eligible.length === 0) {
      note(
        this,
        height / 2,
        'No unlocked characters. Collect pieces to unlock one.',
      );
      this.backButton();
      return;
    }

    const me = [...conn.db.vw_me.iter()][0];
    const remembered = me?.selectedCharacterId?.toString();
    const selected =
      eligible.find((c) => c.id.toString() === this.selectedId) ??
      eligible.find((c) => c.id.toString() === remembered) ??
      eligible[0];

    this.rosterRail(characters, unlocked, selected);
    this.hero(selected);
    this.statPanel(selected);
    this.backButton();
    button(
      this,
      width / 2,
      height - 52,
      'ROLL OUT ▸',
      () => this.startRun(selected.id.toString()),
      { width: 280 },
    );
  }

  /** Left rail of roller chips; tapping an unlocked chip reselects. */
  private rosterRail(
    characters: CharacterRow[],
    unlocked: Set<string>,
    selected: CharacterRow,
  ): void {
    characters.forEach((character, index) => {
      const x = 66;
      const y = 96 + index * 62;
      const id = character.id.toString();
      const isUnlocked = unlocked.has(id);
      const isSelected = id === selected.id.toString();

      const g = this.add.graphics();
      g.fillStyle(isUnlocked ? 0xf5ecd8 : 0x2e1c34, isUnlocked ? 1 : 0.7);
      g.fillRoundedRect(x - 26, y - 26, 52, 52, 14);
      if (isSelected) {
        g.lineStyle(3, 0xf2a63c, 1);
        g.strokeRoundedRect(x - 28, y - 28, 56, 56, 15);
      }
      if (isUnlocked) {
        addRoller(this, x, y, 44, character.style);
        this.add
          .rectangle(x, y, 56, 56, 0xffffff, 0.0001)
          .setInteractive({ useHandCursor: true })
          .on('pointerup', () =>
            this.scene.restart({ levelId: this.levelId, selectedId: id }),
          );
      } else {
        this.add
          .text(x, y, '?', {
            fontFamily: UI_FONT,
            fontSize: '24px',
            fontStyle: '700',
            color: '#8a6a7a',
          })
          .setOrigin(0.5);
      }
    });
  }

  private hero(character: CharacterRow): void {
    const x = VIEW_W / 2 - 40;
    const y = 220;
    const glow = this.add.graphics();
    glow.fillStyle(0xffce7a, 0.18);
    glow.fillCircle(x, y, 110);
    this.add.ellipse(x, y + 84, 130, 26, 0x241d16, 0.35);

    const sprite = addRoller(this, x, y, 140, character.style);
    this.tweens.add({
      targets: sprite,
      y: y - 9,
      duration: 1500,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
    });

    this.add
      .text(x, y + 118, character.name, {
        fontFamily: UI_FONT,
        fontSize: '32px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 3, 'rgba(36,29,22,0.55)', 4);
    this.add
      .text(x, y + 150, characterTrait(character), {
        fontFamily: BODY_FONT,
        fontSize: '15px',
        fontStyle: '700',
        color: '#e6c9a0',
      })
      .setOrigin(0.5);
  }

  private statPanel(character: CharacterRow): void {
    const x = VIEW_W - 168;
    const y = 210;
    const w = 264;
    const h = 240;
    const g = this.add.graphics();
    g.fillStyle(0x140a12, 0.3);
    g.fillRoundedRect(x - w / 2 + 3, y - h / 2 + 8, w, h, 20);
    g.fillStyle(0xf5ecd8, 1);
    g.fillRoundedRect(x - w / 2, y - h / 2, w, h, 20);

    this.add
      .text(x - w / 2 + 24, y - h / 2 + 26, `BUILD · ${character.name}`, {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: NOTE_ON_CREAM,
      })
      .setOrigin(0, 0.5)
      .setLetterSpacing(2);
    statBars(
      this,
      x - w / 2 + 24,
      y - h / 2 + 62,
      characterStatBars(character),
      { rowGap: 42 },
    );
    this.add
      .text(
        x - w / 2 + 24,
        y + h / 2 - 24,
        `HOLD ${character.flightTimeMs}MS`,
        {
          fontFamily: MONO_FONT,
          fontSize: '11px',
          color: INK,
        },
      )
      .setOrigin(0, 0.5)
      .setLetterSpacing(2);
  }

  private backButton(): void {
    button(this, 84, 40, '‹ BACK', () => this.scene.start('level-select'), {
      width: 120,
      small: true,
      variant: 'cream',
    });
  }

  private startRun(characterId: string): void {
    const conn = db();
    conn.reducers
      .selectCharacter({ characterId: Uuid.parse(characterId) })
      .catch((error) => console.error('selectCharacter failed:', error));
    this.scene.start('game', { levelId: this.levelId, characterId });
  }
}
