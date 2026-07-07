import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
import { addRoller, silhouetteTextureKey } from '../rollers';
import { db } from '../db';
import { openLootboxAndAwaitPiece } from '../lootbox';
import { ensureBackdropTextures } from '../textures';
import {
  BODY_FONT,
  button,
  characterStatBars,
  characterTrait,
  CREAM_TEXT,
  INK,
  MONO_FONT,
  note,
  pill,
  setupCamera,
  STAT_ACCENTS,
  UI_FONT,
  VIEW_H,
  VIEW_W,
} from '../ui';

const CARD_W = 272;
const CARD_H = 172;
const GRID_COLS = 3;

/** The roller collection grid plus unopened-lootbox opening. */
export class CollectionScene extends Phaser.Scene {
  private opening = false;

  constructor() {
    super('collection');
  }

  create(): void {
    this.opening = false;
    const width = VIEW_W;
    const height = VIEW_H;
    setupCamera(this);
    ensureBackdropTextures(this);
    this.add
      .image(width / 2, height / 2, 'dusk-sky')
      .setDisplaySize(width, height);

    const conn = db();
    button(this, 84, 40, '‹ BACK', () => this.scene.start('level-select'), {
      width: 120,
      small: true,
      variant: 'cream',
    });
    this.add
      .text(width / 2, 40, 'Your Rollers', {
        fontFamily: UI_FONT,
        fontSize: '30px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 3, 'rgba(36,29,22,0.55)', 4);

    const unlocked = new Set(
      [...conn.db.vw_my_unlocked_character_v1.iter()].map((row) =>
        row.characterId.toString(),
      ),
    );
    const characters = [...conn.db.vw_character_v1.iter()].sort((a, b) =>
      a.id.compareTo(b.id),
    );
    pill(
      this,
      width - 110,
      40,
      170,
      40,
      `${unlocked.size} / ${characters.length} unlocked`,
    );

    const unopened = [...conn.db.vw_my_lootbox_v1.iter()]
      .filter((row) => !row.opened)
      .sort((a, b) => a.id.compareTo(b.id));
    if (unopened.length > 0) {
      button(
        this,
        width / 2,
        88,
        `OPEN LOOTBOX (${unopened.length}) ▸`,
        () => this.openLootbox(unopened[0].id),
        { width: 300, small: true },
      );
    }

    characters.forEach((character, index) => {
      const col = index % GRID_COLS;
      const row = Math.floor(index / GRID_COLS);
      const x = width / 2 + (col - 1) * (CARD_W + 16);
      const y = 200 + row * (CARD_H + 18);
      if (unlocked.has(character.id.toString())) {
        this.unlockedCard(character, x, y);
      } else {
        this.lockedCard(character, x, y);
      }
    });
  }

  private unlockedCard(
    character: {
      name: string;
      style: string;
      density: number;
      jumpSpeed: number;
      buoyancy: number;
      fireResistance: number;
    },
    x: number,
    y: number,
  ): void {
    const g = this.add.graphics();
    g.fillStyle(0x140a12, 0.3);
    g.fillRoundedRect(
      x - CARD_W / 2 + 3,
      y - CARD_H / 2 + 8,
      CARD_W,
      CARD_H,
      18,
    );
    g.fillStyle(0xf5ecd8, 1);
    g.fillRoundedRect(x - CARD_W / 2, y - CARD_H / 2, CARD_W, CARD_H, 18);

    addRoller(this, x - CARD_W / 2 + 48, y - 22, 58, character.style);
    this.add
      .text(x - CARD_W / 2 + 88, y - CARD_H / 2 + 30, character.name, {
        fontFamily: UI_FONT,
        fontSize: '22px',
        fontStyle: '600',
        color: INK,
      })
      .setOrigin(0, 0.5);

    // Signature stat: the character's strongest build dimension.
    const signature = characterStatBars(character).reduce((a, b) =>
      b.value > a.value ? b : a,
    );
    const dots = this.add.graphics();
    const accent = STAT_ACCENTS[signature.label] ?? 0xc66240;
    for (let i = 0; i < 5; i++) {
      dots.fillStyle(i < signature.value ? accent : 0xe2d6bd, 1);
      dots.fillCircle(x - CARD_W / 2 + 94 + i * 18, y - 18, 6);
    }
    this.add
      .text(x - CARD_W / 2 + 88, y - 40, signature.label.toUpperCase(), {
        fontFamily: MONO_FONT,
        fontSize: '10px',
        color: '#9a7d5c',
      })
      .setOrigin(0, 0.5)
      .setLetterSpacing(2);

    const ready = this.add.graphics();
    ready.fillStyle(0x6d7a44, 1);
    ready.fillRoundedRect(x - CARD_W / 2 + 22, y + CARD_H / 2 - 46, 74, 26, 13);
    this.add
      .text(x - CARD_W / 2 + 59, y + CARD_H / 2 - 33, 'READY', {
        fontFamily: MONO_FONT,
        fontSize: '11px',
        color: '#f5ecd8',
      })
      .setOrigin(0.5)
      .setLetterSpacing(2);
    this.add
      .text(
        x + CARD_W / 2 - 22,
        y + CARD_H / 2 - 33,
        characterTrait(character),
        {
          fontFamily: MONO_FONT,
          fontSize: '10px',
          color: '#9a7d5c',
        },
      )
      .setOrigin(1, 0.5)
      .setLetterSpacing(1);
  }

  private lockedCard(
    character: { id: Uuid; name: string; style: string },
    x: number,
    y: number,
  ): void {
    const conn = db();
    const g = this.add.graphics();
    g.fillStyle(0x2e1c34, 0.8);
    g.fillRoundedRect(x - CARD_W / 2, y - CARD_H / 2, CARD_W, CARD_H, 18);
    g.lineStyle(2, 0x8a6a7a, 0.5);
    g.strokeRoundedRect(x - CARD_W / 2, y - CARD_H / 2, CARD_W, CARD_H, 18);

    this.add
      .image(x, y - 28, silhouetteTextureKey(character.style))
      .setDisplaySize(58, 58)
      .setAlpha(0.8);
    this.add
      .text(x, y + 14, character.name, {
        fontFamily: UI_FONT,
        fontSize: '20px',
        fontStyle: '600',
        color: '#c9b8a4',
      })
      .setOrigin(0.5);

    const pieces = [...conn.db.vw_piece_v1.iter()].filter(
      (row) => row.characterId.compareTo(character.id) === 0,
    );
    const owned = new Set(
      [...conn.db.vw_my_piece_v1.iter()]
        .filter((row) => row.count > 0)
        .map((row) => row.pieceId.toString()),
    );
    const have = pieces.filter((p) => owned.has(p.id.toString())).length;

    const size = 14;
    const gap = 8;
    const startX = x - ((size + gap) * pieces.length - gap) / 2;
    pieces.forEach((_, i) => {
      if (i < have) {
        g.fillStyle(0xf2a63c, 1);
        g.fillRoundedRect(startX + i * (size + gap), y + 32, size, size, 4);
      } else {
        g.lineStyle(2, 0xe6c9a0, 0.45);
        g.strokeRoundedRect(startX + i * (size + gap), y + 32, size, size, 4);
      }
    });
    this.add
      .text(x, y + 62, `${have} / ${pieces.length} PIECES`, {
        fontFamily: MONO_FONT,
        fontSize: '11px',
        color: '#f2a63c',
      })
      .setOrigin(0.5)
      .setLetterSpacing(2);
  }

  private openLootbox(playerLootboxId: Uuid): void {
    if (this.opening) return;
    this.opening = true;
    openLootboxAndAwaitPiece(playerLootboxId)
      .then((pieceId) => this.reveal(pieceId))
      .catch((error: Error) => {
        this.opening = false;
        note(this, 126, `Could not open the lootbox: ${error.message}`);
      });
  }

  /** Animate the server-decided award; the client never chooses the piece. */
  private reveal(pieceId: Uuid): void {
    const conn = db();
    const piece = [...conn.db.vw_piece_v1.iter()].find(
      (row) => row.id.compareTo(pieceId) === 0,
    );
    const character = piece
      ? [...conn.db.vw_character_v1.iter()].find(
          (row) => row.id.compareTo(piece.characterId) === 0,
        )
      : undefined;
    const centerX = VIEW_W / 2;
    const centerY = VIEW_H / 2;

    const dim = this.add
      .rectangle(centerX, centerY, VIEW_W, VIEW_H, 0x241d16, 0.88)
      .setDepth(10)
      .setInteractive();
    if (character) {
      addRoller(this, centerX, centerY - 96, 84, character.style).setDepth(11);
    }
    const label = this.add
      .text(centerX, centerY - 20, piece?.name ?? pieceId.toString(), {
        fontFamily: UI_FONT,
        fontSize: '40px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setDepth(11)
      .setScale(0.1)
      .setShadow(0, 3, 'rgba(36,29,22,0.6)', 5);
    const sub = this.add
      .text(
        centerX,
        centerY + 34,
        `${character?.name ?? '?'} piece — tap to continue`,
        {
          fontFamily: BODY_FONT,
          fontSize: '18px',
          fontStyle: '600',
          color: '#e6c9a0',
        },
      )
      .setOrigin(0.5)
      .setDepth(11)
      .setAlpha(0);
    this.tweens.add({
      targets: label,
      scale: 1,
      duration: 450,
      ease: 'Back.Out',
    });
    this.tweens.add({ targets: sub, alpha: 1, delay: 350, duration: 300 });
    dim.once('pointerup', () => this.scene.restart());
  }
}
