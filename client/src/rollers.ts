import Phaser from 'phaser';
import { svgDataUrl } from './tiles';

/**
 * The five two-layer rollers, copied verbatim from the approved design
 * (design_handoff_rocknrolla/source/RocknRolla-Characters.dc.html).
 *
 * Bodies are faceless and rotate with the physics; faces are a shared set
 * of expressions layered on top, kept upright and swapped on events. Keys
 * are the backend `vw_character_v1.style` values and expression names.
 */
const body = (defs: string, art: string): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120"><defs>${defs}</defs>${art}</svg>`;

const face = (art: string): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 50">${art}</svg>`;

export const ROLLER_BODY_SVG: Record<string, string> = {
  // Rock · angular ten-sided boulder
  rock: body(
    '<radialGradient id="c-rock" cx="36%" cy="28%"><stop offset="0" stop-color="#bcb1a1"/><stop offset=".6" stop-color="#736a5c"/><stop offset="1" stop-color="#463f36"/></radialGradient>',
    '<polygon points="58,9 86,17 109,43 106,73 90,100 60,113 33,104 11,77 17,44 33,19" fill="url(#c-rock)"/>' +
      '<polygon points="58,9 33,19 17,44 33,52 60,30" fill="#ffffff" opacity=".14"/>' +
      '<polygon points="106,73 90,100 60,113 60,72" fill="#000000" opacity=".16"/>' +
      '<path d="M26 62 l14 -5 l9 6" fill="none" stroke="#40382f" stroke-width="2.4" stroke-linecap="round" opacity=".5"/>' +
      '<path d="M78 34 l-10 8" fill="none" stroke="#40382f" stroke-width="2" stroke-linecap="round" opacity=".4"/>',
  ),
  // Gem Shard · long-axis crystal
  gem: body(
    '<linearGradient id="c-gem" x1="0" y1="0" x2="0.6" y2="1"><stop offset="0" stop-color="#8ff0f0"/><stop offset=".5" stop-color="#3fb6bd"/><stop offset="1" stop-color="#237580"/></linearGradient>',
    '<polygon points="66,4 90,40 78,88 58,116 42,80 38,40 52,16" fill="url(#c-gem)"/>' +
      '<polygon points="66,4 52,16 38,40 58,44" fill="#c6fbfb" opacity=".55"/>' +
      '<polygon points="66,4 90,40 58,44" fill="#ffffff" opacity=".28"/>' +
      '<path d="M38 40 L78 44 M58 44 L58 116 M58 44 L42 80 M58 44 L78 88" stroke="#ffffff" stroke-width="1.4" opacity=".4" fill="none"/>' +
      '<polygon points="78,88 58,116 58,72" fill="#12525a" opacity=".35"/>',
  ),
  // Egg · off-center weight
  egg: body(
    '<radialGradient id="c-egg" cx="40%" cy="30%"><stop offset="0" stop-color="#fffaf0"/><stop offset="1" stop-color="#e9d6ba"/></radialGradient>',
    '<path d="M60 8 C42 8 31 36 31 62 C31 92 44 114 60 114 C76 114 89 92 89 62 C89 36 78 8 60 8 Z" fill="url(#c-egg)"/>' +
      '<ellipse cx="47" cy="38" rx="12" ry="17" fill="#ffffff" opacity=".5"/>' +
      '<circle cx="76" cy="70" r="4" fill="#f6b8c4" opacity=".55"/><circle cx="42" cy="74" r="4" fill="#f6b8c4" opacity=".55"/>',
  ),
  // Coconut · lumpy but near-round
  coco: body(
    '<radialGradient id="c-coco" cx="38%" cy="30%"><stop offset="0" stop-color="#b07c50"/><stop offset=".55" stop-color="#7d4e2c"/><stop offset="1" stop-color="#4e2f1a"/></radialGradient>',
    '<path d="M60 8 C84 6 104 24 106 48 C108 62 112 76 100 92 C88 108 70 114 56 112 C40 110 22 100 16 82 C10 66 10 46 20 32 C30 16 42 10 60 8 Z" fill="url(#c-coco)"/>' +
      '<path d="M34 30 C44 24 52 26 58 20 M78 34 C86 40 90 50 88 58 M30 74 C36 82 46 86 54 84 M84 78 C78 88 68 92 60 90" fill="none" stroke="#3a2214" stroke-width="2" stroke-linecap="round" opacity=".45"/>' +
      '<ellipse cx="43" cy="30" rx="13" ry="8" fill="#ffffff" opacity=".14"/>' +
      '<circle cx="86" cy="86" r="2.6" fill="#33200f" opacity=".6"/><circle cx="96" cy="74" r="2.4" fill="#33200f" opacity=".5"/><circle cx="90" cy="96" r="2.2" fill="#33200f" opacity=".5"/>',
  ),
  // Paper Ball · crumpled featherweight
  paper: body(
    '<radialGradient id="c-paper" cx="40%" cy="32%"><stop offset="0" stop-color="#fbf6ea"/><stop offset="1" stop-color="#d8cdb8"/></radialGradient>',
    '<polygon points="60,8 74,15 87,10 93,26 107,33 100,49 110,62 97,74 102,92 85,93 74,106 60,99 45,107 37,92 20,91 27,73 12,60 25,49 17,33 33,28 39,13 53,19" fill="url(#c-paper)"/>' +
      '<path d="M60 20 L62 58 L50 84 M92 30 L62 58 L100 66 M22 58 L62 58 L34 90 M78 96 L62 58" fill="none" stroke="#b7ac94" stroke-width="1.8" stroke-linecap="round" opacity=".7"/>' +
      '<path d="M40 32 L62 58 M84 78 L62 58" fill="none" stroke="#fffdf7" stroke-width="1.6" stroke-linecap="round" opacity=".8"/>' +
      '<polygon points="60,8 53,19 39,13 60,20" fill="#ffffff" opacity=".3"/>' +
      '<polygon points="85,93 74,106 60,99 78,88" fill="#b7ac94" opacity=".28"/>',
  ),
};

