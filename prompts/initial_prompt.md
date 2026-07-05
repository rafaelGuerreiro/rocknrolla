# Implement the RocknRolla vertical slice

Implement a complete, playable RocknRolla prototype in this repository. Work autonomously until the acceptance criteria pass. Read and follow the root, client, and server `AGENTS.md` files before changing code. Do not stop for clarification; where a minor detail is unspecified, choose the smallest implementation consistent with this prompt.

## Product goal

RocknRolla is a landscape mobile physics game. The player chooses a circular character, rolls downhill through an authored level, and uses one-touch jumps to avoid hazards and reach the finish. Levels and character configuration come from SpacetimeDB so content can change without an app update.

Deliver one polished vertical slice: level selection, character selection, one playable imported level, completion/progression, lootbox opening, and character-piece collection.

## Non-negotiable architecture

- `client/` owns all rendering, input, Matter physics, collision behavior, and moment-to-moment gameplay.
- `server/` owns level data, enabled-level progression, level completion, lootboxes, character definitions, character pieces, inventory, and selected character.
- The server trusts client-reported level completion. Do not simulate or replay physics server-side and do not add anti-cheat.
- Still enforce ownership with `ctx.sender()`. Never accept a player identity as authority from reducer arguments.
- Use SpacetimeDB tables, subscriptions, and reducers. Do not add REST, GraphQL, another database, a cache, or a separate service.
- Use Phaser's bundled Matter integration. Do not install standalone `matter-js`.
- Keep the implementation direct. Do not add a UI framework, state framework, event bus, service layer, repository abstraction, or plugin system.
- Keep native gameplay in the shared web client. Do not duplicate it in Swift or Java.

## Game structure

- Levels are authored, not endless or procedurally generated.
- A successful run starts at the authored spawn and ends at the finish tile.
- Touching a lethal hazard or falling below the level fails the run.
- There are no checkpoints, lives, stamina, or energy. Failure restarts the whole level immediately from the already-decoded in-memory data.
- Non-lethal obstacles slow, redirect, block, or challenge the player rather than ending the run.
- Level duration is unconstrained and determined entirely by level design.
- The player chooses one unlocked character before starting and cannot switch during a run.

## Controls and physics feel

- Horizontal motion comes from gravity, slopes, momentum, and collisions. There is no horizontal steering.
- Pressing while grounded starts a jump with an immediate upward impulse.
- Holding continues applying limited lift. Longer holds produce longer flight, capped by the selected character's `flight_time` stat.
- Releasing cuts the remaining lift for responsive variable-height jumps.
- Allow one double jump. The second press in the air applies another impulse and may also be held for its capped lift period.
- Landing resets the double jump.
- Add a small coyote window and input buffer, roughly 100–120 ms, so touch controls feel forgiving.
- Use one pointer path for touch and mouse.
- Avoid tunneling and unstable bodies at expected mobile frame rates. Use fixed/consistent Matter settings where Phaser supports them.

## Character mechanics

Character definitions are backend data and include at least:

- stable ID and display name;
- visual color/style key;
- rarity weight;
- density;
- jump speed/impulse;
- maximum held-flight time;
- buoyancy;
- fire resistance;
- whether the character is initially unlocked.

These stats must affect gameplay:

- density affects the Matter body and the ability to move or pass heavy obstacles;
- jump speed controls jump impulse;
- flight time controls held lift duration;
- buoyancy controls upward force while inside water;
- fire resistance is compared with a fire hazard's required resistance.

Provide at least a baseline stone character and a contrasting paper-ball character. Stone is dense, jumps lower, and sinks. Paper is light, flies longer, and floats, but cannot move sufficiently heavy obstacles and has low fire resistance. Keep all mechanics data-driven through whitelisted client behavior; adding a new stat value must not require new code, while adding a new behavior type may require an app update.

## Look and feel

- Use a clean, stylized geometric/paper-cut look suitable for temporary procedural art.
- Use a dark twilight background, layered cool-color silhouettes, and warm/high-contrast playable objects.
- Make each circular character visibly rotate so rolling is readable.
- Assemble angled terrain sprites tightly so the downhill surface appears continuous.
- Use restrained game feel: dust on hard landings, a small jump effect, subtle camera lag, and light camera shake on heavy collisions or death.
- Keep the HUD minimal: current level, selected character, restart, and pause only.
- Use readable large touch targets and respect mobile safe areas.
- Keep landscape 16:9 behavior using Phaser scaling; it must remain usable in a resizable browser and Capacitor WebView.
- Do not add downloaded art, audio, a design system, or elaborate menus. Procedural Phaser graphics are sufficient for this slice.

