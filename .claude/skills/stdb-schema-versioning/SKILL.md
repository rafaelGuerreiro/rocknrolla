---
name: stdb-schema-versioning
description: RocknRolla SpacetimeDB schema naming and versioning rules. Read before adding or renaming any table, view, or public-facing type in server/, and when planning a breaking schema change. Explains the _v1/V1 suffix convention and the pre-live overwrite flow.
---

# SpacetimeDB Schema Versioning

Every table, view, and public-facing type carries a version suffix so a v2 can
be added alongside v1 once the game is live and the schema must evolve without
breaking deployed clients.

## Naming rules

- Table accessors end in `_v1` AND carry an explicit matching `name`:
  `#[spacetimedb::table(accessor = player_v1, name = "player_v1", private)]`.
  Without `name`, SpacetimeDB derives the canonical table name by word-splitting
  the accessor and publishes `player_v_1` (it does not understand `v1` as one word).
- View accessors and functions follow the same rule:
  `#[view(accessor = vw_me_v1, name = "vw_me_v1", public)] pub fn vw_me_v1(...)`.
- Public-facing types (anything that reaches generated bindings: view row
  structs, reducer argument types) end in `V1`: `MyLootboxViewV1`, `LayerImportV1`.
- Private table row structs stay unversioned (`Player`, `LevelLayer`) — they
  never enter bindings. Only the accessor name is versioned.
- Views never return a table struct directly; map to a dedicated `*ViewV1`
  struct so the private table type never enters bindings (see `vw_me_v1`).
- Reducer names are not versioned.

## Why

Live SpacetimeDB modules cannot destructively change a published table or the
shape of a subscribed view without breaking connected clients. The versioned
names reserve the upgrade path: ship `player_v2` / `vw_me_v2` / `MeViewV2`
next to v1, migrate, then retire v1 once no client subscribes to it.

## Current flow (pre-live)

The game is not live, so schema changes just overwrite the dev database —
still under v1 names:

1. Change the module under `server/bins/rocknrolladb/`.
2. `task server:lint` and `task server:test`.
3. `task build` — regenerates `client/src/module_bindings/` (never hand-edit)
   and builds the WASM module; fix client call sites to match.
4. `task unsafe-overwrite` — republishes `rocknrolladb-dev`, wiping all data,
   then reseeds content (scripted `import all` through the admin CLI).

## Once live

- Never rename, reshape, or drop a `_v1` table/view that shipped: add the
  `_v2` twin, dual-write in reducers, backfill, move the client, then drop v1
  in a later release.
- The overwrite flow above becomes forbidden on the production database.
