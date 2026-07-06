import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
import { db } from '../db';
import { UI_FONT, button } from '../ui';

/** How long to wait for the server to confirm a completion report. */
const CONFIRM_TIMEOUT_MS = 6000;

export interface RunOutcomeConfig {
  levelId: string;
  levelName: string;
  /** Invoked exactly once when the run settles, before any scene change. */
  onSettled: () => void;
}

type CompletedRow = { levelId: Uuid };
type RowCallback = (ctx: unknown, row: CompletedRow) => void;

/**
 * Mutually exclusive success/defeat outcome for one run. Success requires a
 * server-confirmed completion; every failure (hazard, fall, reducer
 * rejection, confirmation timeout) is a defeat that returns to level
 * selection. Listeners and timers are cleaned up on either outcome and on
 * scene shutdown, so late callbacks cannot transition an abandoned scene.
 */
export class RunOutcome {
  private state: 'playing' | 'settled' = 'playing';
  private confirmListener?: RowCallback;
  private confirmTimeout?: Phaser.Time.TimerEvent;

  constructor(
    private readonly scene: Phaser.Scene,
    private readonly config: RunOutcomeConfig,
  ) {
    scene.events.once(Phaser.Scenes.Events.SHUTDOWN, this.cleanup);
  }

  get settled(): boolean {
    return this.state === 'settled';
  }

  /** Report completion once and move to the result only when confirmed. */
  finish(): void {
    if (this.state !== 'playing') return;
    this.state = 'settled';
    this.config.onSettled();
    const conn = db();

    const alreadyCompleted = [...conn.db.vw_my_completed_level.iter()].some(
      (row) => row.levelId.toString() === this.config.levelId,
    );
    if (alreadyCompleted) {
      this.succeed(); // replays grant nothing; no need to re-report
      return;
    }

    const saving = this.scene.add
      .text(
        this.scene.scale.width / 2,
        this.scene.scale.height / 2,
        'Finish! Saving…',
        {
          fontFamily: UI_FONT,
          fontSize: '32px',
          color: '#f5c451',
        },
      )
      .setOrigin(0.5)
      .setScrollFactor(0)
      .setDepth(301);

    this.confirmListener = (_ctx, row) => {
      if (row.levelId.toString() !== this.config.levelId) return;
      this.cleanup();
      this.succeed();
    };
    this.confirmTimeout = this.scene.time.delayedCall(
      CONFIRM_TIMEOUT_MS,
      () => {
        this.cleanup();
        saving.destroy();
        this.showDefeat('The server did not confirm the run.');
      },
    );
    conn.db.vw_my_completed_level.onInsert(this.confirmListener);
    conn.reducers
      .completeLevel({ levelId: Uuid.parse(this.config.levelId) })
      .catch((error) => {
        if (!this.scene.scene.isActive()) return;
        this.cleanup();
        saving.destroy();
        this.showDefeat(
          `Saving failed: ${error instanceof Error ? error.message : String(error)}`,
        );
      });
  }

  /** End the run as a defeat and offer only the way back to level select. */
  defeat(reason: string): void {
    if (this.state !== 'playing') return;
    this.state = 'settled';
    this.config.onSettled();
    this.cleanup();
    this.scene.cameras.main.shake(180, 0.008);
    this.showDefeat(reason);
  }

  private succeed(): void {
    this.scene.scene.start('result', {
      levelId: this.config.levelId,
      levelName: this.config.levelName,
    });
  }

  private showDefeat(reason: string): void {
    this.scene.matter.world.pause();
    const { width, height } = this.scene.scale;
    this.scene.add
      .rectangle(0, 0, width, height, 0x2b0a0a, 0.6)
      .setOrigin(0)
      .setScrollFactor(0)
      .setDepth(300);
    this.scene.add
      .text(width / 2, height / 2 - 60, 'Defeat', {
        fontFamily: UI_FONT,
        fontSize: '42px',
        color: '#ffd9d9',
      })
      .setOrigin(0.5)
      .setScrollFactor(0)
      .setDepth(301);
    this.scene.add
      .text(width / 2, height / 2 - 10, reason, {
        fontFamily: UI_FONT,
        fontSize: '19px',
        color: '#e8b7b7',
        align: 'center',
        wordWrap: { width: width - 160 },
      })
      .setOrigin(0.5)
      .setScrollFactor(0)
      .setDepth(301);
    button(
      this.scene,
      width / 2,
      height / 2 + 70,
      'Level select',
      () => this.scene.scene.start('level-select'),
      { width: 300 },
    )
      .setScrollFactor(0)
      .setDepth(301);
  }

  private cleanup = (): void => {
    if (this.confirmListener) {
      db().db.vw_my_completed_level.removeOnInsert(this.confirmListener);
      this.confirmListener = undefined;
    }
    this.confirmTimeout?.remove();
    this.confirmTimeout = undefined;
  };
}