export type FaceName =
  'happy' | 'determined' | 'surprised' | 'nervous' | 'dizzy';

export const FACE_SVG: Record<FaceName, string> = {
  happy: face(
    '<ellipse cx="28" cy="20" rx="6" ry="8" fill="#241d16"/><ellipse cx="52" cy="20" rx="6" ry="8" fill="#241d16"/>' +
      '<circle cx="30" cy="17" r="2" fill="#fff"/><circle cx="54" cy="17" r="2" fill="#fff"/>' +
      '<path d="M30 33 q10 9 20 0" fill="none" stroke="#241d16" stroke-width="3.4" stroke-linecap="round"/>',
  ),
  determined: face(
    '<path d="M18 9 L34 15 M62 9 L46 15" stroke="#241d16" stroke-width="3.2" stroke-linecap="round" fill="none"/>' +
      '<ellipse cx="28" cy="22" rx="6" ry="8" fill="#241d16"/><ellipse cx="52" cy="22" rx="6" ry="8" fill="#241d16"/>' +
      '<circle cx="30" cy="19" r="2" fill="#fff"/><circle cx="54" cy="19" r="2" fill="#fff"/>' +
      '<path d="M31 36 q9 5 18 0" fill="none" stroke="#241d16" stroke-width="3.2" stroke-linecap="round"/>',
  ),
  surprised: face(
    '<ellipse cx="28" cy="19" rx="7" ry="9.5" fill="#241d16"/><ellipse cx="52" cy="19" rx="7" ry="9.5" fill="#241d16"/>' +
      '<circle cx="30" cy="15" r="2.4" fill="#fff"/><circle cx="54" cy="15" r="2.4" fill="#fff"/>' +
      '<ellipse cx="40" cy="36" rx="5.5" ry="6.5" fill="#241d16"/>',
  ),
  nervous: face(
    '<ellipse cx="28" cy="20" rx="6.5" ry="8.5" fill="#241d16"/><ellipse cx="52" cy="20" rx="6.5" ry="8.5" fill="#241d16"/>' +
      '<circle cx="26" cy="18" r="2" fill="#fff"/><circle cx="50" cy="18" r="2" fill="#fff"/>' +
      '<path d="M30 35 q4 -4 8 0 t8 0" fill="none" stroke="#241d16" stroke-width="3" stroke-linecap="round"/>' +
      '<path d="M64 8 c-3.5 5 -3.5 8 0 8 c3.5 0 3.5 -3 0 -8 z" fill="#7fd4ff"/>',
  ),
  dizzy: face(
    '<path d="M23 14 L33 24 M33 14 L23 24 M47 14 L57 24 M57 14 L47 24" stroke="#241d16" stroke-width="3" stroke-linecap="round" fill="none"/>' +
      '<path d="M30 35 q5 5 10 0 t10 0" fill="none" stroke="#241d16" stroke-width="3" stroke-linecap="round"/>',
  ),
};

