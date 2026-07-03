import Phaser from 'phaser';
import { db } from '../db';
import { ensureBallTexture } from '../textures';
import { UI_FONT, button, note, title } from '../ui';

export class CharacterSelectScene extends Phaser.Scene {
  private levelId!: string;

  constructor() {
    super('character-select');
  }

  init(data: { levelId: string }): void {
    this.levelId = data.levelId;
  }

  create(): void {
    title(this, 'Choose your character');
    const conn = db();

    const unlocked = new Set(
      [...conn.db.player_unlocked_character.iter()].map((row) => row.characterId),
    );
    const owned = new Set(
      [...conn.db.player_piece.iter()].filter((row) => row.count > 0).map((row) => row.pieceId),
    );
    const pieces = [...conn.db.piece_def.iter()];
    const characters = [...conn.db.character_def.iter()].sort((a, b) => a.id.localeCompare(b.id));

    characters.forEach((character, index) => {
      const x = this.scale.width / 2 + (index - (characters.length - 1) / 2) * 260;
      const y = this.scale.height / 2 - 20;
      const isUnlocked = unlocked.has(character.id);

      const ballKey = ensureBallTexture(this, character.id, character.style);
      const ball = this.add.image(x, y - 60, ballKey).setScale(2.4);
      if (!isUnlocked) ball.setAlpha(0.3);

      const required = pieces.filter((piece) => piece.characterId === character.id);
      const have = required.filter((piece) => owned.has(piece.id)).length;
      const fmt = (value: number) => parseFloat(value.toPrecision(3));
      const detail = isUnlocked
        ? `jump ${fmt(character.jumpSpeed)} · flight ${character.flightTimeMs}ms\n` +
          `density ${fmt(character.density)} · buoyancy ${fmt(character.buoyancy)}\n` +
          `fire resist ${fmt(character.fireResistance)}`
        : `Locked — pieces ${have}/${required.length}`;
      this.add
        .text(x, y + 100, detail, {
          fontFamily: UI_FONT,
          fontSize: '15px',
          color: isUnlocked ? '#9aa7c0' : '#6b7280',
          align: 'center',
          wordWrap: { width: 240 },
        })
        .setOrigin(0.5, 0);

      button(
        this,
        x,
        y + 36,
        character.name,
        () => {
          conn.reducers
            .selectCharacter({ characterId: character.id })
            .catch((error) => console.error('selectCharacter failed:', error));
          this.scene.start('game', { levelId: this.levelId, characterId: character.id });
        },
        { width: 220, disabled: !isUnlocked },
      );
    });

    note(this, 110, 'Stats come from the server and change how the ball rolls, jumps, floats, and burns.');
    button(this, 120, this.scale.height - 56, 'Back', () => this.scene.start('level-select'), {
      width: 160,
      small: true,
    });
  }
}
