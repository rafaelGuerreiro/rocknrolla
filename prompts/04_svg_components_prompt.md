# SVG Component System

Replace ASCII-grid levels with levels composed from reusable SVG components.
Components are authored as files, served from the database, and previewed in a
dev-only gallery page. See `CONTEXT.md` for the glossary (Component, Placement,
Gameplay plane, etc.).

## Decisions (grilled 2026-07-06)

| Question | Decision |
|---|---|
| What is a component? | One concept, any size: decor prop up to multi-screen map section |
| Composition point | Client-side, native SVG `<use>`/`<symbol>` |
| Game3play in components? | Yes — components embed collider markers (art + physics) |
| Level format | Levels become placement lists; ASCII grids retire |
| Source of truth | Files in repo; DB serves the game; import pushes files → DB |
| Editor page | Live gallery + preview only (edit files in IDE; HMR feedback) |
| Spawn/finish | Level-owned, never inside components |
| Placement transforms | Offset + flipX + uniform scale + z |
| Layers/parallax | No layers; flat placements, parallax derived from z |
| Term | "Component" (canonical everywhere) |
| Migration | Bootstrap starter library from svggen painters, rebuild the 3 levels, retire svggen ASCII path |

## File formats (source of truth)

### `levels/components/<slug>.svg`
Standalone SVG, slug = filename. `viewBox` defines component-local space,
`width`/`height` the natural pixel size. Colliders use the existing marker
contract: `<g visibility="hidden">` with `data-t="<tile id>"` rects/polygons,
component-local coordinates. Same 512KB cap as today.

### `levels/src/<slug>.json` (new shape)
```json
{
  "id": "…uuid…",
  "slug": "tutorial-hill",
  "name": "Tutorial Hill",
  "starting": true,
  "active": true,
  "reward_lootbox_id": "…",
  "successors": ["…"],
  "spawn": { "x": 512, "y": 256 },
  "finish": { "x": 3800, "y": 900 },
  "placements": [
    { "component": "hill-slope-l", "x": 0, "y": 640 },
    { "component": "pine-cluster", "x": 900, "y": 400, "z": -40, "flip_x": true, "scale": 1.5 }
  ]
}
```
`z` defaults 0 (gameplay plane), `flip_x` false, `scale` 1.

## Depth model

Positions are a `Vec3` SpacetimeType: `{ x: u16, y: u16, z: i8 }`.
**Unversioned by decision** — a geometric primitive that will never change
shape, so it skips the `V1` suffix rule (deliberate exception to
`stdb-schema-versioning`). x/y are world pixels (0–65535, no negative
offsets; world origin top-left). z is signed depth centered on the gameplay
plane: negative = background, positive = foreground. The old
`GAMEPLAY_Z = 127` constant in `client/src/levels.ts` retires with the
svg-v1 layer format.

- z = 0 → gameplay plane: colliders built, parallax 1:1.
- z ≠ 0 → scenery only, no bodies.
- Parallax derived: `parallax = 1 + z * PARALLAX_PER_Z` (clamped), constant in
  `client/src/tuning.ts` so it's console-tweakable like the rest of feel.
- Client groups placements by distinct z into one composed image each;
  placement order within a z group is draw order.

## DB schema (per `stdb-schema-versioning`; breaking, pre-live overwrite OK)

- New table `component_v1`: `Component { id, slug (unique), width_px, height_px,
  content_hash, data }` — same FNV-1a64 hash convention as level layers today.
- New types `Vec2 { x: u16, y: u16 }` and `Vec3 { x: u16, y: u16, z: i8 }` —
  shared SpacetimeTypes, unversioned (see Depth model). Conversions:
  `Vec2::from(Vec3)` (drops z) and `Vec3::from((Vec2, i8))` / `vec2.with_z(z)`.
- `level_v1`: replace layer linkage with `spawn: Vec2`, `finish: Vec2`.
- Drop `level_layer_v1`; add `level_placement_v1`:
  `LevelPlacement { id, level_id, component_id, position: Vec3,
  flip_x: bool, scale: f32, order }`.
- Views: `vw_component_v1` (`ComponentViewV1`), `vw_level_placement_v1`
  (`LevelPlacementViewV1`); update `LevelViewV1` with spawn/finish.
- Reducers: `import_component(ComponentImportV1)`;
  `import_level` takes new `PlacementImportV1` list — validates every placement
  references an existing component slug and spawn/finish are inside level
  bounds. Component validation stays substring-based (`<svg` wrapper, size cap).
- Admin `import all` gains components (import before levels, referential order).

## Client

- `db.ts`: subscribe `vw_component_v1` + `vw_level_placement_v1`.
- `levels.ts` rework: compose per-z SVG documents —
  `<svg><defs><symbol id="slug">…component art…</symbol></defs><use …/></svg>`,
  fully self-contained (Phaser SVG loading stays base64-data-URL, one texture
  per z group; texture key from hash of component hashes + placement list).
- Marker parsing: extract `data-t` markers from each gameplay-plane (z=0)
  component, apply placement transform (translate + flipX + scale) to marker
  coordinates → existing `levelBuilder.ts` body construction unchanged in
  vocabulary.
- flipX mirrors polygon points around the component's own width; scale is
  uniform on both art and colliders.

## Gallery page

- Second Vite entry, dev-only: `client/components.html` + small TS module.
- Loads `../levels/components/*.svg` via `import.meta.glob(…, { query: '?raw' })`
  (add `levels/` to `server.fs.allow`); Vite HMR = live reload on file save.
- Per component card: rendered SVG at true scale (zoom control), collider
  overlay toggle (parse `data-t` markers, draw outlines), light/dark/game-sky
  background switch, slug + dimensions + hash.
- Excluded from production build (dev-only entry).

## Bootstrap & migration

1. One-time admin command `export components <dir>` reusing `svggen.rs`
   painters to emit a starter library (flat ground run, slope L/R, water pool,
   fire pit, heavy block, decor clusters, background dirt bands).
2. Rebuild `tutorial-hill`, `riverside-run`, `stats-playground` as placement
   lists honoring the playability laws (spawn over a slope, water 1 deep on
   required paths).
3. Retire ASCII path: delete `levelsrc.rs` grid rendering, old layer import,
   `svg-v1` layer handling in client.

## Risks / guards

- **Scale × feel**: tuning constants are absolute; a scaled slope changes
  ramp length, not angle. Schema supports scale from day one, but starter
  levels use scale 1 on the gameplay plane until playtested.
- **Marker transform math**: flipX + scale on polygons is the one fiddly
  function — unit-test it (mirrored slope produces the mirrored polygon).
- **Composed size**: cap composed per-z document like today's 512KB layer cap.

## Implementation order

1. **Gallery page** — works off files alone, no schema change; immediate
   iteration value.
2. **Schema** — `component_v1`, placements, reducers, views; `task build`;
   fix bindings call sites; `task unsafe-overwrite`.
3. **Client composition** — per-z compose, marker transform, parallax-from-z.
4. **Bootstrap + rebuild levels** — starter library export, three levels as
   placements, retire ASCII/svggen path, reseed.
5. **Verify** — server lint/tests, client tests, playtest all three levels
   (CDP screenshot checks per the established pattern).

Each phase leaves the repo green (lint + tests) before the next.