/**
 * Body SVG for a backend style. Styles outside the designed five render as
 * the rock body until the seed catches up.
 */
export function rollerBodySvg(style: string): string {
  return ROLLER_BODY_SVG[style] ?? ROLLER_BODY_SVG.rock;
}

export function rollerBodyDataUrl(style: string): string {
  return svgDataUrl(rollerBodySvg(style));
}

/** Flatten every color to dark ink while keeping the alpha silhouette. */
const SILHOUETTE_FILTER =
  '<filter id="sil"><feColorMatrix type="matrix" values="0 0 0 0 0.08 0 0 0 0 0.04 0 0 0 0 0.07 0 0 0 1 0"/></filter>';

/**
 * Silhouette variant of a body for locked collection cards, baked into the
 * texture itself so it renders identically on WebGL and Canvas (Phaser's
 * fill-tint is unreliable on the Canvas renderer).
 */
export function rollerSilhouetteDataUrl(style: string): string {
  // ponytail: string surgery relies on the fixed body() structure above.
  const svg = rollerBodySvg(style)
    .replace('<defs>', `<defs>${SILHOUETTE_FILTER}`)
    .replace('</defs>', '</defs><g filter="url(#sil)">')
    .replace('</svg>', '</g></svg>');
  return svgDataUrl(svg);
}

export function silhouetteTextureKey(style: string): string {
  return `roller_${style}_sil`;
}

export function faceTextureKey(name: FaceName): string {
  return `face_${name}`;
}

/** Face proportions relative to the body size (from the design mockup). */
export const FACE_WIDTH_RATIO = 0.5;
export const FACE_ASPECT = 50 / 80;
export const FACE_OFFSET_Y_RATIO = 0.04;

/**
 * Compose a body + upright face at a given display size. The returned
 * container can bob/scale; for physics-driven rollers keep the layers as
 * separate scene objects instead (see GameScene) so only the body spins.
 */
export function addRoller(
  scene: Phaser.Scene,
  x: number,
  y: number,
  size: number,
  style: string,
  expr: FaceName = 'happy',
): Phaser.GameObjects.Container {
  const bodyImage = scene.add
    .image(0, 0, `roller_${style}`)
    .setDisplaySize(size, size);
  const faceWidth = size * FACE_WIDTH_RATIO;
  const faceImage = scene.add
    .image(0, size * FACE_OFFSET_Y_RATIO, faceTextureKey(expr))
    .setDisplaySize(faceWidth, faceWidth * FACE_ASPECT);
  const container = scene.add.container(x, y, [bodyImage, faceImage]);
  container.setSize(size, size);
  return container;
}
