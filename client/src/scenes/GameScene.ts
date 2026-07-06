import Phaser from 'phaser';
import { characterSpriteKey } from '../assets';
import { db } from '../db';
import { GAMEPLAY_Z, loadLevel, type DecodedLevel } from '../levels';
import { CameraFollow } from '../gameplay/cameraFollow';
import { buildLevel } from '../gameplay/levelBuilder';
import { createPlayerBody } from '../gameplay/playerBody';
import {
  PlayerController,
  type CharacterStats,
} from '../gameplay/playerController';
import { RunOutcome } from '../gameplay/runOutcome';
import { ensureParticleTextures } from '../textures';
import { UI_FONT, button } from '../ui';

/** Falling this far below the level ends the run in defeat. */
const FALL_MARGIN_PX = 90;
/** Impact speed against a heavy block that triggers a camera shake. */
const HEAVY_SHAKE_SPEED = 6;

/**
 * Lifecycle coordinator for one run: loads the selected level and character,
 * composes the gameplay modules, connects collision events, and owns the
 * small HUD and particle effects.
 */
export class GameScene extends Phaser.Scene {
  private levelId!: string;
  private characterId!: string;
  private level!: DecodedLevel;
  private stats!: CharacterStats;
  private player?: Phaser.Physics.Matter.Image;
  private controller!: PlayerController;
  private outcome!: RunOutcome;
  private paused = false;

  constructor() {
    super('game');
  }

  init(data: { levelId: string; characterId: string }): void {
    this.levelId = data.levelId;
    this.characterId = data.characterId;
  }

  create(): void {
    this.paused = false;
    this.player = undefined;
    ensureParticleTextures(this);

    const conn = db();
    const character = [...conn.db.vw_character.iter()].find(
      (row) => row.id.toString() === this.characterId,
    );
    if (!character) {
      this.failToMenu(`Character '${this.characterId}' is not available.`);
      return;
    }
    this.stats = { ...character, id: character.id.toString() };
    let built;
    try {
      this.level = loadLevel(conn, this.levelId);
      built = buildLevel(this, this.level);
      this.player = createPlayerBody(
        this,
        characterSpriteKey(this.stats.style),
        built.spawn,
        {
          density: this.stats.density,
        },
      );
    } catch (error) {
      this.failToMenu(error instanceof Error ? error.message : String(error));
      return;
    }

    this.controller = new PlayerController(
      this,
      this.player,
      this.stats,
      built.waterRects,
      {
        onJump: () => this.jumpPuff(),
        onHardLanding: () => this.landingDust(),
      },
    );
    new CameraFollow(this, this.player);
    this.outcome = new RunOutcome(this, {
      levelId: this.levelId,
      levelName: this.level.name,
      onSettled: () => this.controller.disable(),
    });

    this.bindCollisions();
    this.buildHud();
    this.matter.world.setBounds(
      0,
      0,
      this.level.widthPx,
      this.level.heightPx,
      128,
      true,
      true,
      true,
      false,
    );
  }

  private failToMenu(message: string): void {
    this.add
      .text(
        this.scale.width / 2,
        this.scale.height / 2,
        `${message}\nTap to return.`,
        {
          fontFamily: UI_FONT,
          fontSize: '22px',
          color: '#e8ecf5',
          align: 'center',
        },
      )
      .setOrigin(0.5);
    this.input.once('pointerdown', () => this.scene.start('level-select'));
  }

  private bindCollisions(): void {
    const onCollisionStart = (
      event: Phaser.Physics.Matter.Events.CollisionStartEvent,
    ) => {
      const playerBody = this.player?.body as MatterJS.BodyType | undefined;
      if (!playerBody || this.outcome.settled) return;
      for (const pair of event.pairs) {
        const other =
          pair.bodyA === playerBody
            ? pair.bodyB
            : pair.bodyB === playerBody
              ? pair.bodyA
              : null;
        if (!other) continue;
        if (other.label === 'lethal')
          this.outcome.defeat('Wrecked by a hazard.');
        if (other.label === 'fire' && !this.controller.survivesFire()) {
          this.outcome.defeat('Burned up — not enough fire resistance.');
        }
        if (other.label === 'finish') this.outcome.finish();
        if (other.label === 'heavy' && this.player) {
          const velocity = this.player.getVelocity();
          if (
            Math.hypot(velocity.x ?? 0, velocity.y ?? 0) > HEAVY_SHAKE_SPEED
          ) {
            this.cameras.main.shake(120, 0.004);
          }
        }
      }
    };
    this.matter.world.on(
      Phaser.Physics.Matter.Events.COLLISION_START,
      onCollisionStart,
    );
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.matter.world?.off(
        Phaser.Physics.Matter.Events.COLLISION_START,
        onCollisionStart,
      );
    });
  }

  update(time: number): void {
    if (!this.player || this.paused || this.outcome.settled) return;
    if (this.player.y > this.level.heightPx + FALL_MARGIN_PX) {
      this.outcome.defeat('You fell out of the level.');
      return;
    }
    this.controller.update(time);
  }

  // -- HUD and game feel --------------------------------------------------------

  private buildHud(): void {
    this.add
      .text(24, 20, `${this.level.name} — ${this.stats.name}`, {
        fontFamily: UI_FONT,
        fontSize: '20px',
        color: '#e8ecf5',
      })
      .setScrollFactor(0)
      .setDepth(300);

    button(this, this.scale.width - 60, 44, '↻', () => this.scene.restart(), {
      width: 64,
      small: true,
    })
      .setScrollFactor(0)
      .setDepth(300);
    button(
      this,
      this.scale.width - 140,
      44,
      '❚❚',
      () => {
        if (this.outcome.settled) return;
        this.paused = !this.paused;
        if (this.paused) {
          this.matter.world.pause();
        } else {
          this.matter.world.resume();
        }
      },
      { width: 64, small: true },
    )
      .setScrollFactor(0)
      .setDepth(300);
  }

  private jumpPuff(): void {
    if (!this.player) return;
    const ring = this.add
      .image(this.player.x, this.player.y + 12, 'dust')
      .setDepth(GAMEPLAY_Z)
      .setAlpha(0.7)
      .setTint(0xc9d4ea);
    this.tweens.add({
      targets: ring,
      scaleX: 2.2,
      scaleY: 0.6,
      alpha: 0,
      duration: 220,
      onComplete: () => ring.destroy(),
    });
  }

  private landingDust(): void {
    if (!this.player) return;
    for (let i = 0; i < 6; i++) {
      const puff = this.add
        .image(
          this.player.x + Phaser.Math.Between(-14, 14),
          this.player.y + 12,
          'dust',
        )
        .setDepth(GAMEPLAY_Z)
        .setAlpha(0.6)
        .setTint(0xb9a58c)
        .setScale(Phaser.Math.FloatBetween(0.4, 0.9));
      this.tweens.add({
        targets: puff,
        x: puff.x + Phaser.Math.Between(-24, 24),
        y: puff.y - Phaser.Math.Between(4, 18),
        alpha: 0,
        duration: 320,
        onComplete: () => puff.destroy(),
      });
    }
  }
}
