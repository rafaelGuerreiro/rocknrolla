import Phaser from 'phaser';
import { db } from '../db';
import { button, note, title } from '../ui';

export class LevelSelectScene extends Phaser.Scene {
  constructor() {
    super('level-select');
  }

  create(): void {
    title(this, 'RocknRolla');
    const conn = db();

    const enabledIds = new Set([...conn.db.player_enabled_level.iter()].map((row) => row.levelId));
    const completedIds = new Set([...conn.db.player_completed_level.iter()].map((row) => row.levelId));
    const levels = [...conn.db.level.iter()]
      .filter((level) => level.active && enabledIds.has(level.id))
      .sort((a, b) => a.id.localeCompare(b.id));

    if (levels.length === 0) {
      note(this, this.scale.height / 2, 'No levels enabled yet. Reconnect after importing content.');
    }
    levels.forEach((level, index) => {
      const done = completedIds.has(level.id) ? ' ✓' : '';
      button(
        this,
        this.scale.width / 2,
        140 + index * 80,
        `${level.name}${done}`,
        () => this.scene.start('character-select', { levelId: level.id }),
        { width: 420 },
      );
    });

    const unopened = [...conn.db.player_lootbox.iter()].filter((row) => !row.opened).length;
    button(
      this,
      this.scale.width / 2,
      this.scale.height - 56,
      unopened > 0 ? `Rewards (${unopened} unopened)` : 'Rewards',
      () => this.scene.start('collection'),
      { width: 420, small: true },
    );
  }
}
