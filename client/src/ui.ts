import Phaser from 'phaser';

// Logical design space every scene lays out in. The canvas backing store is
// DPR times larger (see main.ts) and each scene's camera zooms by DPR, so
// coordinates stay 960×540 while rendering is retina-crisp.
export const VIEW_W = 960;
export const VIEW_H = 540;
export const DPR = Math.min(window.devicePixelRatio || 1, 2);

/** Zoom the scene camera so the DPR-scaled canvas shows the logical view. */
export function setupCamera(scene: Phaser.Scene): void {
  scene.cameras.main.setZoom(DPR).centerOn(VIEW_W / 2, VIEW_H / 2);
}

// "Claymation Dusk" theme kit — see design_handoff_rocknrolla/README.md.
export const UI_FONT = 'Fredoka, Nunito, sans-serif';
export const BODY_FONT = 'Nunito, sans-serif';
export const MONO_FONT = '"Space Mono", monospace';

export const INK = '#3a2a22';
export const INK_DARK = '#241d16';
export const CREAM = 0xf5ecd8;
export const CREAM_TEXT = '#f5ecd8';
export const AMBER = '#f2a63c';
export const AMBER_LIGHT = 0xffce7a;
export const AMBER_DEEP = 0xf2932f;
export const AMBER_SHADOW = 0xb5651f;
export const CREAM_SHADOW = 0x9a8261;
export const NOTE_ON_DARK = '#e6c9a0';
export const NOTE_ON_CREAM = '#9a7d5c';
export const STAR_GOLD = '#ffce5c';

/** Per-stat accent colors for stat pips (design tokens). */
export const STAT_ACCENTS: Record<string, number> = {
  Density: 0xc66240,
  Jump: 0x6d7a44,
  Hold: 0x8a4a56,
  Buoyancy: 0x3f7d8c,
  'Fire Resist': 0xf2a63c,
};

export interface ButtonOptions {
  width?: number;
  disabled?: boolean;
  small?: boolean;
  /** 'amber' CTA (default) or 'cream' secondary. */
  variant?: 'amber' | 'cream';
}

/**
 * Large-touch-target chunky button: gradient face over the signature
 * `0 6px 0` offset shadow block, used by every menu scene.
 */
export function button(
  scene: Phaser.Scene,
  x: number,
  y: number,
  label: string,
  onTap: () => void,
  options: ButtonOptions = {},
): Phaser.GameObjects.Container {
  const width = options.width ?? 300;
  const height = options.small ? 52 : 64;
  const cream = options.variant === 'cream';
  const radius = Math.min(18, height / 2 - 2);

  const g = scene.add.graphics();
  const drawFace = (offsetY: number) => {
    g.clear();
    const shadow = options.disabled
      ? 0x6b5a4a
      : cream
        ? CREAM_SHADOW
        : AMBER_SHADOW;
    g.fillStyle(shadow, 1);
    g.fillRoundedRect(-width / 2, -height / 2 + 6, width, height, radius);
    if (options.disabled) {
      g.fillStyle(0xcdbda8, 1);
    } else if (cream) {
      g.fillStyle(CREAM, 1);
    } else {
      g.fillGradientStyle(AMBER_LIGHT, AMBER_LIGHT, AMBER_DEEP, AMBER_DEEP, 1);
    }
    g.fillRoundedRect(-width / 2, -height / 2 + offsetY, width, height, radius);
  };
  drawFace(0);

  const text = scene.add
    .text(0, 0, label, {
      fontFamily: UI_FONT,
      fontSize: options.small ? '20px' : '24px',
      fontStyle: '600',
      color: options.disabled ? '#8a7a68' : cream ? INK : '#5a2f14',
      align: 'center',
    })
    .setOrigin(0.5);
  const hit = scene.add.rectangle(0, 2, width, height + 8, 0xffffff, 0.0001);
  const container = scene.add.container(x, y, [g, text, hit]);
  container.setSize(width, height);
  if (!options.disabled) {
    hit.setInteractive({ useHandCursor: true });
    hit.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      pointer.event?.stopPropagation?.();
      drawFace(4);
      text.setY(4);
    });
    const release = () => {
      drawFace(0);
      text.setY(0);
    };
    hit.on('pointerout', release);
    hit.on('pointerup', () => {
      release();
      onTap();
    });
  }
  return container;
}

export function title(
  scene: Phaser.Scene,
  text: string,
): Phaser.GameObjects.Text {
  return scene.add
    .text(VIEW_W / 2, 56, text, {
      fontFamily: UI_FONT,
      fontSize: '34px',
      fontStyle: '700',
      color: CREAM_TEXT,
    })
    .setOrigin(0.5)
    .setShadow(0, 3, 'rgba(36,29,22,0.55)', 4);
}

