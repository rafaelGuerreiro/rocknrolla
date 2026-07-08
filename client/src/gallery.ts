/**
 * Dev-only asset gallery (components.html). Renders every authored SVG in
 * `content/` straight from disk so saving a file in the IDE live-reloads
 * the page: components (with collider overlays), character bodies (plus
 * their import-derived silhouettes), faces, and backdrops (layers plus a
 * composed preview). A viewer, not an editor.
 */
import { TILE, contentHash } from './levels';

const glob = (files: Record<string, unknown>) =>
  files as Record<string, string>;

const componentFiles = glob(
  import.meta.glob('../../content/components/*.svg', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
);
const characterFiles = glob(
  import.meta.glob('../../content/characters/*.svg', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
);
const faceFiles = glob(
  import.meta.glob('../../content/faces/*.svg', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
);
const backdropFiles = glob(
  import.meta.glob('../../content/backdrops/*.svg', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
);

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

/** Every authored content file carries explicit width/height; the viewBox
 * fallback keeps malformed files visible instead of zero-sized. */
function dimensionsOf(root: SVGSVGElement): { width: number; height: number } {
  const width = Number(root.getAttribute('width'));
  const height = Number(root.getAttribute('height'));
  if (width && height) return { width, height };
  const viewBox = root.getAttribute('viewBox')?.split(/\s+/).map(Number);
  return viewBox
    ? { width: viewBox[2], height: viewBox[3] }
    : { width: 0, height: 0 };
}

/**
 * The importer's silhouette derivation (see admin `charactersrc.rs`), so
 * the gallery previews exactly what will be imported for locked cards.
 */
const SILHOUETTE_FILTER =
  '<filter id="sil"><feColorMatrix type="matrix" values="0 0 0 0 0.08 0 0 0 0 0.04 0 0 0 0 0.07 0 0 0 1 0"/></filter>';

function deriveSilhouette(bodySvg: string): string {
  return bodySvg
    .replace('<defs>', `<defs>${SILHOUETTE_FILTER}`)
    .replace('</defs>', '</defs><g filter="url(#sil)">')
    .replace('</svg>', '</g></svg>');
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

function renderSection(
  selector: string,
  files: Record<string, string>,
  emptyLabel: string,
): void {
  const section = document.querySelector<HTMLElement>(selector)!;
  const paths = Object.keys(files).sort();
  if (paths.length === 0) {
    section.innerHTML = `<div class="empty">No SVG files found in <code>${emptyLabel}</code>.</div>`;
    return;
  }
  for (const path of paths) addCard(section, path, files[path]);
}

renderSection('#gallery', componentFiles, 'content/components/');
renderSection('#faces', faceFiles, 'content/faces/');

// Characters: each body next to the silhouette the importer will derive.
const characters = document.querySelector<HTMLElement>('#characters')!;
const characterPaths = Object.keys(characterFiles).sort();
if (characterPaths.length === 0) {
  characters.innerHTML = `<div class="empty">No SVG files found in <code>content/characters/</code>.</div>`;
}
for (const path of characterPaths) {
  addCard(characters, path, characterFiles[path]);
  addCard(
    characters,
    path.replace(/\.svg$/, '.silhouette.svg'),
    deriveSilhouette(characterFiles[path]),
  );
}

// Backdrops: the three layer cards plus a composed preview per slug.
const backdrops = document.querySelector<HTMLElement>('#backdrops')!;
const backdropPaths = Object.keys(backdropFiles).sort();
if (backdropPaths.length === 0) {
  backdrops.innerHTML = `<div class="empty">No SVG files found in <code>content/backdrops/</code>.</div>`;
}
for (const path of backdropPaths) addCard(backdrops, path, backdropFiles[path]);

function composedBackdropCard(slug: string, layers: Map<string, string>): void {
  const sky = layers.get('sky');
  const far = layers.get('far');
  const mid = layers.get('mid');
  if (!sky || !far || !mid) return; // incomplete: layer cards already show it
  const dataUrl = (svg: string) => `data:image/svg+xml;base64,${btoa(svg)}`;
  const card = document.createElement('div');
  card.className = 'card';
  const stage = document.createElement('div');
  stage.className = 'stage composed';
  stage.style.width = '480px';
  const skyImg = document.createElement('img');
  skyImg.src = dataUrl(sky);
  skyImg.style.width = '100%';
  skyImg.height = 270;
  const strip = (svg: string, bottom: number, opacity: string) => {
    const img = document.createElement('img');
    img.className = 'strip';
    img.src = dataUrl(svg);
    img.style.bottom = `${bottom}px`;
    img.style.opacity = opacity;
    return img;
  };
  stage.append(skyImg, strip(far, 20, '0.9'), strip(mid, 0, '1'));
  const meta = document.createElement('div');
  meta.className = 'meta';
  meta.innerHTML = `<span class="slug"></span><span>composed</span>`;
  meta.querySelector('.slug')!.textContent = slug;
  card.append(stage, meta);
  backdrops.appendChild(card);
}

const backdropLayers = new Map<string, Map<string, string>>();
for (const path of backdropPaths) {
  const stem = slugOf(path);
  const dot = stem.lastIndexOf('.');
  if (dot < 0) continue;
  const slug = stem.slice(0, dot);
  const role = stem.slice(dot + 1);
  if (!backdropLayers.has(slug)) backdropLayers.set(slug, new Map());
  backdropLayers.get(slug)!.set(role, backdropFiles[path]);
}
for (const [slug, layers] of backdropLayers) composedBackdropCard(slug, layers);

const zoom = document.querySelector<HTMLSelectElement>('#zoom')!;
zoom.addEventListener('change', () => applyZoom(Number(zoom.value)));

const colliders = document.querySelector<HTMLInputElement>('#colliders')!;
colliders.addEventListener('change', () => applyColliders(colliders.checked));

const background = document.querySelector<HTMLSelectElement>('#background')!;
background.addEventListener('change', () => {
  document.body.className = `bg-${background.value}`;
});
