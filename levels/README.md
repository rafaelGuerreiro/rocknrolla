# RocknRolla levels

Authored level content imported into SpacetimeDB by the interactive admin
shell (`server/bins/admin`). Git history of these files is the rollback
mechanism; imports overwrite the stored rows for a stable level UUID.

- `src/`: authored level sources — compact JSON documents whose layers are
  ASCII tile grids. The importer renders each layer into a standalone
  `svg-v1` scene SVG (art + collider markers) and stores those bytes.
- `seed.json`: characters, unique pieces, and lootbox drop tables.

## Commands

```sh
task server:admin   # interactive shell: validate/import levels and seed
```

Inside the shell: `validate all` dry-runs `src/` and `seed.json`;
`import all` imports into the configured database (default
`rocknrolladb-dev`) after confirmation. `help` lists everything.

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
  "layers": [
    { "z": 20, "cell": 128, "parallax_x": 0.4, "rows": ["...d..."] },
    { "z": 127, "rows": ["S....F", "######"] }
  ]
}
```

- `starting` defaults to `false`, `active` to `true`.
- `cell` is the tile size in pixels (default 64; the gameplay layer must
  use 64). `parallax_x` / `parallax_y` default to 1.0.
- Every row in a layer must have the same length.

## Tile characters

| Char | Tile                                                  |
| ---- | ----------------------------------------------------- |
| `.`  | empty                                                 |
| `#`  | solid terrain                                         |
| `/`  | slope up (floor rises left→right)                     |
| `\`  | slope down (JSON-escaped as `\\`)                     |
| `S`  | spawn                                                 |
| `F`  | finish                                                |
| `^`  | lethal hazard                                         |
| `~`  | water sensor (buoyancy)                               |
| `f`  | fire hazard (lethal below the resistance threshold)   |
| `H`  | heavy pushable obstacle (needs a dense character)     |
| `d`  | decorative, non-colliding                             |

Rules enforced on import: exactly one gameplay layer at `z = 127` with
parallax `(1.0, 1.0)` and 64px cells containing a spawn and a finish;
unique `z` per layer; layers below 127 render behind gameplay, above 127
in front. `z` controls draw order only.

## Storage encoding (`svg-v1`)

Each layer row stores pixel dimensions, parallax, and one standalone SVG
document ("Claymation Dusk" scene art rendered by the importer's
generator, `server/bins/admin/src/svggen.rs`). The gameplay layer's SVG
additionally carries a hidden group of collider markers — rects/polygons
tagged `data-t="<semantic tile id>"` — from which the client builds its
Matter bodies (terrain, slopes, sensors, spawn/finish, heavy blocks).
Heavy blocks are marker-only; the client draws them as dynamic sprites.

Every layer stores the encoding identifier (`svg-v1`) and an FNV-1a 64
content hash of `width_px_le ++ height_px_le ++ svg_bytes`, which the
client re-verifies after loading.
