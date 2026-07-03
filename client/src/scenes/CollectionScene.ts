import Phaser from 'phaser';
import { db } from '../db';
import { UI_FONT, button, note, title } from '../ui';

/** Unopened lootboxes plus the per-character piece collection. */
export class CollectionScene extends Phaser.Scene {
  private opening = false;

  constructor() {
    super('collection');
  }

  create(): void {
    this.opening = false;
    const conn = db();
    title(this, 'Rewards');

    const unopened = [...conn.db.player_lootbox.iter()]
      .filter((row) => !row.opened)
      .sort((a, b) => (a.id < b.id ? -1 : 1));
    const lootboxNames = new Map([...conn.db.lootbox_def.iter()].map((row) => [row.id, row.name]));

    if (unopened.length === 0) {
      note(this, 130, 'No unopened lootboxes. Complete levels to earn more.');
    } else {
      const box = unopened[0];
      button(
        this,
        this.scale.width / 2,
        140,
        `Open ${lootboxNames.get(box.lootboxId) ?? box.lootboxId} (${unopened.length} left)`,
        () => this.openLootbox(box.id),
        { width: 460 },
      );
    }

    this.drawCollection();
    button(this, 120, this.scale.height - 56, 'Back', () => this.scene.start('level-select'), {
      width: 160,
      small: true,
    });
  }

  private openLootbox(playerLootboxId: bigint): void {
    if (this.opening) return;
    this.opening = true;
    const conn = db();

    const timeout = this.time.delayedCall(6000, () => {
      conn.db.player_lootbox.removeOnUpdate(onUpdate);
      this.opening = false;
      note(this, 200, 'The server did not answer. Check the connection and try again.');
    });
    const failed = (error: unknown) => {
      conn.db.player_lootbox.removeOnUpdate(onUpdate);
      timeout.remove();
      this.opening = false;
      note(this, 200, `Could not open the lootbox: ${error instanceof Error ? error.message : error}`);
    };
    const onUpdate = (
      _ctx: unknown,
      _old: { id: bigint },
      row: { id: bigint; opened: boolean; awardedPieceId?: string },
    ) => {
      if (row.id !== playerLootboxId || !row.opened || !row.awardedPieceId) return;
      conn.db.player_lootbox.removeOnUpdate(onUpdate);
      timeout.remove();
      this.reveal(row.awardedPieceId);
    };
    conn.db.player_lootbox.onUpdate(onUpdate);
    conn.reducers.openLootbox({ playerLootboxId }).catch(failed);
  }

  /** Animate the server-decided award; the client never chooses the piece. */
  private reveal(pieceId: string): void {
    const conn = db();
    const piece = [...conn.db.piece_def.iter()].find((row) => row.id === pieceId);
    const character = piece
      ? [...conn.db.character_def.iter()].find((row) => row.id === piece.characterId)
      : undefined;
    const centerX = this.scale.width / 2;
    const centerY = this.scale.height / 2;

    const dim = this.add
      .rectangle(centerX, centerY, this.scale.width, this.scale.height, 0x060810, 0.85)
      .setDepth(10)
      .setInteractive();
    const label = this.add
      .text(centerX, centerY - 20, piece?.name ?? pieceId, {
        fontFamily: UI_FONT,
        fontSize: '40px',
        color: '#f5c451',
      })
      .setOrigin(0.5)
      .setDepth(11)
      .setScale(0.1);
    const sub = this.add
      .text(centerX, centerY + 34, `${character?.name ?? '?'} piece — tap to continue`, {
        fontFamily: UI_FONT,
        fontSize: '20px',
        color: '#9aa7c0',
      })
      .setOrigin(0.5)
      .setDepth(11)
      .setAlpha(0);
    this.tweens.add({ targets: label, scale: 1, duration: 450, ease: 'Back.Out' });
    this.tweens.add({ targets: sub, alpha: 1, delay: 350, duration: 300 });
    dim.once('pointerup', () => this.scene.restart());
  }

  private drawCollection(): void {
    const conn = db();
    const counts = new Map(
      [...conn.db.player_piece.iter()].map((row) => [row.pieceId, row.count]),
    );
    const unlocked = new Set(
      [...conn.db.player_unlocked_character.iter()].map((row) => row.characterId),
    );
    const pieces = [...conn.db.piece_def.iter()];
    const characters = [...conn.db.character_def.iter()].sort((a, b) => a.id.localeCompare(b.id));

    let y = 220;
    for (const character of characters) {
      const state = unlocked.has(character.id) ? 'unlocked' : 'locked';
      this.add
        .text(150, y, `${character.name} (${state})`, {
          fontFamily: UI_FONT,
          fontSize: '22px',
          color: unlocked.has(character.id) ? '#e8ecf5' : '#6b7280',
        })
        .setOrigin(0, 0.5);
      const line = pieces
        .filter((piece) => piece.characterId === character.id)
        .map((piece) => `${piece.name} ×${counts.get(piece.id) ?? 0}`)
        .join('   ');
      this.add
        .text(150, y + 28, line || 'No pieces defined', {
          fontFamily: UI_FONT,
          fontSize: '17px',
          color: '#9aa7c0',
        })
        .setOrigin(0, 0.5);
      y += 80;
    }
  }
}
