import Phaser from 'phaser';
import {
  BACKDROP_RASTER_SCALE,
  backdropById,
  characterBodyKey,
  faceKey,
  type FaceName,
} from '../content';
import { db } from '../db';
import type { DbConnection } from '../module_bindings';
import {
  DEPTH,
  awaitLevelPlacements,
  collisionRoot,
  loadLevel,
  type DecodedLevel,
} from '../levels';
import { FACE_ASPECT, FACE_OFFSET_Y_RATIO, FACE_WIDTH_RATIO } from '../rollers';
import { svgDataUrl } from '../tiles';
import { TUNING } from '../tuning';
import { CameraFollow } from '../gameplay/cameraFollow';
import { buildLevel } from '../gameplay/levelBuilder';
import { createPlayerBody } from '../gameplay/playerBody';
import {
  PlayerController,
  type CharacterStats,
} from '../gameplay/playerController';
import { RunOutcome } from '../gameplay/runOutcome';
import { ensureParticleTextures } from '../textures';
import { UI_FONT, VIEW_H, VIEW_W, note, setupCamera } from '../ui';

/** Falling this far below the level ends the run in defeat. */
const FALL_MARGIN_PX = 90;
/** Below this horizontal speed the player counts as stopped (float jitter). */
const STALL_SPEED = 0.1;
/** Being stopped this long straight ends the run in defeat. */
const STALL_TIMEOUT_MS = 2000;
/** Impact speed against a heavy block that triggers a camera shake. */
const HEAVY_SHAKE_SPEED = 6;
/** On-screen character size the face proportions are computed against. */
const PLAYER_FACE_BODY_PX = 64;

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
  private sky?: Phaser.GameObjects.Image;
  private face?: Phaser.GameObjects.Image;
  private expression: FaceName = 'determined';
  private surprisedUntil = 0;
  private hillFar?: Phaser.GameObjects.TileSprite;
  private hillMid?: Phaser.GameObjects.TileSprite;
  private stallMs = 0;

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
    this.stallMs = 0;
    // Live feel-tuning: gravity is re-read on every run start.
    this.matter.world.setGravity(0, TUNING.GRAVITY_Y);
    setupCamera(this);
    ensureParticleTextures(this);

    const conn = db();
    const character = [...conn.db.vw_character_v1.iter()].find(
      (row) => row.id.toString() === this.characterId,
    );
    if (!character) {
      this.failToMenu(`Character '${this.characterId}' is not available.`);
      return;
    }
    this.stats = { ...character, id: character.id.toString() };
    // The placement view is gated by the server-side level selection, so the
    // first play of a level races selectLevelV1 against the subscription
    // update — wait for the rows instead of failing the run.
    const loading = note(this, VIEW_H / 2, 'Rolling the hill in…');
    awaitLevelPlacements(conn.db.vw_level_placement_v1, this.levelId)
      .then(() => {
        loading.destroy();
        this.loadArtAndBuild(conn);
      })
      .catch((error: Error) => this.failToMenu(error.message));
  }

  private loadArtAndBuild(conn: DbConnection): void {
    try {
      this.level = loadLevel(conn, this.levelId);
      this.buildBackdrop();
    } catch (error) {
      this.failToMenu(error instanceof Error ? error.message : String(error));
      return;
    }
    // Component SVGs become textures once per content hash and are shared
    // across every level that places them.
    const missing = this.level.textures.filter(
      (t) => !this.textures.exists(t.key),
    );
    if (missing.length === 0) {
      this.buildWorld();
      return;
    }
    for (const texture of missing) {
      this.load.svg(texture.key, svgDataUrl(texture.svg));
    }
    this.load.once(Phaser.Loader.Events.COMPLETE, () => this.buildWorld());
    this.load.start();
  }

  private buildWorld(): void {
    if (this.level.textures.some((t) => !this.textures.exists(t.key))) {
      this.failToMenu('Level art failed to load.');
      return;
    }
    let built;
    try {
      built = buildLevel(this, this.level);
      this.player = createPlayerBody(
        this,
        characterBodyKey(this.characterId),
        built.spawn,
        {
          density: this.stats.density,
        },
      );
    } catch (error) {
      this.failToMenu(error instanceof Error ? error.message : String(error));
      return;
    }
    this.buildFace(built.spawn);

    this.controller = new PlayerController(
      this,
      this.player,
      this.stats,
      built.waterRects,
      {
        onJump: (jumps) => {
          this.jumpPuff();
          if (jumps >= 2) {
            this.comboFlash('×2 NICE!');
            this.surprisedUntil = this.time.now + 700;
          }
        },
        onHardLanding: () => this.landingDust(),
      },
    );
    new CameraFollow(this, this.player);
    this.outcome = new RunOutcome(this, {
      levelId: this.levelId,
      levelName: this.level.name,
      onSettled: (outcome) => {
        this.controller.disable();
        this.setFace(outcome === 'success' ? 'happy' : 'dizzy');
      },
    });

    this.bindCollisions();
    this.scene.launch('hud', {
      levelName: this.level.name,
      rollerName: this.stats.name,
      characterId: this.characterId,
    });
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () =>
      this.scene.stop('hud'),
    );
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
      .text(VIEW_W / 2, VIEW_H / 2, `${message}\nTap to return.`, {
        fontFamily: UI_FONT,
        fontSize: '22px',
        color: '#e8ecf5',
        align: 'center',
      })
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
        const rootA = collisionRoot(pair.bodyA);
        const rootB = collisionRoot(pair.bodyB);
        const other =
          rootA === playerBody ? rootB : rootB === playerBody ? rootA : null;
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

  update(time: number, delta: number): void {
    this.trackBackdrop();
    this.trackFace(time);
    if (!this.player || this.paused || this.outcome.settled) return;
    if (this.player.y > this.level.heightPx + FALL_MARGIN_PX) {
      this.outcome.defeat('You fell out of the level.');
      return;
    }
    // Gravity on slopes is the only propulsion, so a run with no horizontal
    // speed can never recover — end it instead of leaving the player stuck.
    // Accumulating delta here (after the paused guard) keeps pauses from
    // counting toward the timeout.
    if (Math.abs(this.player.getVelocity().x ?? 0) < STALL_SPEED) {
      this.stallMs += delta;
      if (this.stallMs >= STALL_TIMEOUT_MS) {
        this.outcome.defeat('Stuck — nowhere left to roll.');
        return;
      }
    } else {
      this.stallMs = 0;
    }
    this.controller.update(time);
  }

  // -- Two-layer roller rig -------------------------------------------------

  /**
   * The face is a sibling of the Matter body: it tracks the body's position
   * but never its rotation, so the expression stays upright while the
   * irregular body tumbles.
   */
  private buildFace(spawn: Phaser.Math.Vector2): void {
    const width = PLAYER_FACE_BODY_PX * FACE_WIDTH_RATIO;
    this.expression = 'determined';
    this.face = this.add
      .image(spawn.x, spawn.y, faceKey(this.expression))
      .setDisplaySize(width, width * FACE_ASPECT)
      .setDepth(DEPTH.FACE);
  }

  private setFace(expression: FaceName): void {
    if (!this.face || this.expression === expression) return;
    this.expression = expression;
    this.face.setTexture(faceKey(expression));
    const width = PLAYER_FACE_BODY_PX * FACE_WIDTH_RATIO;
    this.face.setDisplaySize(width, width * FACE_ASPECT);
  }

  private trackFace(time: number): void {
    if (!this.face || !this.player) return;
    this.face.setPosition(
      this.player.x,
      this.player.y + PLAYER_FACE_BODY_PX * FACE_OFFSET_Y_RATIO,
    );
    if (this.outcome?.settled) return; // happy/dizzy set at settle time
    if (time < this.surprisedUntil) {
      this.setFace('surprised');
    } else if (this.controller?.isInWater()) {
      this.setFace('nervous');
    } else {
      this.setFace('determined');
    }
  }

  // -- HUD and game feel --------------------------------------------------------

  /**
   * Backdrop objects follow the camera's world view instead of using
   * scrollFactor 0, which misplaces objects under the DPR camera zoom.
   */
  private buildBackdrop(): void {
    const backdrop = backdropById(this.level.backdropId);
    // Below the deepest background plane (placement z bottoms out at -128).
    this.sky = this.add
      .image(0, 0, backdrop.sky.key)
      .setDisplaySize(VIEW_W, VIEW_H)
      .setDepth(-200);
    // Strips raster at BACKDROP_RASTER_SCALE× their natural size; the tile
    // scale maps them back to logical pixels.
    this.hillFar = this.add
      .tileSprite(0, 0, VIEW_W, backdrop.far.height, backdrop.far.key)
      .setTileScale(1 / BACKDROP_RASTER_SCALE)
      .setDepth(-190)
      .setAlpha(0.9);
    this.hillMid = this.add
      .tileSprite(0, 0, VIEW_W, backdrop.mid.height, backdrop.mid.key)
      .setTileScale(1 / BACKDROP_RASTER_SCALE)
      .setDepth(-180);
    this.trackBackdrop();
  }

  private trackBackdrop(): void {
    const view = this.cameras.main.worldView;
    this.sky?.setPosition(view.centerX, view.centerY);
    if (this.hillFar) {
      this.hillFar.setPosition(view.centerX, view.bottom - 70);
      this.hillFar.tilePositionX =
        view.x * TUNING.BACKDROP_FAR_PARALLAX * BACKDROP_RASTER_SCALE;
    }
    if (this.hillMid) {
      this.hillMid.setPosition(view.centerX, view.bottom - 40);
      this.hillMid.tilePositionX =
        view.x * TUNING.BACKDROP_MID_PARALLAX * BACKDROP_RASTER_SCALE;
    }
  }

  /** Toggled by the HUD overlay scene. */
  togglePause(): void {
    if (this.outcome.settled) return;
    this.paused = !this.paused;
    if (this.paused) {
      this.matter.world.pause();
    } else {
      this.matter.world.resume();
    }
  }

  /** Pop-rise-fade celebration text above the player (design "combo flash"). */
  private comboFlash(message: string): void {
    if (!this.player) return;
    const flash = this.add
      .text(this.player.x, this.player.y - 64, message, {
        fontFamily: UI_FONT,
        fontSize: '26px',
        fontStyle: '700',
        color: '#ffe08a',
      })
      .setOrigin(0.5)
      .setDepth(200)
      .setScale(0.6)
      .setShadow(0, 3, 'rgba(36,29,22,0.6)', 4);
    this.tweens.add({
      targets: flash,
      scale: 1,
      y: flash.y - 20,
      duration: 600,
      ease: 'Back.easeOut',
    });
    this.tweens.add({
      targets: flash,
      alpha: 0,
      delay: 380,
      duration: 220,
      onComplete: () => flash.destroy(),
    });
  }

  private jumpPuff(): void {
    if (!this.player) return;
    const ring = this.add
      .image(this.player.x, this.player.y + 12, 'dust')
      .setDepth(DEPTH.EFFECTS)
      .setAlpha(0.7)
      .setTint(0xffe0a3);
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
        .setDepth(DEPTH.EFFECTS)
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
