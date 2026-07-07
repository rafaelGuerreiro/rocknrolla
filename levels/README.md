# RocknRolla levels

Authored game content imported into SpacetimeDB by the interactive admin
shell (`server/bins/admin`). Git history of these files is the rollback
mechanism; imports overwrite the stored rows for a stable identity
(level UUID, component slug).

- `components/`: the component library — standalone SVG files, one
  component per file, slug = filename. Each carries its art plus hidden
  collider markers. Preview them in the dev gallery
  (`client/components.html`, served by `task client:dev` at
  `/components.html`).
- `src/`: authored level sources — compact JSON placement lists over the
  component library plus a level-owned spawn and finish.
- `seed.json`: characters, unique pieces, and lootbox drop tables.

## Commands

```sh
task server:admin   # interactive shell: validate/import content
```

Inside the shell: `validate all` dry-runs `components/`, `src/`, and
`seed.json`; `import all` imports into the configured database (default
`rocknrolladb-dev`) after confirmation, components before levels.
`export components <dir>` regenerates the starter library from the
built-in painters (one-time bootstrap; the committed files are the source
of truth). `help` lists everything.

## Component files (`components/<slug>.svg`)

A standalone SVG document. The root tag's `width`/`height` attributes are
the component's natural pixel size and `viewBox` defines component-local
space. Colliders are a hidden group (`<g visibility="hidden">`) of
rects/polygons tagged `data-t="<semantic tile id>"` in component-local
coordinates. 512KB cap per file.

Semantic tile ids: 1 solid, 2 slope up, 3 slope down, 6 lethal, 7 water,
8 fire, 9 heavy (marker-only; the client draws a dynamic sprite),
10 decor (no marker emitted).

## Level source shape (`src/*.json`)

```json
{
  "id": "<stable level UUID>",
  "slug": "tutorial-hill",
  "name": "Tutorial Hill",
  "starting": true,
  "active": true,
  "reward_lootbox_id": "<lootbox UUID, optional>",
  "successors": ["<level UUID>"],
  "spawn": { "x": 544, "y": 288 },
  "finish": { "x": 3800, "y": 736 },
  "placements": [
    { "component": "ground-flat", "x": 0, "y": 384 },
    { "component": "bush-cluster", "x": 900, "y": 400, "z": -40, "flip_x": true, "scale": 1.5 }
  ]
}
```

- `starting` defaults to `false`, `active` to `true`.
- Placement `z` defaults to `0`, `flip_x` to `false`, `scale` to `1`.
- Coordinates are world pixels (u16, origin top-left). `z` is signed
  depth: `0` is the gameplay plane (colliders built, scrolls 1:1),
  negative is background, positive is foreground. Parallax is derived
  from `z` (`PARALLAX_PER_Z` in `client/src/tuning.ts`).
- Placement list order is draw order within a depth.
- Spawn and finish are level-owned, never inside components. World bounds
  derive from the gameplay-plane extent; spawn/finish must land inside.

Playability laws: jumps are purely vertical — gravity on slopes is the
only propulsion, so spawn the roller over a slope and keep water pools
one cell deep on required paths.

## Client composition

The client composes one SVG document per distinct `z` (components as
`<symbol>`s, placements as `<use>`s), loads each as a texture, and builds
Matter bodies from the gameplay-plane markers after applying each
placement's transform (translate + flipX + uniform scale). Components
store an FNV-1a 64 content hash of `width_px_le ++ height_px_le ++
svg_bytes`, which the client re-verifies after loading.
