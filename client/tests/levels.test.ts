import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import {
  planeParallax,
  transformMarker,
  type LevelMarker,
} from '../src/levels.ts';

const SLOPE_DOWN: LevelMarker = {
  t: 3,
  x: 0,
  y: 0,
  width: 64,
  height: 64,
  points: [
    { x: 0, y: 0 },
    { x: 64, y: 64 },
    { x: 0, y: 64 },
  ],
};

test('translates a rect marker into world space', () => {
  const marker: LevelMarker = { t: 1, x: 10, y: 20, width: 100, height: 50 };
  const world = transformMarker(marker, {
    x: 500,
    y: 300,
    flipX: false,
    scale: 1,
    componentWidth: 384,
  });
  assert.deepEqual(world, { t: 1, x: 510, y: 320, width: 100, height: 50 });
});

test('flipX mirrors a rect around the component width', () => {
  const marker: LevelMarker = { t: 1, x: 10, y: 20, width: 100, height: 50 };
  const world = transformMarker(marker, {
    x: 500,
    y: 300,
    flipX: true,
    scale: 1,
    componentWidth: 384,
  });
  // Mirrored: right edge (10 + 100 = 110 from left) becomes 384 - 110 = 274.
  assert.deepEqual(world, { t: 1, x: 774, y: 320, width: 100, height: 50 });
});

test('scale grows a rect uniformly from the placement origin', () => {
  const marker: LevelMarker = { t: 7, x: 10, y: 20, width: 100, height: 50 };
  const world = transformMarker(marker, {
    x: 500,
    y: 300,
    flipX: false,
    scale: 2,
    componentWidth: 384,
  });
  assert.deepEqual(world, { t: 7, x: 520, y: 340, width: 200, height: 100 });
});

test('flipX on a down slope produces the mirrored (up) polygon', () => {
  const world = transformMarker(SLOPE_DOWN, {
    x: 1000,
    y: 500,
    flipX: true,
    scale: 1,
    componentWidth: 64,
  });
  // The down slope's surface ran (0,0)→(64,64); mirrored it must rise
  // right-to-left: (64,0)→(0,64) in component space, offset to the placement.
  assert.deepEqual(world.points, [
    { x: 1064, y: 500 },
    { x: 1000, y: 564 },
    { x: 1064, y: 564 },
  ]);
  assert.deepEqual(
    { x: world.x, y: world.y, width: world.width, height: world.height },
    { x: 1000, y: 500, width: 64, height: 64 },
  );
});

test('flipX and scale compose on polygons', () => {
  const world = transformMarker(SLOPE_DOWN, {
    x: 1000,
    y: 500,
    flipX: true,
    scale: 1.5,
    componentWidth: 64,
  });
  assert.deepEqual(world.points, [
    { x: 1096, y: 500 },
    { x: 1000, y: 596 },
    { x: 1096, y: 596 },
  ]);
});

test('parallax derives from z and the gameplay plane scrolls 1:1', () => {
  assert.equal(planeParallax(0), 1);
  assert.ok(planeParallax(-120) < 1);
  assert.ok(planeParallax(40) > 1);
  // Clamped at the extremes so a deep background can never scroll backwards.
  assert.ok(planeParallax(-1000) >= 0.05);
});
