import Phaser from 'phaser';
import { addRoller } from '../rollers';
import { db } from '../db';
import { ensureBackdropTextures } from '../textures';
import {
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

const NODE_SIZE = 72;

export class LevelSelectScene extends Phaser.Scene {
  constructor() {
    super('level-select');
  }

  create(): void {
    const width = VIEW_W;
    const height = VIEW_H;
    setupCamera(this);
    ensureBackdropTextures(this);
    this.add
      .image(width / 2, height / 2, 'dusk-sky')
      .setDisplaySize(width, height);
    this.add
      .image(width / 2, height - 50, 'hill-far')
      .setDisplaySize(width, 150)
      .setAlpha(0.85);
    this.add
      .image(width / 2, height - 22, 'hill-mid')
      .setDisplaySize(width, 110);

    const conn = db();
    pill(this, 190, 48, 260, 44, 'Choose your hill');
    this.collectionPill();

    const enabledIds = new Set(
      [...conn.db.vw_my_enabled_level.iter()].map((row) =>
        row.levelId.toString(),
      ),
    );
    const completedAt = new Map(
      [...conn.db.vw_my_completed_level.iter()].map((row) => [
        row.levelId.toString(),
        row.completedAt,
      ]),
    );
    const completedIds = new Set(completedAt.keys());
    // Trail order: cleared hills first (in the order the player cleared
    // them), then playable ones, locked last. The server exposes no level
    // ordering, so player progress is the best progression signal we have.
    const rank = (id: string) =>
      completedIds.has(id) ? 0 : enabledIds.has(id) ? 1 : 2;
    const levels = [...conn.db.vw_level.iter()].sort((a, b) => {
      const aId = a.id.toString();
      const bId = b.id.toString();
      if (rank(aId) !== rank(bId)) return rank(aId) - rank(bId);
      if (rank(aId) === 0) {
        const diff =
          (completedAt.get(aId)?.toDate().getTime() ?? 0) -
          (completedAt.get(bId)?.toDate().getTime() ?? 0);
        if (diff !== 0) return diff;
      }
      return a.slug.localeCompare(b.slug);
    });

    if (levels.length === 0) {
      note(this, height / 2, 'No levels yet. Reconnect after importing.');
      return;
    }

    // Downhill trail: nodes descend left→right with a gentle zigzag.
    const points = levels.map((_, i) => {
      const t = levels.length === 1 ? 0 : i / (levels.length - 1);
      return new Phaser.Math.Vector2(
        150 + t * (width - 300),
        168 + t * (height - 330) + (i % 2 === 0 ? 0 : -34),
      );
    });
    this.drawTrailDots(points);

    let currentMarked = false;
    levels.forEach((level, index) => {
      const id = level.id.toString();
      const completed = completedIds.has(id);
      const enabled = enabledIds.has(id);
      const isCurrent = enabled && !completed && !currentMarked;
      if (isCurrent) currentMarked = true;
      this.levelNode(points[index], index + 1, level, {
        completed,
        enabled,
        isCurrent,
      });
    });

    this.add
      .text(width / 2, height - 26, 'TAP A HILL TO ROLL IN ▸', {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(3);
  }

  private drawTrailDots(points: Phaser.Math.Vector2[]): void {
    const g = this.add.graphics();
    g.fillStyle(0xf5ecd8, 0.45);
    for (let i = 0; i < points.length - 1; i++) {
      const from = points[i];
      const to = points[i + 1];
      const steps = Math.floor(from.distance(to) / 26);
      for (let s = 1; s < steps; s++) {
        const p = from.clone().lerp(to, s / steps);
        g.fillCircle(p.x, p.y, 3.5);
      }
    }
  }

  private levelNode(
    at: Phaser.Math.Vector2,
    number: number,
    level: { id: { toString(): string }; name: string },
    state: { completed: boolean; enabled: boolean; isCurrent: boolean },
  ): void {
    const g = this.add.graphics();
    const half = NODE_SIZE / 2;
    const radius = 20;

    if (state.isCurrent) {
      g.lineStyle(5, 0xffe08a, 0.6);
      g.strokeRoundedRect(
        at.x - half - 5,
        at.y - half - 5,
        NODE_SIZE + 10,
        NODE_SIZE + 10,
        radius + 5,
      );
    }
    if (state.completed) {
      g.fillStyle(0xb5651f, 1);
      g.fillRoundedRect(
        at.x - half,
        at.y - half + 5,
        NODE_SIZE,
        NODE_SIZE,
        radius,
      );
      g.fillGradientStyle(0xffce7a, 0xffce7a, 0xf2932f, 0xf2932f, 1);
      g.fillRoundedRect(at.x - half, at.y - half, NODE_SIZE, NODE_SIZE, radius);
    } else if (state.enabled) {
      g.fillStyle(0x9a8261, 1);
      g.fillRoundedRect(
        at.x - half,
        at.y - half + 5,
        NODE_SIZE,
        NODE_SIZE,
        radius,
      );
      g.fillStyle(0xf5ecd8, 1);
      g.fillRoundedRect(at.x - half, at.y - half, NODE_SIZE, NODE_SIZE, radius);
      g.lineStyle(3, 0xffffff, 0.9);
      g.strokeRoundedRect(
        at.x - half,
        at.y - half,
        NODE_SIZE,
        NODE_SIZE,
        radius,
      );
    } else {
      g.fillStyle(0x3a2a22, 0.42);
      g.fillRoundedRect(at.x - half, at.y - half, NODE_SIZE, NODE_SIZE, radius);
    }

    const label = state.enabled || state.completed ? `${number}` : '?';
    this.add
      .text(at.x, at.y, label, {
        fontFamily: UI_FONT,
        fontSize: '30px',
        fontStyle: '700',
        color: state.completed ? '#5a2f14' : state.enabled ? INK : '#c9b8a4',
      })
      .setOrigin(0.5);

    if (state.completed) {
      // ponytail: no per-level score exists — completion always shows 3 stars
      this.add
        .text(at.x, at.y - half - 16, '★★★', {
          fontFamily: UI_FONT,
          fontSize: '18px',
          color: STAR_GOLD,
        })
        .setOrigin(0.5)
        .setShadow(0, 2, 'rgba(36,29,22,0.5)', 3);
    }
    if (state.isCurrent) {
      this.add
        .text(at.x, at.y + half + 16, 'PLAY', {
          fontFamily: MONO_FONT,
          fontSize: '11px',
          color: '#ffe0a3',
        })
        .setOrigin(0.5)
        .setLetterSpacing(3);
    }

    if (state.enabled || state.completed) {
      this.add
        .rectangle(at.x, at.y, NODE_SIZE + 12, NODE_SIZE + 12, 0xffffff, 0.0001)
        .setInteractive({ useHandCursor: true })
        .on('pointerup', () =>
          this.scene.start('character-select', {
            levelId: level.id.toString(),
          }),
        );
    }
  }

  /** Top-right pill: current roller, unlock count, lootbox badge → collection. */
  private collectionPill(): void {
    const conn = db();
    const unlockedCount = [...conn.db.vw_my_unlocked_character.iter()].length;
    const total = Number(conn.db.vw_character.count());
    const unopened = [...conn.db.vw_my_lootbox.iter()].filter(
      (row) => !row.opened,
    ).length;

    const me = [...conn.db.vw_me.iter()][0];
    const selected = me?.selectedCharacterId
      ? [...conn.db.vw_character.iter()].find(
          (row) => row.id.toString() === me.selectedCharacterId?.toString(),
        )
      : undefined;
    const style = (selected ?? [...conn.db.vw_character.iter()][0])?.style;

    const x = VIEW_W - 110;
    const container = pill(this, x, 48, 150, 44);
    if (style) {
      container.add(addRoller(this, -48, 0, 34, style));
    }
    container.add(
      this.add
        .text(10, 0, `${unlockedCount}/${total}`, {
          fontFamily: UI_FONT,
          fontSize: '20px',
          fontStyle: '600',
          color: INK,
        })
        .setOrigin(0.5),
    );
    if (unopened > 0) {
      const badge = this.add.graphics();
      badge.fillStyle(0xe85d3c, 1);
      badge.fillCircle(62, -16, 11);
      container.add(badge);
      container.add(
        this.add
          .text(62, -16, `${unopened}`, {
            fontFamily: UI_FONT,
            fontSize: '13px',
            fontStyle: '700',
            color: '#f5ecd8',
          })
          .setOrigin(0.5),
      );
    }
    container.add(
      this.add
        .rectangle(0, 0, 150, 48, 0xffffff, 0.0001)
        .setInteractive({ useHandCursor: true })
        .on('pointerup', () => this.scene.start('collection')),
    );
  }
}
