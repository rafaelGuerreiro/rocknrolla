import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import {
  BACKDROP_RASTER_SCALE,
  BODY_RASTER,
  FACE_RASTER_H,
  FACE_RASTER_W,
  indexContent,
  rasterSize,
  type BackdropRowLike,
  type CharacterArtRowLike,
  type FaceRowLike,
  type SvgAssetFields,
} from '../src/content.ts';
import { contentHash } from '../src/levels.ts';

const CHARACTER_ID = '570ede0a-1c7b-4d04-946d-446947e33fd5';
const BACKDROP_ID = '0195c8f1-0000-7000-8000-0000000000b1';

function asset(svg: string, width: number, height: number): SvgAssetFields {
  const data = new TextEncoder().encode(svg);
  return {
    widthPx: width,
    heightPx: height,
    contentHash: contentHash(width, height, data),
    data,
  };
}

function artRow(kind: string, svg: string): CharacterArtRowLike {
  return { characterId: CHARACTER_ID, kind, ...asset(svg, 120, 120) };
}

function faceRow(slug: string): FaceRowLike {
  return { slug, ...asset(`<svg>${slug}</svg>`, 80, 50) };
}

function backdropRow(slug: string): BackdropRowLike {
  return {
    id: BACKDROP_ID,
    slug,
    sky: asset('<svg>sky</svg>', 512, 512),
    far: asset('<svg>far</svg>', 512, 150),
    mid: asset('<svg>mid</svg>', 512, 110),
  };
}

test('indexes art, faces, and backdrops with hash-based keys', () => {
  const index = indexContent(
    [artRow('body', '<svg>body</svg>'), artRow('silhouette', '<svg>sil</svg>')],
    [faceRow('happy')],
    [backdropRow('dusk')],
  );

  const bodyKey = index.bodyKeys.get(CHARACTER_ID);
  const silhouetteKey = index.silhouetteKeys.get(CHARACTER_ID);
  assert.ok(bodyKey?.startsWith('roller_'));
  assert.ok(silhouetteKey?.startsWith('roller_'));
  assert.notEqual(bodyKey, silhouetteKey);
  assert.ok(index.faceKeys.get('happy')?.startsWith('face_'));

  const backdrop = index.backdropsBySlug.get('dusk');
  assert.ok(backdrop);
  assert.equal(index.backdropsById.get(BACKDROP_ID), backdrop);
  assert.equal(backdrop.far.height, 150);
  assert.equal(backdrop.mid.height, 110);
  assert.ok(backdrop.sky.key.startsWith('backdrop_'));

  // body + silhouette + face + three backdrop layers
  assert.equal(index.textures.length, 6);
  assert.equal(new Set(index.textures.map((t) => t.key)).size, 6);
  const decoded = index.textures.find((t) => t.key === bodyKey);
  assert.equal(decoded?.svg, '<svg>body</svg>');
});

test('rejects tampered bytes and unknown art kinds', () => {
  const tampered = artRow('body', '<svg>body</svg>');
  tampered.data = new TextEncoder().encode('<svg>evil</svg>');
  assert.throws(() => indexContent([tampered], [], []), /hash mismatch/);

  assert.throws(
    () => indexContent([artRow('hat', '<svg>hat</svg>')], [], []),
    /unknown kind/,
  );

  const badLayer = backdropRow('dusk');
  badLayer.mid = { ...badLayer.mid, contentHash: 'deadbeefdeadbeef' };
  assert.throws(() => indexContent([], [], [badLayer]), /dusk.*mid/);
});

test('raster sizes follow the texture kind', () => {
  const index = indexContent(
    [artRow('body', '<svg>body</svg>')],
    [faceRow('dizzy')],
    [backdropRow('dusk')],
  );
  const byKind = (kind: string) => index.textures.find((t) => t.kind === kind)!;
  assert.deepEqual(rasterSize(byKind('body')), {
    width: BODY_RASTER,
    height: BODY_RASTER,
  });
  assert.deepEqual(rasterSize(byKind('face')), {
    width: FACE_RASTER_W,
    height: FACE_RASTER_H,
  });
  const sky = index.textures.find((t) => t.key.startsWith('backdrop_'))!;
  assert.deepEqual(rasterSize(sky), {
    width: 512 * BACKDROP_RASTER_SCALE,
    height: 512 * BACKDROP_RASTER_SCALE,
  });
});
