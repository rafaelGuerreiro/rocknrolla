import Phaser from 'phaser';
import { FIRE_RESISTANCE_THRESHOLD } from '../levels';

// Touch feel tuning.
const COYOTE_MS = 110;
const BUFFER_MS = 110;
const MAX_JUMPS = 2;
const LIFT_GRAVITY_FACTOR = 0.82;
const RELEASE_CUT = 0.45;
const WATER_DRAG = 0.96;
const HARD_LANDING_SPEED = 9;

export interface CharacterStats {
  id: string;
  name: string;
  style: string;
  density: number;
  jumpSpeed: number;
  flightTimeMs: number;
  buoyancy: number;
  fireResistance: number;
}

export interface ControllerEvents {
  /** Fired on every jump with the jump count in this airtime (2 = double). */
  onJump: (jumps: number) => void;
  onHardLanding: () => void;
}

/**
 * Per-run touch movement: jump buffering, coyote time, one double jump,
 * held lift capped by flight time, grounded tracking, and water buoyancy.
 * Owns its pointer and collision listeners; call `destroy()` on shutdown.
 */
export class PlayerController {
  private groundedUntil = 0;
  private wasGrounded = false;
  private jumpsUsed = 0;
  private holding = false;
  private buffedAt = -Infinity;
  private liftUntil = 0;
  private enabled = true;
  private inWater = false;

  constructor(
    private readonly scene: Phaser.Scene,
    private readonly player: Phaser.Physics.Matter.Image,
    private readonly stats: CharacterStats,
    private readonly waterRects: Phaser.Geom.Rectangle[],
    private readonly events: ControllerEvents,
  ) {
    scene.input.on(Phaser.Input.Events.POINTER_DOWN, this.onPointerDown);
    scene.input.on(Phaser.Input.Events.POINTER_UP, this.onPointerUp);
    scene.matter.world.on(
      Phaser.Physics.Matter.Events.COLLISION_ACTIVE,
      this.onCollisionActive,
    );
    scene.events.once(Phaser.Scenes.Events.SHUTDOWN, this.destroy);
  }

  /** Whether this character survives fire tiles. */
  survivesFire(): boolean {
    return this.stats.fireResistance >= FIRE_RESISTANCE_THRESHOLD;
  }

  /** Whether the player is currently inside a water region. */
  isInWater(): boolean {
    return this.inWater;
  }

  /** Stop reacting to input once the run has an outcome. */
  disable(): void {
    this.enabled = false;
    this.holding = false;
  }

  update(time: number): void {
    if (!this.enabled) return;

    const grounded = time <= this.groundedUntil;
    if (grounded && !this.wasGrounded) {
      this.jumpsUsed = 0;
      const impact = Math.abs(this.player.getVelocity().y ?? 0);
      if (impact > HARD_LANDING_SPEED) this.events.onHardLanding();
    }
    this.wasGrounded = grounded;

    // Buffered, coyote-friendly variable jump with one double jump.
    const buffered = time - this.buffedAt <= BUFFER_MS;
    if (buffered) {
      const firstJump =
        grounded || (this.jumpsUsed === 0 && time <= this.groundedUntil);
      const canDouble = !firstJump && this.jumpsUsed < MAX_JUMPS;
      if (firstJump || canDouble) {
        this.buffedAt = -Infinity;
        this.jumpsUsed = firstJump ? 1 : this.jumpsUsed + 1;
        this.groundedUntil = 0;
        this.liftUntil = time + this.stats.flightTimeMs;
        this.player.setVelocityY(-this.stats.jumpSpeed);
        this.events.onJump(this.jumpsUsed);
      }
    }

    const body = this.player.body as MatterJS.BodyType;
    const gravity = (
      this.scene.matter.world.engine as unknown as {
        gravity: { y: number; scale: number };
      }
    ).gravity;
    const gravityForce = body.mass * gravity.y * gravity.scale;

    // Held lift, capped by the character's flight time.
    if (this.holding && time < this.liftUntil && !grounded) {
      this.player.applyForce(
        new Phaser.Math.Vector2(0, -gravityForce * LIFT_GRAVITY_FACTOR),
      );
    }

    // Buoyancy while inside water, scaled by the character stat.
    const inWater = this.waterRects.some((rect) =>
      rect.contains(this.player.x, this.player.y),
    );
    this.inWater = inWater;
    if (inWater) {
      this.player.applyForce(
        new Phaser.Math.Vector2(0, -gravityForce * this.stats.buoyancy),
      );
      const velocity = this.player.getVelocity();
      this.player.setVelocity(
        (velocity.x ?? 0) * WATER_DRAG,
        (velocity.y ?? 0) * WATER_DRAG,
      );
    }
  }

  destroy = (): void => {
    this.enabled = false;
    this.scene.input.off(Phaser.Input.Events.POINTER_DOWN, this.onPointerDown);
    this.scene.input.off(Phaser.Input.Events.POINTER_UP, this.onPointerUp);
    this.scene.matter.world?.off(
      Phaser.Physics.Matter.Events.COLLISION_ACTIVE,
      this.onCollisionActive,
    );
  };

  private onPointerDown = (
    _pointer: Phaser.Input.Pointer,
    over: Phaser.GameObjects.GameObject[],
  ): void => {
    if (!this.enabled || over.length > 0) return;
    this.holding = true;
    this.buffedAt = this.scene.time.now;
  };

  private onPointerUp = (): void => {
    if (!this.holding) return;
    this.holding = false;
    const velocity = this.player.getVelocity();
    if ((velocity.y ?? 0) < 0) {
      this.player.setVelocityY((velocity.y ?? 0) * RELEASE_CUT);
    }
  };

  private onCollisionActive = (
    event: Phaser.Physics.Matter.Events.CollisionActiveEvent,
  ): void => {
    const playerBody = this.player.body as MatterJS.BodyType;
    for (const pair of event.pairs) {
      const other =
        pair.bodyA === playerBody
          ? pair.bodyB
          : pair.bodyB === playerBody
            ? pair.bodyA
            : null;
      if (!other || other.isSensor) continue;
      for (const support of pair.collision.supports) {
        if (support && support.y > this.player.y + 6) {
          this.groundedUntil = this.scene.time.now + COYOTE_MS;
        }
      }
    }
  };
}
