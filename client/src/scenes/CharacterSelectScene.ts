import Phaser from 'phaser';
import { Uuid } from 'spacetimedb';
import { characterSpriteKey } from '../assets';
import { db } from '../db';
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
    const conn = db();
    const unlocked = new Set(
      [...conn.db.vw_my_unlocked_character.iter()].map((row) =>
        row.characterId.toString(),
      ),
    );
    const characters = [...conn.db.vw_character.iter()].sort((a, b) =>
      a.id.toString().localeCompare(b.id.toString()),
    );
    const eligible = characters.filter((character) =>
      unlocked.has(character.id.toString()),
    );

    // One eligible character is not a choice: proceed with it directly.
    if (eligible.length === 1) {
      this.startRun(eligible[0].id.toString());
      return;
    }

    title(this, 'Choose your character');
    if (eligible.length === 0) {
      note(
        this,
        this.scale.height / 2,
        'No unlocked characters. Collect pieces to unlock one.',
      );
    }

    const owned = new Set(
      [...conn.db.vw_my_piece.iter()]
        .filter((row) => row.count > 0)
        .map((row) => row.pieceId.toString()),
    );
    const pieces = [...conn.db.vw_piece.iter()];

    characters.forEach((character, index) => {
      const x =
        this.scale.width / 2 + (index - (characters.length - 1) / 2) * 260;
      const y = this.scale.height / 2 - 20;
      const characterId = character.id.toString();
      const isUnlocked = unlocked.has(characterId);

      const spriteKey = characterSpriteKey(character.style);
      if (this.textures.exists(spriteKey)) {
        const sprite = this.add.image(x, y - 60, spriteKey);
        if (!isUnlocked) sprite.setAlpha(0.3);
      }

      const required = pieces.filter(
        (piece) => piece.characterId.toString() === characterId,
      );
      const have = required.filter((piece) =>
        owned.has(piece.id.toString()),
      ).length;
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
        () => this.startRun(characterId),
        {
          width: 220,
          disabled: !isUnlocked,
        },
      );
    });

    note(
      this,
      110,
      'Stats come from the server and change how the character rolls, jumps, floats, and burns.',
    );
    button(
      this,
      120,
      this.scale.height - 56,
      'Back',
      () => this.scene.start('level-select'),
      {
        width: 160,
        small: true,
      },
    );
  }

  private startRun(characterId: string): void {
    const conn = db();
    conn.reducers
      .selectCharacter({ characterId: Uuid.parse(characterId) })
      .catch((error) => console.error('selectCharacter failed:', error));
    this.scene.start('game', { levelId: this.levelId, characterId });
  }
}
