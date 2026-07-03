import Phaser from 'phaser';

export const UI_FONT = 'Arial, sans-serif';

/** Large-touch-target text button used by every menu scene. */
export function button(
  scene: Phaser.Scene,
  x: number,
  y: number,
  label: string,
  onTap: () => void,
  options: { width?: number; disabled?: boolean; small?: boolean } = {},
): Phaser.GameObjects.Container {
  const width = options.width ?? 300;
  const height = options.small ? 52 : 64;
  const bg = scene.add
    .rectangle(0, 0, width, height, options.disabled ? 0x232a3a : 0x2e3a55, 1)
    .setStrokeStyle(2, options.disabled ? 0x323a4d : 0x51648f);
  const text = scene.add
    .text(0, 0, label, {
      fontFamily: UI_FONT,
      fontSize: options.small ? '20px' : '24px',
      color: options.disabled ? '#6b7280' : '#e8ecf5',
      align: 'center',
    })
    .setOrigin(0.5);
  const container = scene.add.container(x, y, [bg, text]);
  container.setSize(width, height);
  if (!options.disabled) {
    bg.setInteractive({ useHandCursor: true });
    bg.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      pointer.event?.stopPropagation?.();
    });
    bg.on('pointerup', () => onTap());
    bg.on('pointerover', () => bg.setFillStyle(0x3a4a6d));
    bg.on('pointerout', () => bg.setFillStyle(0x2e3a55));
  }
  return container;
}

export function title(scene: Phaser.Scene, text: string): Phaser.GameObjects.Text {
  return scene.add
    .text(scene.scale.width / 2, 56, text, {
      fontFamily: UI_FONT,
      fontSize: '34px',
      color: '#f5c451',
    })
    .setOrigin(0.5);
}

export function note(scene: Phaser.Scene, y: number, text: string): Phaser.GameObjects.Text {
  return scene.add
    .text(scene.scale.width / 2, y, text, {
      fontFamily: UI_FONT,
      fontSize: '18px',
      color: '#9aa7c0',
      align: 'center',
      wordWrap: { width: scene.scale.width - 120 },
    })
    .setOrigin(0.5);
}
