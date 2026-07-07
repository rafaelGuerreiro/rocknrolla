/**
 * Dev-only asset gallery (components.html). Renders every SVG in
 * `levels/components/` straight from disk so saving a file in the IDE
 * live-reloads the page, plus the roller character bodies from
 * `rollers.ts`. A viewer, not an editor.
 */
import { TILE, contentHash } from './levels';
import { ROLLER_BODY_SVG } from './rollers';

const files = import.meta.glob('../../levels/components/*.svg', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const TILE_NAMES = new Map<number, string>(
  Object.entries(TILE).map(([name, id]) => [id, name.toLowerCase()]),
);

/** Overlay stroke per marker kind; anything unknown renders red. */
const MARKER_COLORS = new Map<number, string>([
  [TILE.SOLID, '#3b82f6'],
  [TILE.SLOPE_UP, '#3b82f6'],
  [TILE.SLOPE_DOWN, '#3b82f6'],
  [TILE.LETHAL, '#dc2626'],
  [TILE.WATER, '#06b6d4'],
  [TILE.FIRE, '#f97316'],
  [TILE.HEAVY, '#8b5cf6'],
]);

interface Card {
  svg: SVGSVGElement;
  overlay: SVGGElement;
  width: number;
  height: number;
}

const cards: Card[] = [];

function slugOf(path: string): string {
  return path
    .split('/')
    .pop()!
    .replace(/\.svg$/, '');
}

/** Component SVGs carry explicit width/height; roller bodies only have a viewBox. */
function dimensionsOf(root: SVGSVGElement): { width: number; height: number } {
  const width = Number(root.getAttribute('width'));
  const height = Number(root.getAttribute('height'));
  if (width && height) return { width, height };
  const viewBox = root.getAttribute('viewBox')?.split(/\s+/).map(Number);
  return viewBox ? { width: viewBox[2], height: viewBox[3] } : { width: 0, height: 0 };
}

function buildOverlay(svg: SVGSVGElement): SVGGElement {
  const overlay = svg.ownerDocument.createElementNS(
    'http://www.w3.org/2000/svg',
    'g',
  );
  overlay.setAttribute('data-overlay', '');
  overlay.setAttribute('display', 'none');
  for (const marker of svg.querySelectorAll('[data-t]')) {
    const t = Number(marker.getAttribute('data-t'));
    const color = MARKER_COLORS.get(t) ?? '#dc2626';
    const clone = marker.cloneNode(false) as SVGElement;
    clone.setAttribute('fill', color);
    clone.setAttribute('fill-opacity', '0.25');
    clone.setAttribute('stroke', color);
    clone.setAttribute('stroke-width', '2');
    const title = svg.ownerDocument.createElementNS(
      'http://www.w3.org/2000/svg',
      'title',
    );
    title.textContent = TILE_NAMES.get(t) ?? `tile ${t}`;
    clone.appendChild(title);
    overlay.appendChild(clone);
  }
  svg.appendChild(overlay);
  return overlay;
}

function addCard(gallery: HTMLElement, path: string, raw: string): void {
  const slug = slugOf(path);
  const doc = new DOMParser().parseFromString(raw, 'image/svg+xml');
  const card = document.createElement('div');
  card.className = 'card';
  if (doc.querySelector('parsererror')) {
    card.innerHTML = `<div class="meta"><span class="slug"></span><span>does not parse as SVG</span></div>`;
    card.querySelector('.slug')!.textContent = slug;
    gallery.appendChild(card);
    return;
  }
  const svg = document.importNode(doc.documentElement, true) as unknown;
  const root = svg as SVGSVGElement;
  const { width, height } = dimensionsOf(root);
  // Roller bodies only carry a viewBox; give every card an explicit
  // intrinsic size so it doesn't stretch to fill the flex row.
  root.setAttribute('width', String(width));
  root.setAttribute('height', String(height));
  const overlay = buildOverlay(root);
  const hash = contentHash(width, height, new TextEncoder().encode(raw));

  const stage = document.createElement('div');
  stage.className = 'stage';
  stage.appendChild(root);
  const meta = document.createElement('div');
  meta.className = 'meta';
  const markers = overlay.childElementCount;
  meta.innerHTML = `<span class="slug"></span><span>${width}×${height}</span><span>${markers} marker${markers === 1 ? '' : 's'}</span><span class="hash">${hash}</span>`;
  meta.querySelector('.slug')!.textContent = slug;
  card.append(stage, meta);
  gallery.appendChild(card);
  cards.push({ svg: root, overlay, width, height });
}

function applyZoom(zoom: number): void {
  for (const card of cards) {
    card.svg.setAttribute('width', String(card.width * zoom));
    card.svg.setAttribute('height', String(card.height * zoom));
  }
}

function applyColliders(visible: boolean): void {
  for (const card of cards) {
    card.overlay.setAttribute('display', visible ? 'inline' : 'none');
  }
}

const gallery = document.querySelector<HTMLElement>('#gallery')!;
const paths = Object.keys(files).sort();
if (paths.length === 0) {
  gallery.innerHTML = `<div class="empty">No components found in <code>levels/components/</code>.</div>`;
}
for (const path of paths) addCard(gallery, path, files[path]);

const characters = document.querySelector<HTMLElement>('#characters')!;
for (const style of Object.keys(ROLLER_BODY_SVG).sort()) {
  addCard(characters, `characters/${style}.svg`, ROLLER_BODY_SVG[style]);
}

const zoom = document.querySelector<HTMLSelectElement>('#zoom')!;
zoom.addEventListener('change', () => applyZoom(Number(zoom.value)));

const colliders = document.querySelector<HTMLInputElement>('#colliders')!;
colliders.addEventListener('change', () => applyColliders(colliders.checked));

const background = document.querySelector<HTMLSelectElement>('#background')!;
background.addEventListener('change', () => {
  document.body.className = `bg-${background.value}`;
});
