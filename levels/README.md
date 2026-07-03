# RocknRolla levels

Authored level content imported into SpacetimeDB by the admin CLI
(`server/bins/rocknrolla-admin`). Git history of these files is the rollback
mechanism; imports overwrite the stored rows for a stable level ID.

- `tiled/`: editable Tiled sources (JSON maps plus the `tiles.png` catalog).
- `generated/`: exported Tiled JSON consumed by the importer.
- `seed.json`: characters, unique pieces, and lootbox drop tables.

## Commands

```sh
task server:levels-validate   # dry-run validation of generated/ and seed.json
task server:levels-import     # import into the dev database (default rocknrolladb-dev)
```

## Required map shape

- Orthogonal, finite, fixed-size map with `tilewidth`/`tileheight` of 32.
- Exactly one tileset with `firstgid: 1`; global tile id = catalog id + 1.
- Tile flip/rotation flags are rejected.
- Only tile layers are allowed.

## Map custom properties

| Property            | Type   | Meaning                                             |
| ------------------- | ------ | --------------------------------------------------- |
| `id`                | string | Stable level ID (import key). Required.             |
| `name`              | string | Display name. Required.                             |
| `starting`          | bool   | Enabled for every new player. Default `false`.      |
| `active`            | bool   | Level is playable. Default `true`.                  |
| `reward_lootbox_id` | string | Lootbox granted on first completion. Optional.      |
| `successors`        | string | Comma-separated level IDs enabled on completion.    |

## Layer custom properties

| Property                    | Type   | Meaning                                          |
| --------------------------- | ------ | ------------------------------------------------ |
| `z`                         | int    | Draw order 0..255. Required. `127` = gameplay.   |
| `role`                      | string | Optional `gameplay`/`visual` sanity check.       |
| `cell_width`, `cell_height` | int    | Visual-layer cell size override. Default 32.     |

Parallax comes from Tiled's native `parallaxx`/`parallaxy` layer fields.

Rules enforced on import: exactly one gameplay layer at `z = 127` with
parallax `(1.0, 1.0)` and 32px cells containing a spawn and a finish tile;
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
