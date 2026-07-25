# RocknRolla content

Authored game content imported into SpacetimeDB by the interactive admin
shell (`server/bins/admin`). Git history of these files is the rollback
mechanism; imports overwrite the stored rows for a stable identity
(level UUID, component/backdrop/face slug, character style).

- `components/`: the component library — standalone SVG files, one
  component per file, slug = filename. Each carries its art plus hidden
  collider markers.
- `characters/`: character body art — one standalone SVG per character,
  filename = the character's `style` in `seed.json`. The locked-card
  silhouette is derived at import time (never authored), so every body
  must contain a `<defs>` block for the filter injection.
- `faces/`: the shared expression set (one SVG per expression name)
  layered upright on every character body.
- `backdrops/`: per-level scenery themes — three files per backdrop,
  `<slug>.sky.svg`, `<slug>.far.svg`, `<slug>.mid.svg`. The sky stretches
  to the screen; far/mid are horizontally tileable parallax strips.
- `levels/`: authored level sources — compact JSON placement lists over
  the component library plus a level-owned spawn, finish, and backdrop.
- `seed.json`: characters, unique pieces, and lootbox drop tables.

Preview all SVG content in the dev gallery (`client/components.html`,
served by `task client:dev` at `/components.html`).

## Commands

```sh
task server:admin   # interactive shell: validate/import content
```

Inside the shell: `validate all` dry-runs every content type;
`import all` imports into the configured database (default
`rocknrolladb-dev`) after confirmation, in dependency order:
seed → components → character art → faces → backdrops → levels.
`export components <dir>` regenerates the starter component library from
the built-in painters (one-time bootstrap; the committed files are the
source of truth). `help` lists everything.

## SVG file rules (all content types)

A standalone SVG document. The root tag's `width`/`height` attributes are
the natural pixel size and `viewBox` defines local space. 512KB cap per
file. Content is hashed (FNV-1a 64 over `width_le ++ height_le ++ bytes`)
and the client re-verifies the hash after loading.

Components additionally carry colliders: a hidden group
(`<g visibility="hidden">`) of rects/polygons tagged
`data-t="<semantic tile id>"` in component-local coordinates.
Semantic tile ids: 1 solid, 2 slope up, 3 slope down, 6 lethal, 7 water,
8 fire, 9 heavy (marker-only; the client draws a dynamic sprite),
10 decor (no marker emitted).

## Level source shape (`levels/*.json`)

```json
{
  "id": "<stable level UUID>",
  "slug": "tutorial-hill",
  "name": "Tutorial Hill",
  "backdrop": "dusk",
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

- `backdrop` is required and must name an authored backdrop slug — there
  is no default; a missing or unknown slug fails validation and import.
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

## Client rendering

Every texture the game draws (components, character bodies, silhouettes,
faces, backdrop layers) is rasterized from database bytes after the
subscription applies — the boot screen is a plain solid color until then.
Textures are keyed by content hash, so republished art invalidates
naturally. Each placement draws its own component texture (no composed
mega-textures); Matter bodies come from gameplay-plane markers after
applying each placement's transform (translate + flipX + uniform scale).
