import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import {
  awaitLevelPlacements,
  collisionRoot,
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

// Regression: vw_level_placement_v1 is gated by the server-side level
// selection, so the first play of a level found zero placements and failed;
// the game must wait for the rows to arrive instead.
function fakePlacementTable(rows: string[]) {
  type Listener = (ctx: unknown, row: { levelId: string }) => void;
  const listeners = new Set<Listener>();
  return {
    listeners,
    iter: () => rows.map((levelId) => ({ levelId })),
    onInsert: (cb: Listener) => listeners.add(cb),
    removeOnInsert: (cb: Listener) => listeners.delete(cb),
    insert(levelId: string) {
      for (const cb of [...listeners]) cb(undefined, { levelId });
    },
  };
}

test('awaitLevelPlacements resolves immediately when rows are present', async () => {
  const table = fakePlacementTable(['hill-1']);
  await awaitLevelPlacements(table, 'hill-1', 5);
  assert.equal(table.listeners.size, 0);
});

test('awaitLevelPlacements waits for the matching insert, ignoring others', async () => {
  const table = fakePlacementTable([]);
  const wait = awaitLevelPlacements(table, 'hill-2', 1000);
  table.insert('hill-1');
  assert.equal(table.listeners.size, 1);
  table.insert('hill-2');
  await wait;
  assert.equal(table.listeners.size, 0);
});

test('awaitLevelPlacements rejects when the rows never arrive', async () => {
  const table = fakePlacementTable([]);
  await assert.rejects(
    awaitLevelPlacements(table, 'hill-3', 5),
    /placements did not arrive/,
  );
  assert.equal(table.listeners.size, 0);
});

// Regression: concave character hulls decompose into compound Matter
// bodies, and collision pairs carry the parts — matching the player by
// identity missed the finish sensor (and ground contact) entirely.
test('collisionRoot resolves compound parts to their parent body', () => {
  interface FakeBody {
    parent: FakeBody;
  }
  const root = {} as FakeBody;
  root.parent = root; // Matter: top-level bodies are their own parent
  const part = { parent: root };

  assert.equal(collisionRoot(part), root);
  assert.equal(collisionRoot(root), root);

  const nested = { parent: part };
  assert.equal(collisionRoot(nested), root);
});
