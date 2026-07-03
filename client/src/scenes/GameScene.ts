import Phaser from 'phaser';
import { db } from '../db';
import {
  CELL,
  FIRE_RESISTANCE_THRESHOLD,
  GAMEPLAY_Z,
  HEAVY_DENSITY,
  TILE,
  loadLevel,
  type DecodedLayer,
  type DecodedLevel,
} from '../levels';
import { ensureBallTexture, ensureTextures, tileTextureKey } from '../textures';
import { UI_FONT, button } from '../ui';

// Touch feel tuning.
const COYOTE_MS = 110;
const BUFFER_MS = 110;
const MAX_JUMPS = 2;
const LIFT_GRAVITY_FACTOR = 0.82;
const RELEASE_CUT = 0.45;
const WATER_DRAG = 0.96;
const HARD_LANDING_SPEED = 9;

type CharacterStats = {
  id: string;
  name: string;
  style: string;
  density: number;
  jumpSpeed: number;
  flightTimeMs: number;
  buoyancy: number;
  fireResistance: number;
};

export class GameScene extends Phaser.Scene {
  private levelId!: string;
  private characterId!: string;
  private level!: DecodedLevel;
  private stats!: CharacterStats;
  private player!: Phaser.Physics.Matter.Image;
  private waterRects: Phaser.Geom.Rectangle[] = [];

  private groundedUntil = 0;
  private wasGrounded = false;
  private jumpsUsed = 0;
  private holding = false;
  private buffedAt = -Infinity;
  private liftUntil = 0;
  private dead = false;
  private finished = false;
  private paused = false;

  constructor() {
    super('game');
  }

  init(data: { levelId: string; characterId: string }): void {
    this.levelId = data.levelId;
    this.characterId = data.characterId;
  }

  create(): void {
    this.dead = false;
    this.finished = false;
    this.paused = false;
    this.holding = false;
    this.jumpsUsed = 0;
    this.buffedAt = -Infinity;
    this.groundedUntil = 0;
    this.waterRects = [];
    ensureTextures(this);

    const conn = db();
    const character = [...conn.db.character_def.iter()].find((row) => row.id === this.characterId);
    if (!character) {
      this.failToMenu(`Character '${this.characterId}' is not available.`);
      return;
    }
    this.stats = character;
    try {
      this.level = loadLevel(conn, this.levelId);
    } catch (error) {
      this.failToMenu(error instanceof Error ? error.message : String(error));
      return;
    }

    for (const layer of this.level.layers) {
      if (layer.z === GAMEPLAY_Z) this.buildGameplayLayer(layer);
      else this.buildVisualLayer(layer);
    }
    this.spawnPlayer();
    this.bindCollisions();
    this.bindInput();
    this.buildHud();

    this.cameras.main.setBounds(0, 0, this.level.widthPx, this.level.heightPx);
    this.cameras.main.startFollow(this.player, false, 0.08, 0.08);
    this.cameras.main.setFollowOffset(-110, 30);
    this.matter.world.setBounds(0, 0, this.level.widthPx, this.level.heightPx, 128, true, true, true, false);
  }

  private failToMenu(message: string): void {
    this.add
      .text(this.scale.width / 2, this.scale.height / 2, `${message}\nTap to return.`, {
        fontFamily: UI_FONT,
        fontSize: '22px',
        color: '#e8ecf5',
        align: 'center',
      })
      .setOrigin(0.5);
    this.input.once('pointerdown', () => this.scene.start('level-select'));
  }

  // -- level construction ---------------------------------------------------

  private buildVisualLayer(layer: DecodedLayer): void {
    const behind = layer.z < GAMEPLAY_Z;
    const tint = behind ? 0x39496e : 0x151b29;
    const alpha = behind ? 1 : 0.9;
    this.forEachTile(layer, (tile, x, y) => {
      const key = tileTextureKey(tile);
      if (!key) return;
      this.add
        .image(
          x * layer.cellWidth + layer.cellWidth / 2,
          y * layer.cellHeight + layer.cellHeight / 2,
          key,
        )
        .setScale(layer.cellWidth / CELL, layer.cellHeight / CELL)
        .setScrollFactor(layer.parallaxX, layer.parallaxY)
        .setDepth(layer.z)
        .setTint(tint)
        .setAlpha(alpha);
    });
  }

