import Phaser from 'phaser';
import { db } from '../db';
import { button, note, title } from '../ui';

export class ResultScene extends Phaser.Scene {
  private levelId!: string;
  private levelName!: string;

  constructor() {
    super('result');
  }

  init(data: { levelId: string; levelName: string }): void {
    this.levelId = data.levelId;
    this.levelName = data.levelName;
  }

  create(): void {
    const conn = db();
    title(this, `${this.levelName} complete!`);

    const unopened = [...conn.db.player_lootbox.iter()].filter((row) => !row.opened).length;
    note(
      this,
      160,
      unopened > 0
        ? `You have ${unopened} unopened lootbox${unopened === 1 ? '' : 'es'} waiting.`
        : 'Replays of completed levels grant no extra lootboxes.',
    );

    const centerX = this.scale.width / 2;
    button(this, centerX, 250, 'Open rewards', () => this.scene.start('collection'), { width: 360 });
    button(this, centerX, 330, 'Level select', () => this.scene.start('level-select'), { width: 360 });
    button(
      this,
      centerX,
      410,
      'Replay level',
      () => this.scene.start('character-select', { levelId: this.levelId }),
      { width: 360 },
    );
  }
}