## Tiled level source and storage

Use Tiled JSON as the importer input. Commit both editable Tiled source and exported JSON under a clear repository directory such as:

```text
levels/
  tiled/
  generated/
```

Document the required Tiled properties next to those files. Include one small valid tutorial level that demonstrates spawn, downhill terrain, jumping, a non-lethal obstacle, water or fire, and a finish.

### Grid and layers

- Every level is a finite fixed 2D tile grid.
- A level may contain multiple tile layers.
- Exactly one layer is the gameplay/collision layer.
- The gameplay layer always has `z = 127`, parallax `(1.0, 1.0)`, and a fixed logical cell size.
- Every other layer is visual-only.
- Visual layers may have independent cell width, cell height, `z: u8`, and explicit horizontal/vertical parallax factors.
- `z` controls draw order only. Do not derive parallax speed from `z`.
- Layers below 127 render behind gameplay; layers above 127 render in front.
- Reject imports with no gameplay layer, multiple gameplay layers, duplicate layer Z values, or a gameplay layer not at 127.

Use Tiled custom properties for values not represented natively. At minimum support stable level ID, display name, starting-level flag, reward lootbox ID, layer role, Z, parallax factors, and optional per-layer cell size overrides.

### Tile encoding

- Use a small client-known tile catalog with IDs `0..=255`; `0` means empty.
- Treat angled slopes as distinct building-block tile IDs for this first encoding.
- Reject unsupported Tiled flip/rotation flags rather than silently corrupting them.
- Store each imported layer as metadata plus a compressed `Vec<u8>`.
- Implement and document `rle-v1`: repeated pairs of `[run_length: u8, tile_id: u8]` in row-major order. Split runs longer than 255. Reject zero-length runs, malformed byte counts, and decoded lengths that do not equal `width * height`.
- Store an encoding identifier and a content hash. Imports overwrite the current rows for a stable level ID; do not create database versions. Git history of the committed Tiled/JSON files is the rollback mechanism.
- Add focused round-trip and malformed-input tests for the encoder/decoder.

The initial client tile catalog must cover:

- empty;
- solid terrain;
- upward and downward slopes;
- spawn;
- finish;
- lethal hazard;
- water sensor;
- fire hazard with a resistance threshold;
- heavy obstacle with a density threshold;
- non-colliding decorative tiles.

Merge adjacent compatible solid tiles into larger static Matter bodies where straightforward, but do not build a general geometry optimizer.

## Level import admin CLI

Create a Rust host binary in `server/bins/`, named consistently with the workspace, for level administration. It must not be part of the WASM module.

The CLI must:

1. accept a Tiled JSON file or directory;
2. parse and validate the map, custom properties, tile IDs, layers, successor IDs, and RLE round trip;
3. compute the content hash;
4. support a `--dry-run` mode that performs all validation without touching SpacetimeDB;
5. call owner-only SpacetimeDB import/update reducers using the locally installed `spacetime` CLI and its authenticated owner session;
6. overwrite the level's metadata, layers, and successor edges atomically;
7. never embed or print the `.env` token.

Use `server/.env` and the existing login task for owner authentication. Verify current `spacetime call --help` syntax instead of guessing it. Use `std::env` for the small CLI argument surface; do not add a CLI framework unless unavoidable. Add only the minimum JSON/error dependencies needed.

Add Taskfile commands for dry-run validation and import. The database name must be configurable, defaulting to the local/development RocknRolla database rather than production. Never publish or overwrite Maincloud while implementing this task.

## Server data model and behavior

Use clear SpacetimeDB tables and current 2.6 APIs. Keep configuration/content tables readable by clients. Keep player-owned state private or expose caller-safe views where supported by the current API.

Model at least these concepts; exact Rust names may follow repository conventions:

- level metadata;
- level layers containing compressed bytes;
- directed level-successor edges;
- player enabled levels;
- player completed levels;
- character definitions and mechanical stats;
- unique character-piece definitions;
- lootbox definitions and weighted piece drops;
- player unopened lootboxes;
- player piece counts, including duplicates;
- player unlocked characters;
- player selected character.

### Progression