  private buildGameplayLayer(layer: DecodedLayer): void {
    // Draw every visible tile of the gameplay layer at depth 127.
    this.forEachTile(layer, (tile, x, y) => {
      if (tile === TILE.HEAVY) return; // rendered by its dynamic body
      const key = tileTextureKey(tile);
      if (!key) return;
      this.add
        .image(x * CELL + CELL / 2, y * CELL + CELL / 2, key)
        .setDepth(GAMEPLAY_Z);
    });

    // Merge horizontal runs of solid tiles into single static bodies.
    for (let y = 0; y < layer.height; y++) {
      let runStart = -1;
      for (let x = 0; x <= layer.width; x++) {
        const solid = x < layer.width && layer.tiles[y * layer.width + x] === TILE.SOLID;
        if (solid && runStart < 0) runStart = x;
        if (!solid && runStart >= 0) {
          const cells = x - runStart;
          this.matter.add.rectangle(
            (runStart + cells / 2) * CELL,
            y * CELL + CELL / 2,
            cells * CELL,
            CELL,
            { isStatic: true, label: 'terrain', friction: 0.9 },
          );
          runStart = -1;
        }
      }
    }

    this.forEachTile(layer, (tile, x, y) => {
      const originX = x * CELL;
      const originY = y * CELL;
      switch (tile) {
        case TILE.SLOPE_UP:
          this.addTriangle(originX, originY, [
            { x: 0, y: CELL },
            { x: CELL, y: CELL },
            { x: CELL, y: 0 },
          ]);
          break;
        case TILE.SLOPE_DOWN:
          this.addTriangle(originX, originY, [
            { x: 0, y: 0 },
            { x: CELL, y: CELL },
            { x: 0, y: CELL },
          ]);
          break;
        case TILE.LETHAL:
          this.addSensor(originX, originY, 'lethal', 6);
          break;
        case TILE.FIRE:
          this.addSensor(originX, originY, 'fire', 4);
          break;
        case TILE.FINISH:
          this.addSensor(originX, originY, 'finish', 0);
          break;
        case TILE.WATER:
          this.waterRects.push(new Phaser.Geom.Rectangle(originX, originY, CELL, CELL));
          break;
        case TILE.HEAVY: {
          const block = this.matter.add.image(originX + CELL / 2, originY + CELL / 2, 'tile-heavy');
          block.setBody(
            { type: 'rectangle', width: CELL, height: CELL },
            { label: 'heavy', density: HEAVY_DENSITY, friction: 0.8, frictionStatic: 1.2 },
          );
          block.setDepth(GAMEPLAY_Z);
          break;
        }
      }
    });
  }

  private forEachTile(
    layer: DecodedLayer,
    fn: (tile: number, x: number, y: number) => void,
  ): void {
    for (let y = 0; y < layer.height; y++) {
      for (let x = 0; x < layer.width; x++) {
        const tile = layer.tiles[y * layer.width + x];
        if (tile !== TILE.EMPTY) fn(tile, x, y);
      }
    }
  }

  private addTriangle(originX: number, originY: number, verts: { x: number; y: number }[]): void {
    const cx = verts.reduce((sum, v) => sum + v.x, 0) / verts.length;
    const cy = verts.reduce((sum, v) => sum + v.y, 0) / verts.length;
    this.matter.add.fromVertices(originX + cx, originY + cy, verts, {
      isStatic: true,
      label: 'terrain',
      friction: 0.9,
    });
  }

  private addSensor(originX: number, originY: number, label: string, inset: number): void {
    this.matter.add.rectangle(
      originX + CELL / 2,
      originY + CELL / 2,
      CELL - inset * 2,
      CELL - inset * 2,
      { isStatic: true, isSensor: true, label },
    );
  }

