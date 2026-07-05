# RocknRolla levels

Authored level content imported into SpacetimeDB by the interactive admin
shell (`server/bins/admin`). Git history of these files is the rollback
mechanism; imports overwrite the stored rows for a stable level UUID.

- `tiled/`: editable Tiled sources (JSON maps, `catalog.tsj`, enemy tilesets,
  and the `Sprites/` art).
- `generated/`: exported Tiled JSON consumed by the importer.
- `seed.json`: characters, unique pieces, and lootbox drop tables.

## Commands

```sh
task server:admin   # interactive shell: validate/import levels and seed
```

Inside the shell: `validate all` dry-runs `generated/` and `seed.json`;
`import all` imports into the configured database (default
`rocknrolladb-dev`) after confirmation. `help` lists everything.

## Required map shape

- Orthogonal, finite, fixed-size map with `tilewidth`/`tileheight` of 64.
- Exactly one tileset with `firstgid: 1`; global tile id = catalog id + 1.
  Maps reference the external `catalog.tsj` collection tileset, whose local
  tile ids equal the semantic catalog ids below.
- Tile flip/rotation flags are rejected.
- Only tile layers are allowed.

## Map custom properties

| Property            | Type   | Meaning                                              |
| ------------------- | ------ | ---------------------------------------------------- |
| `id`                | string | Stable level UUID (import key). Required.            |
| `slug`              | string | Human-readable unique slug. Required.                |
| `name`              | string | Display name. Required.                              |
| `starting`          | bool   | Enabled for every new player. Default `false`.       |
| `active`            | bool   | Level is playable. Default `true`.                   |
| `reward_lootbox_id` | string | Lootbox UUID granted on first completion. Optional.  |
| `successors`        | string | Comma-separated level UUIDs enabled on completion.   |

## Layer custom properties

| Property                    | Type   | Meaning                                          |
| --------------------------- | ------ | ------------------------------------------------ |
| `z`                         | int    | Draw order 0..255. Required. `127` = gameplay.   |
| `role`                      | string | Optional `gameplay`/`visual` sanity check.       |
| `cell_width`, `cell_height` | int    | Visual-layer cell size override. Default 64.     |

Parallax comes from Tiled's native `parallaxx`/`parallaxy` layer fields.

Rules enforced on import: exactly one gameplay layer at `z = 127` with
parallax `(1.0, 1.0)` and 64px cells containing a spawn and a finish tile;
unique `z` per layer; layers below 127 render behind gameplay, above 127 in
front. `z` controls draw order only — parallax is never derived from it.

## Tile catalog (`0..=10`)

| ID  | Tile                                                  |
| --- | ----------------------------------------------------- |
| 0   | empty                                                 |
| 1   | solid terrain                                         |
| 2   | slope up (floor rises left→right)                     |
| 3   | slope down (floor falls left→right)                   |
| 4   | spawn                                                 |
| 5   | finish                                                |
| 6   | lethal hazard                                         |
| 7   | water sensor (buoyancy)                               |
| 8   | fire hazard (lethal below the resistance threshold)   |
| 9   | heavy pushable obstacle (needs a dense character)     |
| 10  | decorative, non-colliding                             |

## Storage encoding (`rle-v1`)

Each layer is stored as metadata plus compressed bytes: repeated
`[run_length: u8, tile_id: u8]` pairs in row-major order, runs longer than
255 split. Zero-length runs, unpaired bytes, and decoded lengths that do not
equal `width * height` are rejected. Every layer also stores an encoding
identifier (`rle-v1`) and an FNV-1a 64 content hash of
`width_le ++ height_le ++ decoded_tiles`, which the client re-verifies after
decoding.

## Assets

Art in `tiled/Sprites/` comes from Kenney's **New Platformer Pack**
(<https://kenney.nl/assets/new-platformer-pack>), licensed **CC0 1.0**
(public domain). `terrain_grass_ramp_short_b_mirror.png` is a horizontal
mirror of `terrain_grass_ramp_short_b.png` derived for the ascending slope
tile, since the pack ships only descending ramps.

- `catalog.tsj`: the semantic tile catalog used by every map (64px tiles;
  local tile ids equal the semantic ids).
- `enemies-default.tsj` / `enemies-double.tsj`: character/enemy sprite
  tilesets for authoring reference; they are never attached to maps.
