import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
import { backdropBySlug, DEFAULT_BACKDROP_SLUG } from '../content';
import { addRoller } from '../rollers';
import { db } from '../db';
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

// Trail layout: level 1 at the bottom, later hills zigzag upward.
const SPACING_Y = 130;
const ZIGZAG_X = 160;
const TOP_MARGIN = 150;
const BOTTOM_MARGIN = 120;
/** Pointer travel past this cancels the tap — it was a scroll drag. */
const DRAG_CANCEL_PX = 10;

export class LevelSelectScene extends Phaser.Scene {
  constructor() {
    super('level-select');
  }

  create(): void {
    const width = VIEW_W;
    const height = VIEW_H;
    setupCamera(this);
    const backdrop = backdropBySlug(DEFAULT_BACKDROP_SLUG);
    this.add
      .image(width / 2, height / 2, backdrop.sky.key)
      .setDisplaySize(width, height);
    this.add
      .image(width / 2, height - 50, backdrop.far.key)
      .setDisplaySize(width, backdrop.far.height)
      .setAlpha(0.85);
    this.add
      .image(width / 2, height - 22, backdrop.mid.key)
      .setDisplaySize(width, backdrop.mid.height);

    const conn = db();
    // Chrome floats above the scrolling trail.
    pill(this, 190, 48, 260, 44, 'Choose your hill').setDepth(1);
    this.collectionPill();

    const enabledIds = new Set(
      [...conn.db.vw_my_enabled_level_v1.iter()].map((row) =>
        row.levelId.toString(),
      ),
    );
    const completedAt = new Map(
      [...conn.db.vw_my_completed_level_v1.iter()].map((row) => [
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
    const levels = [...conn.db.vw_level_v1.iter()].sort((a, b) => {
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

    // The trail climbs: level 1 sits at the bottom and later hills zigzag
    // upward, scrolling in a container while the chrome stays put.
    const worldHeight = Math.max(
      height,
      TOP_MARGIN + (levels.length - 1) * SPACING_Y + BOTTOM_MARGIN,
    );
    const points = levels.map(
      (_, i) =>
        new Phaser.Math.Vector2(
          width / 2 + (i % 2 === 0 ? -ZIGZAG_X : ZIGZAG_X),
          worldHeight - BOTTOM_MARGIN - i * SPACING_Y,
        ),
    );
    const trail = this.add.container(0, height - worldHeight);
    this.drawTrailDots(trail, points);

    let currentIndex = -1;
    levels.forEach((level, index) => {
      const id = level.id.toString();
      const completed = completedIds.has(id);
      const enabled = enabledIds.has(id);
      const isCurrent = enabled && !completed && currentIndex === -1;
      if (isCurrent) currentIndex = index;
      this.levelNode(trail, points[index], index + 1, level, {
        completed,
        enabled,
        isCurrent,
      });
    });
    this.enableScroll(
      trail,
      worldHeight,
      currentIndex >= 0 ? points[currentIndex] : undefined,
    );

    this.add
      .text(width / 2, height - 26, 'TAP A HILL TO ROLL IN ▸', {
        fontFamily: MONO_FONT,
        fontSize: '12px',
        color: '#ffe0a3',
      })
      .setOrigin(0.5)
      .setLetterSpacing(3)
      .setDepth(1);
  }

  /** Drag (touch) or wheel to scroll; starts focused on the current hill. */
  private enableScroll(
    trail: Phaser.GameObjects.Container,
    worldHeight: number,
    focus?: Phaser.Math.Vector2,
  ): void {
    const clampY = (y: number) => Phaser.Math.Clamp(y, VIEW_H - worldHeight, 0);
    if (focus) trail.y = clampY(VIEW_H / 2 - focus.y);
    this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
      if (!pointer.isDown) return;
      // Pointer deltas are in canvas pixels; the camera zooms by DPR.
      trail.y = clampY(
        trail.y +
          (pointer.position.y - pointer.prevPosition.y) /
            this.cameras.main.zoom,
      );
    });
    this.input.on(
      'wheel',
      (
        _pointer: Phaser.Input.Pointer,
        _over: unknown,
        _dx: number,
        dy: number,
      ) => {
        trail.y = clampY(trail.y - dy);
      },
    );
  }

  private drawTrailDots(
    trail: Phaser.GameObjects.Container,
    points: Phaser.Math.Vector2[],
  ): void {
    const g = this.add.graphics();
    trail.add(g);
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
    trail: Phaser.GameObjects.Container,
    at: Phaser.Math.Vector2,
    number: number,
    level: { id: { toString(): string }; name: string },
    state: { completed: boolean; enabled: boolean; isCurrent: boolean },
  ): void {
    const g = this.add.graphics();
    trail.add(g);
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
    trail.add(
      this.add
        .text(at.x, at.y, label, {
          fontFamily: UI_FONT,
          fontSize: '30px',
          fontStyle: '700',
          color: state.completed ? '#5a2f14' : state.enabled ? INK : '#c9b8a4',
        })
        .setOrigin(0.5),
    );

    if (state.completed) {
      // ponytail: no per-level score exists — completion always shows 3 stars
      trail.add(
        this.add
          .text(at.x, at.y - half - 16, '★★★', {
            fontFamily: UI_FONT,
            fontSize: '18px',
            color: STAR_GOLD,
          })
          .setOrigin(0.5)
          .setShadow(0, 2, 'rgba(36,29,22,0.5)', 3),
      );
    }
    if (state.isCurrent) {
      trail.add(
        this.add
          .text(at.x, at.y + half + 16, 'PLAY', {
            fontFamily: MONO_FONT,
            fontSize: '11px',
            color: '#ffe0a3',
          })
          .setOrigin(0.5)
          .setLetterSpacing(3),
      );
    }

    if (state.enabled || state.completed) {
      const levelId = level.id.toString();
      trail.add(
        this.add
          .rectangle(
            at.x,
            at.y,
            NODE_SIZE + 12,
            NODE_SIZE + 12,
            0xffffff,
            0.0001,
          )
          .setInteractive({ useHandCursor: true })
          .on('pointerup', (pointer: Phaser.Input.Pointer) => {
            if (pointer.getDistance() > DRAG_CANCEL_PX) return;
            db()
              .reducers.selectLevelV1({ levelId: Uuid.parse(levelId) })
              .catch((error) => console.error('selectLevelV1 failed:', error));
            this.scene.start('character-select', { levelId });
          }),
      );
    }
  }

  /** Top-right pill: current roller, unlock count, lootbox badge → collection. */
  private collectionPill(): void {
    const conn = db();
    const unlockedCount = [...conn.db.vw_my_unlocked_character_v1.iter()]
      .length;
    const total = Number(conn.db.vw_character_v1.count());
    const unopened = [...conn.db.vw_my_lootbox_v1.iter()].filter(
      (row) => !row.opened,
    ).length;

    const me = [...conn.db.vw_me_v1.iter()][0];
    const selected = me?.selectedCharacterId
      ? [...conn.db.vw_character_v1.iter()].find(
          (row) => row.id.toString() === me.selectedCharacterId?.toString(),
        )
      : undefined;
    const character = selected ?? [...conn.db.vw_character_v1.iter()][0];

    const x = VIEW_W - 110;
    const container = pill(this, x, 48, 150, 44).setDepth(1);
    if (character) {
      container.add(addRoller(this, -48, 0, 34, character.id.toString()));
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
        .on('pointerup', (pointer: Phaser.Input.Pointer) => {
          if (pointer.getDistance() > DRAG_CANCEL_PX) return;
          this.scene.start('collection');
        }),
    );
  }
}