  private spawnPlayer(): void {
    const layer = this.level.gameplay;
    let spawnX = CELL * 2;
    let spawnY = CELL * 2;
    this.forEachTile(layer, (tile, x, y) => {
      if (tile === TILE.SPAWN) {
        spawnX = x * CELL + CELL / 2;
        spawnY = y * CELL + CELL / 2;
      }
    });
    const key = ensureBallTexture(this, this.stats.id, this.stats.style);
    this.player = this.matter.add.image(spawnX, spawnY, key, undefined, {
      shape: 'circle',
      label: 'player',
      density: this.stats.density,
      friction: 0.9,
      frictionAir: 0.012,
      restitution: 0.08,
    });
    this.player.setDepth(GAMEPLAY_Z + 1);
  }

  // -- collisions and input --------------------------------------------------

  private bindCollisions(): void {
    const involvesPlayer = (pair: Phaser.Types.Physics.Matter.MatterCollisionPair) => {
      const playerBody = this.player.body as MatterJS.BodyType;
      if (pair.bodyA === playerBody) return pair.bodyB;
      if (pair.bodyB === playerBody) return pair.bodyA;
      return null;
    };

    this.matter.world.on(
      Phaser.Physics.Matter.Events.COLLISION_START,
      (event: Phaser.Physics.Matter.Events.CollisionStartEvent) => {
        for (const pair of event.pairs) {
          const other = involvesPlayer(pair);
          if (!other) continue;
          if (other.label === 'lethal') this.die();
          if (other.label === 'fire' && this.stats.fireResistance < FIRE_RESISTANCE_THRESHOLD) {
            this.die();
          }
          if (other.label === 'finish') this.reachFinish();
          if (other.label === 'heavy') {
            const impact = Math.hypot(
              this.player.getVelocity().x ?? 0,
              this.player.getVelocity().y ?? 0,
            );
            if (impact > 6) this.cameras.main.shake(120, 0.004);
          }
        }
      },
    );

    this.matter.world.on(
      Phaser.Physics.Matter.Events.COLLISION_ACTIVE,
      (event: Phaser.Physics.Matter.Events.CollisionActiveEvent) => {
        for (const pair of event.pairs) {
          const other = involvesPlayer(pair);
          if (!other || other.isSensor) continue;
          for (const support of pair.collision.supports) {
            if (support && support.y > this.player.y + 6) {
              this.groundedUntil = this.time.now + COYOTE_MS;
            }
          }
        }
      },
    );
  }

  private bindInput(): void {
    this.input.on(
      'pointerdown',
      (_pointer: Phaser.Input.Pointer, over: Phaser.GameObjects.GameObject[]) => {
        if (over.length > 0 || this.paused) return;
        if (this.dead) {
          this.scene.restart();
          return;
        }
        if (this.finished) return;
        this.holding = true;
        this.buffedAt = this.time.now;
      },
    );
    this.input.on('pointerup', () => {
      if (!this.holding) return;
      this.holding = false;
      const velocity = this.player?.getVelocity();
      if (velocity && (velocity.y ?? 0) < 0) {
        this.player.setVelocityY((velocity.y ?? 0) * RELEASE_CUT);
      }
    });
  }

  // -- per-frame simulation ---------------------------------------------------

  update(time: number): void {
    if (!this.player || this.dead || this.paused) return;

    if (this.player.y > this.level.heightPx + 90 && !this.finished) {
      this.die();
      return;
    }

    const grounded = time <= this.groundedUntil;
    if (grounded && !this.wasGrounded) {
      this.jumpsUsed = 0;
      const impact = Math.abs(this.player.getVelocity().y ?? 0);
      if (impact > HARD_LANDING_SPEED) this.landingDust();
    }
    this.wasGrounded = grounded;

    // Buffered, coyote-friendly variable jump with one double jump.
    const buffered = time - this.buffedAt <= BUFFER_MS;
    if (buffered && !this.finished) {
      const firstJump = grounded || (this.jumpsUsed === 0 && time <= this.groundedUntil);
      const canDouble = !firstJump && this.jumpsUsed < MAX_JUMPS;
      if (firstJump || canDouble) {
        this.buffedAt = -Infinity;
        this.jumpsUsed = firstJump ? 1 : this.jumpsUsed + 1;
        this.groundedUntil = 0;
        this.liftUntil = time + this.stats.flightTimeMs;
        this.player.setVelocityY(-this.stats.jumpSpeed);
        this.jumpPuff();
      }
    }

    const body = this.player.body as MatterJS.BodyType;
    const gravity = (
      this.matter.world.engine as unknown as { gravity: { y: number; scale: number } }
    ).gravity;
    const gravityForce = body.mass * gravity.y * gravity.scale;

    // Held lift, capped by the character's flight time.
    if (this.holding && time < this.liftUntil && !grounded) {
      this.player.applyForce(new Phaser.Math.Vector2(0, -gravityForce * LIFT_GRAVITY_FACTOR));
    }

    // Buoyancy while inside water, scaled by the character stat.
    const inWater = this.waterRects.some((rect) => rect.contains(this.player.x, this.player.y));
    if (inWater) {
      this.player.applyForce(new Phaser.Math.Vector2(0, -gravityForce * this.stats.buoyancy));
      const velocity = this.player.getVelocity();
      this.player.setVelocity((velocity.x ?? 0) * WATER_DRAG, (velocity.y ?? 0) * WATER_DRAG);
    }
  }

