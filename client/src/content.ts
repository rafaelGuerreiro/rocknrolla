import type { DbConnection } from './module_bindings';
// Explicit extension so this module chain also loads under `node --test`.
import { contentHash } from './levels.ts';

/**
 * DB-sourced art content: character bodies, silhouettes, the shared face
 * expression set, and backdrops. Rows arrive over the content views and are
 * indexed once after connect; textures are keyed by content hash so
 * republished art invalidates naturally, like component textures.
 */

/** The backdrop menu screens render before a level picks its own. */
export const DEFAULT_BACKDROP_SLUG = 'dusk';

export type FaceName =
  'happy' | 'determined' | 'surprised' | 'nervous' | 'dizzy';

/** SVG raster sizes: 2× their max on-screen size so they stay retina-crisp. */
export const BODY_RASTER = 320;
export const SILHOUETTE_RASTER = 128;
export const FACE_RASTER_W = 176;
export const FACE_RASTER_H = 110;
/** Backdrop layers raster at 2× their natural size for the same reason. */
export const BACKDROP_RASTER_SCALE = 2;

/** Structural row shapes so the pure indexer is `node --test`-able. */
export interface SvgAssetFields {
  widthPx: number;
  heightPx: number;
  contentHash: string;
  data: Uint8Array;
}
export interface CharacterArtRowLike extends SvgAssetFields {
  characterId: { toString(): string };
  kind: string;
}
export interface FaceRowLike extends SvgAssetFields {
  slug: string;
}
export interface BackdropRowLike {
  id: { toString(): string };
  slug: string;
  sky: SvgAssetFields;
  far: SvgAssetFields;
  mid: SvgAssetFields;
}

export type ContentTextureKind = 'body' | 'silhouette' | 'face' | 'backdrop';

/** One texture to rasterize from DB bytes (content-addressed key). */
export interface ContentTexture {
  key: string;
  svg: string;
  kind: ContentTextureKind;
  width: number;
  height: number;
}

export interface BackdropLayerArt {
  key: string;
  width: number;
  height: number;
}

export interface BackdropArt {
  id: string;
  slug: string;
  sky: BackdropLayerArt;
  far: BackdropLayerArt;
  mid: BackdropLayerArt;
}

export interface ContentIndex {
  bodyKeys: Map<string, string>;
  silhouetteKeys: Map<string, string>;
  faceKeys: Map<string, string>;
  backdropsById: Map<string, BackdropArt>;
  backdropsBySlug: Map<string, BackdropArt>;
  textures: ContentTexture[];
}

function decodeAsset(what: string, asset: SvgAssetFields): string {
  const hash = contentHash(asset.widthPx, asset.heightPx, asset.data);
  if (hash !== asset.contentHash) {
    throw new Error(`${what}: content hash mismatch`);
  }
  return new TextDecoder().decode(asset.data);
}

/** Raster size for one content texture, by its kind. */
export function rasterSize(texture: ContentTexture): {
  width: number;
  height: number;
} {
  switch (texture.kind) {
    case 'body':
      return { width: BODY_RASTER, height: BODY_RASTER };
    case 'silhouette':
      return { width: SILHOUETTE_RASTER, height: SILHOUETTE_RASTER };
    case 'face':
      return { width: FACE_RASTER_W, height: FACE_RASTER_H };
    case 'backdrop':
      return {
        width: texture.width * BACKDROP_RASTER_SCALE,
        height: texture.height * BACKDROP_RASTER_SCALE,
      };
  }
}

/**
 * Index the subscribed content rows: verify hashes, assign hash-keyed
 * texture entries, and build the lookup maps the scenes resolve art with.
 */
export function indexContent(
  art: CharacterArtRowLike[],
  faces: FaceRowLike[],
  backdrops: BackdropRowLike[],
): ContentIndex {
  const index: ContentIndex = {
    bodyKeys: new Map(),
    silhouetteKeys: new Map(),
    faceKeys: new Map(),
    backdropsById: new Map(),
    backdropsBySlug: new Map(),
    textures: [],
  };

  for (const row of art) {
    const characterId = row.characterId.toString();
    const what = `character art '${characterId}/${row.kind}'`;
    if (row.kind !== 'body' && row.kind !== 'silhouette') {
      throw new Error(`${what}: unknown kind`);
    }
    const svg = decodeAsset(what, row);
    const key = `roller_${row.contentHash}`;
    index.textures.push({
      key,
      svg,
      kind: row.kind,
      width: row.widthPx,
      height: row.heightPx,
    });
    const keys = row.kind === 'body' ? index.bodyKeys : index.silhouetteKeys;
    keys.set(characterId, key);
  }

  for (const row of faces) {
    const svg = decodeAsset(`face '${row.slug}'`, row);
    const key = `face_${row.contentHash}`;
    index.textures.push({
      key,
      svg,
      kind: 'face',
      width: row.widthPx,
      height: row.heightPx,
    });
    index.faceKeys.set(row.slug, key);
  }

  for (const row of backdrops) {
    const layer = (role: string, asset: SvgAssetFields): BackdropLayerArt => {
      const svg = decodeAsset(`backdrop '${row.slug}' ${role}`, asset);
      const key = `backdrop_${asset.contentHash}`;
      index.textures.push({
        key,
        svg,
        kind: 'backdrop',
        width: asset.widthPx,
        height: asset.heightPx,
      });
      return { key, width: asset.widthPx, height: asset.heightPx };
    };
    const backdrop: BackdropArt = {
      id: row.id.toString(),
      slug: row.slug,
      sky: layer('sky', row.sky),
      far: layer('far', row.far),
      mid: layer('mid', row.mid),
    };
    index.backdropsById.set(backdrop.id, backdrop);
    index.backdropsBySlug.set(backdrop.slug, backdrop);
  }

  return index;
}

// -- Runtime singleton, built once after the subscription applies ------------

let active: ContentIndex | null = null;

export function buildContentIndex(conn: DbConnection): ContentIndex {
  active = indexContent(
    [...conn.db.vw_character_art_v1.iter()],
    [...conn.db.vw_face_v1.iter()],
    [...conn.db.vw_backdrop_v1.iter()],
  );
  return active;
}

function content(): ContentIndex {
  if (!active) throw new Error('content index not built — connect first');
  return active;
}

export function characterBodyKey(characterId: string): string {
  const key = content().bodyKeys.get(characterId);
  if (!key) throw new Error(`character '${characterId}' has no body art`);
  return key;
}

export function characterSilhouetteKey(characterId: string): string {
  const key = content().silhouetteKeys.get(characterId);
  if (!key) {
    throw new Error(`character '${characterId}' has no silhouette art`);
  }
  return key;
}

export function faceKey(name: FaceName): string {
  const key = content().faceKeys.get(name);
  if (!key) throw new Error(`face '${name}' is not available`);
  return key;
}

export function backdropById(id: string): BackdropArt {
  const backdrop = content().backdropsById.get(id);
  if (!backdrop) throw new Error(`backdrop '${id}' is not available`);
  return backdrop;
}

export function backdropBySlug(slug: string): BackdropArt {
  const backdrop = content().backdropsBySlug.get(slug);
  if (!backdrop) throw new Error(`backdrop '${slug}' is not available`);
  return backdrop;
}
