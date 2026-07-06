import Phaser from 'phaser';
import type { Uuid } from 'spacetimedb';
import { characterSpriteKey } from '../assets';
import {
  FACE_ASPECT,
  FACE_WIDTH_RATIO,
  addRoller,
  faceTextureKey,
} from '../rollers';
import { db } from '../db';
import { openLootboxAndAwaitPiece } from '../lootbox';
import { ensureBackdropTextures } from '../textures';
import {
  BODY_FONT,
  button,
  CREAM_TEXT,
  DPR,
  INK,
  MONO_FONT,
  note,
  pill,
  setupCamera,
  STAR_GOLD,
  UI_FONT,
  VIEW_H,
  VIEW_W,
} from '../ui';

const CONFETTI_COLORS = [
  0xffce7a, 0xe85d3c, 0x6d7a44, 0x3f7d8c, 0xf2a63c, 0xa8586a,
];

interface ResultData {
  levelId: string;
  levelName: string;
  outcome?: 'success' | 'defeat';
  reason?: string;
  timeMs?: number;
}

function formatTime(ms: number): string {
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)}:${`${total % 60}`.padStart(2, '0')}`;
}

function metChip(reason: string): string {
  if (reason.includes('hazard')) return 'MET · SPIKES';
  if (reason.includes('Burned')) return 'MET · FIRE';
  if (reason.includes('fell')) return 'MET · GRAVITY';
  return 'MET · TROUBLE';
}

/** Success beat → optional lootbox-reveal beat, or the defeat beat. */
export class ResultScene extends Phaser.Scene {
  private result!: ResultData;
  private opening = false;

  constructor() {
    super('result');
  }

  init(data: ResultData): void {
    this.result = data;
    this.opening = false;
  }

  create(): void {
    setupCamera(this);
    ensureBackdropTextures(this);
    if (this.result.outcome === 'defeat') this.defeatBeat();
    else this.successBeat();
  }

  /** Style of the roller the player just ran with. */
  private heroStyle(): string {
    const conn = db();
    const me = [...conn.db.vw_me.iter()][0];
    const selected = me?.selectedCharacterId
      ? [...conn.db.vw_character.iter()].find(
          (row) => row.id.toString() === me.selectedCharacterId?.toString(),
        )
      : undefined;
    return (selected ?? [...conn.db.vw_character.iter()][0])?.style ?? 'rock';
  }

  // -- Success ---------------------------------------------------------------

  private successBeat(): void {
    const width = VIEW_W;
    const height = VIEW_H;
    this.add
      .image(width / 2, height / 2, 'spotlight')
      .setDisplaySize(width, height);
    this.add
      .image(width / 2, height - 40, 'hill-mid')
      .setDisplaySize(width, 110)
      .setAlpha(0.5);
    this.confetti(14);

    this.add
      .text(width / 2, 64, 'Hill cleared!', {
        fontFamily: UI_FONT,
        fontSize: '46px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 4, 'rgba(36,29,22,0.6)', 6);
    this.add
      .text(width / 2, 102, this.result.levelName.toUpperCase(), {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(3);

    // ponytail: no score model — a clear is always three stars
    [-64, 0, 64].forEach((dx, i) => {
      const star = this.add
        .text(width / 2 + dx, 158, '★', {
          fontFamily: UI_FONT,
          fontSize: i === 1 ? '58px' : '42px',
          color: STAR_GOLD,
        })
        .setOrigin(0.5)
        .setShadow(0, 3, 'rgba(36,29,22,0.55)', 4)
        .setScale(0);
      this.tweens.add({
        targets: star,
        scale: 1,
        delay: 150 + i * 140,
        duration: 320,
        ease: 'Back.easeOut',
      });
    });

    this.timeChip();
    this.heroWithFlag();

    const unopened = [...db().db.vw_my_lootbox.iter()]
      .filter((row) => !row.opened)
      .sort((a, b) => a.id.compareTo(b.id));
    if (unopened.length > 0) {
      this.lootboxCta(unopened[0].id);
    } else {
      button(
        this,
        VIEW_W / 2,
        height - 56,
        '‹ LEVEL SELECT',
        () => this.scene.start('level-select'),
        { width: 280, variant: 'cream' },
      );
    }
  }

  private timeChip(): void {
    const { timeMs, levelId } = this.result;
    if (timeMs === undefined) return;
    const key = `rocknrolla_best:${levelId}`;
    let best = timeMs;
    try {
      const saved = Number(localStorage.getItem(key));
      best = saved > 0 ? Math.min(saved, timeMs) : timeMs;
      localStorage.setItem(key, `${best}`);
    } catch {
      // Private browsing: session-only best.
    }
    const label = `TIME ${formatTime(timeMs)} · BEST ${formatTime(best)}`;
    const chip = pill(this, VIEW_W / 2, 214, 250, 34);
    chip.add(
      this.add
        .text(0, 0, label, {
          fontFamily: MONO_FONT,
          fontSize: '12px',
          color: INK,
        })
        .setOrigin(0.5)
        .setLetterSpacing(2),
    );
  }

  private heroWithFlag(): void {
    const x = VIEW_W / 2;
    const flagKey = 'tile_finish';
    if (this.textures.exists(flagKey)) {
      this.add.image(x + 56, 296, flagKey).setDisplaySize(56, 56);
    }
    const hero = addRoller(this, x - 24, 292, 76, this.heroStyle());
    this.tweens.add({
      targets: hero,
      y: hero.y - 9,
      duration: 1500,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
    });
  }

  private lootboxCta(playerLootboxId: Uuid): void {
    const width = VIEW_W;
    const height = VIEW_H;
    const x = width / 2;
    const y = height - 90;

    const glow = this.add.graphics();
    glow.fillStyle(0xffce7a, 0.22);
    glow.fillCircle(x, y, 66);

    const crate = this.add.container(x, y, [this.crateGraphics()]);
    this.tweens.add({
      targets: [crate, glow],
      scale: 1.06,
      duration: 800,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
    });
    const cta = this.add
      .text(x, y + 58, 'TAP TO OPEN ▸', {
        fontFamily: MONO_FONT,
        fontSize: '13px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(3);

    this.add
      .rectangle(x, y, 130, 130, 0xffffff, 0.0001)
      .setInteractive({ useHandCursor: true })
      .on('pointerup', () => {
        if (this.opening) return;
        this.opening = true;
        cta.setText('OPENING…');
        openLootboxAndAwaitPiece(playerLootboxId)
          .then((pieceId) => this.revealBeat(pieceId))
          .catch((error: Error) => {
            this.opening = false;
            cta.setText('TAP TO OPEN ▸');
            note(this, height - 24, error.message);
          });
      });
  }

  /** Wooden crate drawn from palette shapes (no crate art in the pack). */
  private crateGraphics(): Phaser.GameObjects.Graphics {
    const g = this.add.graphics();
    g.fillStyle(0x6a3c22, 1);
    g.fillRoundedRect(-44, -38, 88, 76, 10);
    g.fillStyle(0x8a5a34, 1);
    g.fillRoundedRect(-40, -34, 80, 68, 8);
    g.fillStyle(0x6a3c22, 1);
    g.fillRect(-40, -8, 80, 14);
    g.lineStyle(4, 0xf2a63c, 1);
    g.strokeRoundedRect(-44, -38, 88, 76, 10);
    g.fillStyle(0xffce7a, 1);
    g.fillCircle(0, -1, 7);
    return g;
  }

  // -- Lootbox reveal ----------------------------------------------------------

  private revealBeat(pieceId: Uuid): void {
    const conn = db();
    const piece = [...conn.db.vw_piece.iter()].find(
      (row) => pieceId.compareTo(row.id) === 0,
    );
    const character = piece
      ? [...conn.db.vw_character.iter()].find(
          (row) => row.id.compareTo(piece.characterId) === 0,
        )
      : undefined;

    this.tweens.killAll();
    this.children.removeAll(true);
    const width = VIEW_W;
    const height = VIEW_H;
    this.add
      .image(width / 2, height / 2, 'spotlight')
      .setDisplaySize(width, height);

    const rays = this.add
      .image(width / 2, 230, 'rays')
      .setScale(1.4 / DPR)
      .setAlpha(0.9);
    this.tweens.add({
      targets: rays,
      angle: 360,
      duration: 24000,
      repeat: -1,
      ease: 'Linear',
    });
    this.confetti(16);

    // Broken crate lids under the token.
    const lids = this.add.graphics();
    lids.fillStyle(0x6a3c22, 1);
    lids
      .slice(width / 2 - 52, 330, 40, Math.PI, Math.PI * 1.8)
      .fillPath()
      .fillStyle(0x8a5a34, 1)
      .slice(width / 2 + 52, 334, 36, Math.PI * 1.2, Math.PI * 2)
      .fillPath();

    // Piece token flies up, then bobs.
    const token = this.add.container(width / 2, 330);
    const card = this.add.graphics();
    card.fillStyle(0x140a12, 0.3);
    card.fillRoundedRect(-56, -62, 112, 132, 18);
    card.fillStyle(0xf5ecd8, 1);
    card.fillRoundedRect(-58, -66, 112, 132, 18);
    token.add(card);
    if (character) {
      token.add(addRoller(this, -2, -18, 84, character.style));
    }
    token.add(
      this.add
        .text(-2, 44, (piece?.name ?? 'PIECE').toUpperCase(), {
          fontFamily: MONO_FONT,
          fontSize: '11px',
          color: '#9a7d5c',
        })
        .setOrigin(0.5)
        .setLetterSpacing(1),
    );
    this.tweens.add({
      targets: token,
      y: 218,
      duration: 550,
      ease: 'Back.easeOut',
      onComplete: () => {
        this.tweens.add({
          targets: token,
          y: 209,
          duration: 1500,
          yoyo: true,
          repeat: -1,
          ease: 'Sine.easeInOut',
        });
      },
    });

    this.add
      .text(width / 2, 66, 'NEW PIECE!', {
        fontFamily: MONO_FONT,
        fontSize: '14px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(4);
    this.add
      .text(width / 2, 104, character?.name ?? piece?.name ?? 'Mystery', {
        fontFamily: UI_FONT,
        fontSize: '38px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 3, 'rgba(36,29,22,0.6)', 5);

    if (character) this.pieceProgress(character.id, width / 2, 360);

    button(this, width / 2, height - 56, 'COLLECT ▸', () =>
      this.scene.start('collection'),
    );
  }

  /** Filled/empty piece squares + "n / m PIECES"; escalates on unlock. */
  private pieceProgress(characterId: Uuid, x: number, y: number): void {
    const conn = db();
    const pieces = [...conn.db.vw_piece.iter()].filter(
      (row) => characterId.compareTo(row.characterId) === 0,
    );
    const owned = new Set(
      [...conn.db.vw_my_piece.iter()]
        .filter((row) => row.count > 0)
        .map((row) => row.pieceId.toString()),
    );
    const have = pieces.filter((p) => owned.has(p.id.toString())).length;

    const size = 20;
    const gap = 10;
    const startX = x - ((size + gap) * pieces.length - gap) / 2;
    const g = this.add.graphics();
    pieces.forEach((_, i) => {
      const px = startX + i * (size + gap);
      if (i < have) {
        g.fillStyle(0xf2a63c, 1);
        g.fillRoundedRect(px, y - size / 2, size, size, 6);
      } else {
        g.lineStyle(2, 0xe6c9a0, 0.6);
        g.strokeRoundedRect(px, y - size / 2, size, size, 6);
      }
    });
    this.add
      .text(x, y + 28, `${have} / ${pieces.length} PIECES`, {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(2);

    if (have === pieces.length && pieces.length > 0) {
      this.confetti(20);
      this.cameras.main.shake(200, 0.006);
      const unlocked = this.add
        .text(x, y + 62, 'UNLOCKED!', {
          fontFamily: UI_FONT,
          fontSize: '30px',
          fontStyle: '700',
          color: STAR_GOLD,
        })
        .setOrigin(0.5)
        .setShadow(0, 3, 'rgba(36,29,22,0.6)', 5)
        .setScale(0);
      this.tweens.add({
        targets: unlocked,
        scale: 1,
        duration: 380,
        ease: 'Back.easeOut',
      });
    }
  }

  // -- Defeat ------------------------------------------------------------------

  private defeatBeat(): void {
    const width = VIEW_W;
    const height = VIEW_H;
    this.add
      .image(width / 2, height / 2, 'dusk-sky')
      .setDisplaySize(width, height);
    this.add
      .image(width / 2, height - 40, 'hill-mid')
      .setDisplaySize(width, 110)
      .setAlpha(0.8);
    this.add.rectangle(width / 2, height / 2, width, height, 0x241d16, 0.55);

    // Tumbled body with an upright dizzy face — only the body rocks.
    const body = this.add
      .image(width / 2, 250, characterSpriteKey(this.heroStyle()))
      .setDisplaySize(110, 110)
      .setRotation(-0.7);
    const faceWidth = 110 * FACE_WIDTH_RATIO;
    this.add
      .image(width / 2, 254, faceTextureKey('dizzy'))
      .setDisplaySize(faceWidth, faceWidth * FACE_ASPECT);
    this.tweens.add({
      targets: body,
      rotation: -0.55,
      duration: 1800,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
    });
    // Dizzy sparkles orbiting overhead.
    [-34, 6, 40].forEach((dx, i) => {
      const spark = this.add
        .text(width / 2 + dx, 178 + (i % 2) * 12, '✦', {
          fontFamily: UI_FONT,
          fontSize: '20px',
          color: '#ffe08a',
        })
        .setOrigin(0.5);
      this.tweens.add({
        targets: spark,
        y: spark.y - 10,
        alpha: 0.3,
        duration: 700 + i * 180,
        yoyo: true,
        repeat: -1,
        ease: 'Sine.easeInOut',
      });
    });

    this.add
      .text(width / 2, 108, 'Ouch!', {
        fontFamily: UI_FONT,
        fontSize: '48px',
        fontStyle: '700',
        color: CREAM_TEXT,
      })
      .setOrigin(0.5)
      .setShadow(0, 4, 'rgba(36,29,22,0.6)', 6);
    this.add
      .text(
        width / 2,
        336,
        'No checkpoints on this hill — back to the top you roll.',
        {
          fontFamily: BODY_FONT,
          fontSize: '17px',
          fontStyle: '600',
          color: '#e6c9a0',
        },
      )
      .setOrigin(0.5);

    const chip = pill(this, width / 2, 382, 190, 34);
    chip.add(
      this.add
        .text(0, 0, metChip(this.result.reason ?? ''), {
          fontFamily: MONO_FONT,
          fontSize: '12px',
          color: INK,
        })
        .setOrigin(0.5)
        .setLetterSpacing(2),
    );

    button(
      this,
      width / 2,
      height - 64,
      '‹ LEVEL SELECT',
      () => this.scene.start('level-select'),
      { width: 300, variant: 'cream' },
    );
  }

  // -- Shared juice --------------------------------------------------------------

  private confetti(count: number): void {
    const width = VIEW_W;
    const height = VIEW_H;
    for (let i = 0; i < count; i++) {
      const shape = this.add
        .rectangle(
          Phaser.Math.Between(40, width - 40),
          Phaser.Math.Between(-140, -20),
          Phaser.Math.Between(8, 14),
          Phaser.Math.Between(8, 14),
          Phaser.Math.RND.pick(CONFETTI_COLORS),
        )
        .setDepth(50)
        .setAngle(Phaser.Math.Between(0, 90));
      this.tweens.add({
        targets: shape,
        y: height + 30,
        angle: shape.angle + Phaser.Math.Between(180, 420),
        x: shape.x + Phaser.Math.Between(-60, 60),
        duration: Phaser.Math.Between(1600, 2800),
        ease: 'Sine.easeIn',
        onComplete: () => shape.destroy(),
      });
    }
  }
}
