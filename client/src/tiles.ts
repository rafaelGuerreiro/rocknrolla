/**
 * The few client-side prop SVGs still drawn as standalone textures. Level
 * terrain and hazards ship inside the component library's SVG documents;
 * the client only draws what is level-owned rather than authored: the
 * finish pole/flag. Dynamic heavy blocks use their component's art.
 */

/** Data URL consumable by Phaser's SVG loader (which base64-decodes it). */
export function svgDataUrl(svg: string): string {
  return 'data:image/svg+xml;base64,' + btoa(svg);
}

const tile = (body: string): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">${body}</svg>`;

export const TILE_SVG: Record<string, string> = {
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
