import Phaser from 'phaser';
import { characterSpriteKey } from '../assets';
import { db } from '../db';
import {
  GAMEPLAY_Z,
  layerTextureKey,
  loadLevel,
  type DecodedLevel,
} from '../levels';
import {
  FACE_ASPECT,
  FACE_OFFSET_Y_RATIO,
  FACE_WIDTH_RATIO,
  faceTextureKey,
  type FaceName,
} from '../rollers';
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
import { ensureBackdropTextures, ensureParticleTextures } from '../textures';
import { DPR, UI_FONT, VIEW_H, VIEW_W, setupCamera } from '../ui';

/** Falling this far below the level ends the run in defeat. */
const FALL_MARGIN_PX = 90;
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
    // Live feel-tuning: gravity is re-read on every run start.
    this.matter.world.setGravity(0, TUNING.GRAVITY_Y);
    setupCamera(this);
    ensureParticleTextures(this);
    this.buildBackdrop();

    const conn = db();
    const character = [...conn.db.vw_character.iter()].find(
      (row) => row.id.toString() === this.characterId,
    );
    if (!character) {
      this.failToMenu(`Character '${this.characterId}' is not available.`);
      return;
    }
    this.stats = { ...character, id: character.id.toString() };
    try {
      this.level = loadLevel(conn, this.levelId);
    } catch (error) {
      this.failToMenu(error instanceof Error ? error.message : String(error));
      return;
    }
    // Layer scene SVGs become textures once per content hash; replays and
    // other levels sharing a hash skip straight to the build.
    const missing = this.level.layers.filter(
      (layer) => !this.textures.exists(layerTextureKey(layer)),
    );
    if (missing.length === 0) {
      this.buildWorld();
      return;
    }
    for (const layer of missing) {
      this.load.svg(layerTextureKey(layer), svgDataUrl(layer.svg));
    }
    this.load.once(Phaser.Loader.Events.COMPLETE, () => this.buildWorld());
    this.load.start();
  }

  private buildWorld(): void {
    if (
      this.level.layers.some(
        (layer) => !this.textures.exists(layerTextureKey(layer)),
      )
    ) {
      this.failToMenu('Level art failed to load.');
      return;
    }
    let built;
    try {
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
      style: this.stats.style,
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
    this.trackBackdrop();
    this.trackFace(time);
    if (!this.player || this.paused || this.outcome.settled) return;
    if (this.player.y > this.level.heightPx + FALL_MARGIN_PX) {
      this.outcome.defeat('You fell out of the level.');
      return;
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
      .image(spawn.x, spawn.y, faceTextureKey(this.expression))
      .setDisplaySize(width, width * FACE_ASPECT)
      .setDepth(GAMEPLAY_Z + 2);
  }

  private setFace(expression: FaceName): void {
    if (!this.face || this.expression === expression) return;
    this.expression = expression;
    this.face.setTexture(faceTextureKey(expression));
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
    ensureBackdropTextures(this);
    this.sky = this.add
      .image(0, 0, 'dusk-sky')
      .setDisplaySize(VIEW_W, VIEW_H)
      .setDepth(-10);
    this.hillFar = this.add
      .tileSprite(0, 0, VIEW_W, 150, 'hill-far')
      .setTileScale(1 / DPR)
      .setDepth(-9)
      .setAlpha(0.9);
    this.hillMid = this.add
      .tileSprite(0, 0, VIEW_W, 110, 'hill-mid')
      .setTileScale(1 / DPR)
      .setDepth(-8);
    this.trackBackdrop();
  }

  private trackBackdrop(): void {
    const view = this.cameras.main.worldView;
    this.sky?.setPosition(view.centerX, view.centerY);
    if (this.hillFar) {
      this.hillFar.setPosition(view.centerX, view.bottom - 70);
      this.hillFar.tilePositionX = view.x * 0.1 * DPR;
    }
    if (this.hillMid) {
      this.hillMid.setPosition(view.centerX, view.bottom - 40);
      this.hillMid.tilePositionX = view.x * 0.25 * DPR;
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
      .setDepth(GAMEPLAY_Z)
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