export function note(
  scene: Phaser.Scene,
  y: number,
  text: string,
): Phaser.GameObjects.Text {
  return scene.add
    .text(VIEW_W / 2, y, text, {
      fontFamily: BODY_FONT,
      fontSize: '18px',
      fontStyle: '600',
      color: NOTE_ON_DARK,
      align: 'center',
      wordWrap: { width: VIEW_W - 120 },
    })
    .setOrigin(0.5);
}

/**
 * Cream rounded pill with a soft drop shadow. Returns a container holding
 * only the background; callers add their own content, or pass `label` for
 * the common text-only case.
 */
export function pill(
  scene: Phaser.Scene,
  x: number,
  y: number,
  width: number,
  height: number,
  label?: string,
): Phaser.GameObjects.Container {
  const g = scene.add.graphics();
  const radius = height / 2;
  g.fillStyle(0x140a12, 0.28);
  g.fillRoundedRect(-width / 2 + 2, -height / 2 + 4, width, height, radius);
  g.fillStyle(CREAM, 1);
  g.fillRoundedRect(-width / 2, -height / 2, width, height, radius);
  const children: Phaser.GameObjects.GameObject[] = [g];
  if (label) {
    children.push(
      scene.add
        .text(0, 0, label, {
          fontFamily: UI_FONT,
          fontSize: `${Math.round(height * 0.44)}px`,
          fontStyle: '600',
          color: INK,
        })
        .setOrigin(0.5),
    );
  }
  const container = scene.add.container(x, y, children);
  container.setSize(width, height);
  return container;
}

export interface StatBar {
  label: string;
  /** Filled pips, 0–5. */
  value: number;
}

/**
 * Rows of 5 rounded pips per stat, filled in the stat's accent color.
 * Top-left anchored at (x, y); rows are `rowGap` apart.
 */
export function statBars(
  scene: Phaser.Scene,
  x: number,
  y: number,
  stats: StatBar[],
  options: { rowGap?: number; pipWidth?: number } = {},
): Phaser.GameObjects.Container {
  const rowGap = options.rowGap ?? 34;
  const pipWidth = options.pipWidth ?? 22;
  const pipHeight = 10;
  const container = scene.add.container(x, y);
  const g = scene.add.graphics();
  stats.forEach((stat, row) => {
    const rowY = row * rowGap;
    container.add(
      scene.add
        .text(0, rowY, stat.label.toUpperCase(), {
          fontFamily: MONO_FONT,
          fontSize: '11px',
          color: NOTE_ON_CREAM,
        })
        .setOrigin(0, 0.5)
        .setLetterSpacing(2),
    );
    const accent = STAT_ACCENTS[stat.label] ?? 0xc66240;
    for (let i = 0; i < 5; i++) {
      g.fillStyle(i < stat.value ? accent : 0xe2d6bd, 1);
      g.fillRoundedRect(i * (pipWidth + 5), rowY + 8, pipWidth, pipHeight, 4);
    }
  });
  container.add(g);
  return container;
}

/** Map a raw stat value onto 0–5 pips over the design's tuning range. */
export function pips(value: number, min: number, max: number): number {
  const t = (value - min) / (max - min);
  return Phaser.Math.Clamp(Math.round(1 + t * 4), 0, 5);
}

/** Five pip rows for a character's real backend fields. */
export function characterStatBars(character: {
  density: number;
  jumpSpeed: number;
  buoyancy: number;
  fireResistance: number;
}): StatBar[] {
  return [
    { label: 'Density', value: pips(character.density, 0.001, 0.0045) },
    { label: 'Jump', value: pips(character.jumpSpeed, 8, 15) },
    { label: 'Buoyancy', value: pips(character.buoyancy, 0.1, 0.9) },
    { label: 'Fire Resist', value: pips(character.fireResistance, 0.2, 1) },
  ];
}

/** Signature trait caption shown under characters (derived from stats). */
export function characterTrait(character: {
  density: number;
  jumpSpeed: number;
  buoyancy: number;
  fireResistance: number;
}): string {
  if (character.fireResistance >= 0.9) return 'FIREPROOF';
  if (character.buoyancy >= 0.85) return 'VERY FLOATY';
  if (character.buoyancy >= 0.6) return 'FLOATS';
  if (character.jumpSpeed >= 14) return 'SPRINGY';
  if (character.density >= 0.004) return 'DENSE · SINKS';
  if (character.density >= 0.0035) return 'SHOVES HEAVY';
  return 'ALL-ROUNDER';
}