  // -- outcomes ----------------------------------------------------------------

  private die(): void {
    if (this.dead || this.finished) return;
    this.dead = true;
    this.holding = false;
    this.cameras.main.shake(180, 0.008);
    this.matter.world.pause();
    this.add
      .rectangle(0, 0, this.scale.width, this.scale.height, 0x2b0a0a, 0.55)
      .setOrigin(0)
      .setScrollFactor(0)
      .setDepth(300);
    this.add
      .text(this.scale.width / 2, this.scale.height / 2, 'Wrecked!\nTap to retry', {
        fontFamily: UI_FONT,
        fontSize: '34px',
        color: '#ffd9d9',
        align: 'center',
      })
      .setOrigin(0.5)
      .setScrollFactor(0)
      .setDepth(301);
  }

  /** Report completion exactly once and wait for the server to record it. */
  private reachFinish(): void {
    if (this.finished || this.dead) return;
    this.finished = true;
    this.holding = false;
    const conn = db();

    const goToResult = () =>
      this.scene.start('result', { levelId: this.levelId, levelName: this.level.name });

    const alreadyCompleted = [...conn.db.player_completed_level.iter()].some(
      (row) => row.levelId === this.levelId,
    );
    if (alreadyCompleted) {
      goToResult(); // replays grant nothing; no need to re-report
      return;
    }

    const saving = this.add
      .text(this.scale.width / 2, this.scale.height / 2, 'Finish! Saving…', {
        fontFamily: UI_FONT,
        fontSize: '32px',
        color: '#f5c451',
      })
      .setOrigin(0.5)
      .setScrollFactor(0)
      .setDepth(301);

    const timeout = this.time.delayedCall(6000, () => failSaving('The server did not confirm the run.'));
    const failSaving = (message: string) => {
      conn.db.player_completed_level.removeOnInsert(onInsert);
      timeout.remove();
      saving.setText(message);
      button(
        this,
        this.scale.width / 2,
        this.scale.height / 2 + 70,
        'Retry saving',
        () => {
          saving.setText('Finish! Saving…');
          conn.db.player_completed_level.onInsert(onInsert);
          report();
        },
        { width: 280, small: true },
      ).setScrollFactor(0).setDepth(301);
    };
    const onInsert = (_ctx: unknown, row: { levelId: string }) => {
      if (row.levelId !== this.levelId) return;
      conn.db.player_completed_level.removeOnInsert(onInsert);
      timeout.remove();
      goToResult();
    };
    const report = () =>
      conn.reducers
        .completeLevel({ levelId: this.levelId })
        .catch((error) => failSaving(`Saving failed: ${error instanceof Error ? error.message : error}`));
    conn.db.player_completed_level.onInsert(onInsert);
    report();
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
        this.paused = !this.paused;
        if (this.paused) {
          this.matter.world.pause();
        } else if (!this.dead) {
          this.matter.world.resume();
        }
      },
      { width: 64, small: true },
    )
      .setScrollFactor(0)
      .setDepth(300);
  }

  private jumpPuff(): void {
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
    for (let i = 0; i < 6; i++) {
      const puff = this.add
        .image(this.player.x + Phaser.Math.Between(-14, 14), this.player.y + 12, 'dust')
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
