import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
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

    const unopened = [...conn.db.vw_my_lootbox.iter()]
      .filter((row) => !row.opened)
      .sort((a, b) => a.id.compareTo(b.id));

    if (unopened.length === 0) {
      note(this, 130, 'No unopened lootboxes. Complete levels to earn more.');
    } else {
      const box = unopened[0];
      button(
        this,
        this.scale.width / 2,
        140,
        `Open ${box.name} (${unopened.length} left)`,
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

  private openLootbox(playerLootboxId: Uuid): void {
    if (this.opening) return;
    this.opening = true;
    const conn = db();

    const timeout = this.time.delayedCall(6000, () => {
      conn.db.vw_my_lootbox.removeOnUpdate(onUpdate);
      this.opening = false;
      note(this, 200, 'The server did not answer. Check the connection and try again.');
    });
    const failed = (error: unknown) => {
      conn.db.vw_my_lootbox.removeOnUpdate(onUpdate);
      timeout.remove();
      this.opening = false;
      note(this, 200, `Could not open the lootbox: ${error instanceof Error ? error.message : error}`);
    };
    const onUpdate = (
      _ctx: unknown,
      _old: { id: Uuid },
      row: { id: Uuid; opened: boolean; awardedPieceId?: Uuid },
    ) => {
      if (row.id.compareTo(playerLootboxId) !== 0 || !row.opened || !row.awardedPieceId) return;
      conn.db.vw_my_lootbox.removeOnUpdate(onUpdate);
      timeout.remove();
      this.reveal(row.awardedPieceId);
    };
    conn.db.vw_my_lootbox.onUpdate(onUpdate);
    conn.reducers.openLootbox({ playerLootboxId }).catch(failed);
  }

  /** Animate the server-decided award; the client never chooses the piece. */
  private reveal(pieceId: Uuid): void {
    const conn = db();
    const piece = [...conn.db.vw_piece.iter()].find((row) => row.id.compareTo(pieceId) === 0);
    const character = piece
      ? [...conn.db.vw_character.iter()].find((row) => row.id.compareTo(piece.characterId) === 0)
      : undefined;
    const centerX = this.scale.width / 2;
    const centerY = this.scale.height / 2;

    const dim = this.add
      .rectangle(centerX, centerY, this.scale.width, this.scale.height, 0x060810, 0.85)
      .setDepth(10)
      .setInteractive();
    const label = this.add
      .text(centerX, centerY - 20, piece?.name ?? pieceId.toString(), {
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
      [...conn.db.vw_my_piece.iter()].map((row) => [row.pieceId.toString(), row.count]),
    );
    const unlocked = new Set(
      [...conn.db.vw_my_unlocked_character.iter()].map((row) => row.characterId.toString()),
    );
    const pieces = [...conn.db.vw_piece.iter()];
    const characters = [...conn.db.vw_character.iter()].sort((a, b) => a.id.compareTo(b.id));

    let y = 220;
    for (const character of characters) {
      const isUnlocked = unlocked.has(character.id.toString());
      this.add
        .text(150, y, `${character.name} (${isUnlocked ? 'unlocked' : 'locked'})`, {
          fontFamily: UI_FONT,
          fontSize: '22px',
          color: isUnlocked ? '#e8ecf5' : '#6b7280',
        })
        .setOrigin(0, 0.5);
      const line = pieces
        .filter((piece) => piece.characterId.compareTo(character.id) === 0)
        .map((piece) => `${piece.name} ×${counts.get(piece.id.toString()) ?? 0}`)
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