- Explicitly mark one or more levels as starting levels.
- On first player initialization, enable all active starting levels and the starter character.
- A player may only start/select an enabled level and unlocked character.
- Completing an enabled level is idempotent.
- On first completion, record completion, insert every configured successor into the player's enabled-level table, and grant one configured unopened lootbox in the same transaction.
- Replaying an already completed level grants no additional completion lootbox.
- Keep successor handling simple: insert configured targets if absent. Do not implement graph versions, relocking, prerequisite expressions, or a full campaign DAG validator.

### Lootboxes and pieces

- A lootbox is granted unopened and can be opened later.
- Opening must happen in a server reducer.
- The reducer uses `ctx.rng()` and backend-configured weights to choose a unique piece definition.
- Pieces are unique definitions, but duplicate drops are allowed.
- Store a count per player and piece so duplicates remain available for a future upgrade system.
- Character rarity influences the drop weight of its pieces.
- A character unlocks when the player owns at least one of every unique piece assigned to it.
- Keep unlocked characters eligible for duplicate drops.
- Do not implement upgrades or duplicate conversion yet.
- Persist the awarded piece before the client reveal animation. The client animates the server-decided result and never chooses the reward.

Add reducer tests for idempotent completion rewards, successor unlocking, duplicate piece counts, character unlocking, unauthorized ownership attempts, and lootbox consumption.

## Client flow

Replace the single-scene spike with the minimum clear Phaser scene flow:

1. boot/connect/loading;
2. enabled-level selection;
3. unlocked-character selection;
4. gameplay;
5. completion/failure result;
6. unopened-lootbox and piece-collection view.

Requirements:

- Connect using the installed `spacetimedb` TypeScript SDK and generated bindings.
- Configure URI and database through public Vite environment variables with sensible local defaults. Never put owner credentials in the client.
- Persist only the normal client identity/session token using a browser-safe mechanism that also works in Capacitor.
- Subscribe only to content and caller-owned state required by the current screens.
- Load the selected level's metadata/layers, validate the encoding/content hash, decode it, and build the visual layers plus gameplay bodies.
- Cache the decoded selected level in memory for instant retries. An overwrite is observed on the next level load; no historical version support is needed.
- Use the selected backend character stats to configure the player body and jump controller before the run starts.
- Report completion once when the finish is reached. Disable duplicate finish handling while waiting for the server update.
- Show useful loading and recoverable error states instead of silently falling back to fabricated progress.
- The failure screen may be extremely brief; tapping restarts the full level without a server request.
- Keep all menus functional with touch and mouse.

Do not build offline progression, account UI, social features, analytics, ads, purchases, cloud saves outside SpacetimeDB, localization, accessibility narration, or production asset pipelines.

## Seed content

Provide enough owner-imported seed content to exercise the complete flow:

- one starting tutorial level and at least one successor level reference;
- one starter stone character;
- one paper-ball character assembled from multiple unique pieces;
- at least one lootbox definition whose weighted drops can produce those pieces;
- tuned stats that visibly differentiate stone and paper;
- a sample level where character density/buoyancy/fire resistance can be observed, without requiring every character to finish the tutorial.

Keep seed data in committed source files consumed by the admin CLI. Do not hide seed content in reducer code.

## Build and verification

Preserve and extend the existing Taskfile workflow. Before finishing:

1. run formatting and the relevant Rust tests;
2. run admin CLI dry-run validation against every committed generated Tiled JSON file;
3. regenerate TypeScript bindings;
4. run `task server:build`;
5. run `task client:build`;
6. run root `task build`;
7. run `npx cap sync` if client dependencies or Capacitor configuration changed;
8. exercise the browser flow against a local SpacetimeDB instance if available.

Do not claim a local database interaction passed if the service was unavailable. Report the exact unverified step, while ensuring all offline parser, reducer, TypeScript, and build checks pass.

## Completion criteria

The task is complete when:

- committed Tiled JSON validates and imports through the owner CLI;
- imported multi-layer levels round-trip through RLE and render with explicit parallax and Z ordering;
- the main `z = 127` layer alone creates gameplay physics;
- stone can roll, variable-jump, and double-jump through the tutorial;
- character stats visibly change physics and environmental interactions;
- failure restarts the full level instantly with no checkpoint;
- first completion unlocks configured successor levels and grants exactly one unopened lootbox;
- opening a lootbox records a weighted server-selected unique piece definition, including duplicate counts;
- collecting every required unique piece unlocks its character;
- the player can select an unlocked character only before a run;
- generated bindings are current and all required builds/tests pass;
- no secrets, local SDK paths, build output, generated web assets, keystores, or tokens are committed.
