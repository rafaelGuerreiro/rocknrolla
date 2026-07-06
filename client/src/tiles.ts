/**
 * The few client-side prop SVGs still drawn as standalone textures. Level
 * terrain and hazards ship inside each layer's `svg-v1` scene document
 * (generated server-side); the client only draws what moves or appears
 * outside gameplay: the dynamic heavy block and the result-screen flag.
 */

/** Data URL consumable by Phaser's SVG loader (which base64-decodes it). */
export function svgDataUrl(svg: string): string {
  return 'data:image/svg+xml;base64,' + btoa(svg);
}

const tile = (body: string): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">${body}</svg>`;

export const TILE_SVG: Record<string, string> = {
  tile_heavy: tile(
    '<rect x="3" y="3" width="94" height="94" rx="14" fill="#7d786e" stroke="#55514a" stroke-width="5"/>' +
      '<rect x="12" y="12" width="76" height="34" rx="10" fill="#98938a" opacity=".55"/>' +
      '<circle cx="17" cy="17" r="4" fill="#4a463f"/><circle cx="83" cy="17" r="4" fill="#4a463f"/>' +
      '<circle cx="17" cy="83" r="4" fill="#4a463f"/><circle cx="83" cy="83" r="4" fill="#4a463f"/>' +
      '<path d="M38 62 h24 M50 52 v20" stroke="#55514a" stroke-width="6" stroke-linecap="round"/>',
  ),
  tile_finish: tile(
    '<rect x="22" y="6" width="7" height="94" rx="3.5" fill="#241d16"/>' +
      '<rect x="29" y="10" width="60" height="36" fill="#f5ecd8"/>' +
      ['0,0', '2,0', '4,0', '1,1', '3,1', '0,2', '2,2', '4,2']
        .map((cell) => {
          const [c, r] = cell.split(',').map(Number);
          return `<rect x="${29 + c * 12}" y="${10 + r * 12}" width="12" height="12" fill="#241d16"/>`;
        })
        .join('') +
      '<circle cx="25.5" cy="8" r="5" fill="#ffce5c"/>',
  ),
};
