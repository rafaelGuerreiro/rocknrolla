# Review and correct the RocknRolla vertical slice

Review the implementation produced from `prompts/initial_prompt.md`, then correct the problems described below. Work autonomously until the acceptance criteria pass. Read and follow the root, client, and server `AGENTS.md` files before changing code.

The requirements in this prompt supersede conflicting instructions in `prompts/initial_prompt.md`.

Preserve the intended product behavior and the architecture split in which the client owns gameplay physics and the server owns content, rewards, and progression. This is a prototype: make the smallest coherent changes that fix the identified problems, reuse existing dependencies and platform features, and do not introduce speculative infrastructure.

Use `/Users/rguerreiro/workspace/blok/server` as the reference for server organization and conventions. Adapt that structure to RocknRolla's domains rather than copying unrelated Blok behavior.

## 1. Reorganize the SpacetimeDB module by domain

The current `server/bins/rocknrolladb/src/lib.rs` combines table declarations, lifecycle behavior, player progression, lootbox logic, content import, validation, and reducers in one large file. Reducers directly read and mutate tables belonging to several different concepts. Replace this with domain modules that provide clear interfaces and keep their implementation local.

Organize `server/bins/rocknrolladb/src/` using the relevant parts of the Blok pattern:

```text
src/
  lib.rs
  error.rs
  extend/
  repository/
    mod.rs
    {domain}.rs
    {domain}/
      reducers.rs
      services.rs
      types.rs
      views.rs
```

Only create subfiles a domain actually needs. Choose domain seams based on ownership of behavior and data; do not mechanically create one module per table. `lib.rs` should be a small module entry point and lifecycle coordinator, not the home of application logic.

### Repository ownership

- Define each table in the repository module that owns it.
- Make every table private, including content/configuration tables and player-owned tables.
- Expose client-readable data through narrowly scoped public SpacetimeDB views.
- A repository may mutate only its own tables.
- Cross-repository mutations must call the owning repository's interface rather than accessing its tables directly.
- Cross-repository reads may be direct only when they remain simple and do not duplicate an invariant; otherwise expose the read through the owning repository's interface.
- Keep interfaces small and behavior-rich. Do not add traits, adapters, factories, or generic repositories when there is only one implementation.

### Reducers and domain logic

- Keep reducers dumb: validate request parameters, enforce caller/owner access, make one delegation to the appropriate repository service, and return that result.
- Every reducer returns `ServiceResult<()>`. Do not expose raw `Result<(), String>` or reducer-specific result aliases.
- Reducers must not contain business decisions, query or mutate tables directly, coordinate multi-table workflows, generate rewards, or duplicate repository invariants.
- Service operations are fully parameterized: pass the acting identity and every request value explicitly rather than reading them from ambient reducer state.
- The reducer obtains the authenticated actor with `ctx.sender()` and passes it to the service. No service method may call `ctx.sender()`, infer the actor from `ReducerContext`, or accept a client-provided identity as proof of ownership.
- Do not pass `ReducerContext` into a service method merely to obtain sender/request data. The same service operation must be callable by lifecycle hooks, scheduled work, or another repository when they supply an explicit acting identity.
- A repository implementation may retain only the SpacetimeDB database, timestamp, or RNG access required to perform its work; caller identity is never hidden inside those dependencies.
- Move progression, lootbox, character, and content-import invariants out of reducer bodies and into their owning repositories.
- Preserve transactional behavior for operations that span repositories, including first-completion rewards and lootbox opening.
- Preserve idempotency guarantees and existing gameplay-visible behavior.
- Test repository services and pure domain decisions through their interfaces. Do not add tests whose only purpose is exercising dumb reducer wrappers.

### Views and client subscriptions

- Replace public table exposure and `client_visibility_filter` use with public views over private tables.
- Views for player-owned state must derive ownership from the view context sender and must never expose another player's state.
- Content views should expose only the rows and fields required by the client.
- Regenerate TypeScript bindings and update client subscriptions and table/view accessors to consume the new public views.
- Do not hand-edit generated bindings.

### Shared checks and errors

- Add a small `extend` module for checks genuinely shared across repositories, following the Blok server's approach.
- Centralize repeated input validation, owner/internal access checks, and reusable SpacetimeDB/RNG helpers.
- Use a consistent typed service error/result model instead of scattered `Result<_, String>` construction.
- Validate reducer inputs at the server trust boundary, including required strings, numeric ranges, finite floating-point values, positive weights, referenced records, and malformed or duplicate import data where applicable.
- Do not copy unused Blok extensions or build a validation framework for one-off checks.

### Acceptance criteria

- `lib.rs` contains module wiring and lifecycle coordination rather than table definitions and domain workflows.
- Every SpacetimeDB table is private and all client-readable state is supplied by public views.
- Every reducer performs only parameter/caller validation plus one service delegation and returns `ServiceResult<()>`.
- No reducer reads or mutates tables directly or contains a multi-table business workflow.
- Every actor-sensitive service method receives the acting `Identity` explicitly and never reads `ctx.sender()`.
- Each table has one owning repository, and no other repository mutates it directly.
- Player-facing views cannot reveal another identity's state.
- Content import, first level completion, character selection, and lootbox opening retain their existing observable behavior.
- Invalid and unauthorized reducer calls return consistent errors without panicking.
- Generated TypeScript bindings and client subscriptions compile against the new views.
- Relevant Rust tests cover repository service invariants and caller-safe views or their underlying selection logic; reducer-wrapper tests are not required.

## 2. Replace the argument-driven admin CLI with an interactive shell

The host administration binary is currently named `rocknrolla-admin` and requires a command plus repeated flags and paths for every operation. Rename it to `admin` and make its primary interface a persistent interactive shell.

### Rename the binary completely

- Rename `server/bins/rocknrolla-admin/` to `server/bins/admin/`.
- Rename the Cargo package and produced binary to `admin`.
- Update the workspace lockfile, Taskfiles, documentation, comments, examples, and user-facing messages that refer to `rocknrolla-admin` or the old argument-driven tasks.
- Do not leave a compatibility binary, alias, forwarding package, or deprecated command behind.

### Interactive behavior

- Running `cargo run -p admin` or `task server:admin` must open the shell without requiring command-line arguments.
- Use a simple `std::io` read-evaluate-print loop. Do not add a CLI, terminal UI, line-editing, or command-dispatch dependency.
- Print a short prompt and keep the process alive after each successful command or recoverable error.
- Support `help`, `status`, `quit`, and `exit`; EOF must also exit cleanly.
- Unknown or malformed commands must show a concise error and usage hint without terminating the shell.
- Keep the current database, server, and default content paths as shell session state so users do not repeat them for every operation.
- Provide shell commands to display and change the target database and server.
- Provide commands to validate and import levels, validate and import seed content, and perform both operations together. Default these commands to the committed `levels/generated/` and `levels/seed.json` paths while allowing a path override when useful.
- Validation commands must never mutate SpacetimeDB.
- Before an import, display the destination server and database and require explicit interactive confirmation. Refusing or cancelling must perform no mutation.
- Continue using the locally installed `spacetime` executable and its authenticated owner session. Never read, embed, or print the `.env` token.
- Keep production protection explicit: the default database remains `rocknrolladb-dev`, and importing into the production database requires an additional unmistakable confirmation.

### Internal structure and checks

- Reuse the existing Tiled and seed parsing/validation implementation rather than duplicating it in command handlers.
- Separate line parsing from command execution enough to test commands without spawning an interactive terminal, but do not create a generic shell framework.
- Preserve actionable errors from filesystem validation and failed `spacetime call` processes.
- Remove the positional/flag parser and its usage text; command-line compatibility is not required.
- Replace `levels-validate` and `levels-import` Taskfile entry points with one documented `admin` task. Do not automate the interactive shell by piping scripted answers into it.

### Acceptance criteria

- `server/bins/admin` is the only administration package and its Cargo package/binary name is `admin`.
- Starting the binary with no arguments enters a persistent interactive prompt.
- The shell can validate and import the committed levels and seed content without restarting between operations.
- Target server, database, and paths have visible defaults and can be changed during the session.
- Import commands require confirmation and clearly identify their destination.
- Invalid input or a failed operation returns control to the prompt.
- Exit commands and EOF terminate successfully.
- Parser tests cover valid commands, malformed commands, state-changing configuration commands, and exit behavior.
- Repository documentation and Taskfiles describe only the new interactive workflow.

## 3. Use `Identity` only for authenticated users and UUIDs for entity IDs

The current model mixes three different identifier roles: SpacetimeDB `Identity` values for players, human-readable strings for content IDs and foreign keys, and auto-incrementing integers for relationship and player-state rows. Make the distinction explicit: `Identity` represents an authenticated principal, while database entities use SpacetimeDB's native `Uuid`.

Keep the complete SpacetimeDB toolchain on 2.6.1 and preserve `features = ["unstable"]`. `Uuid` and reducer UUID generation are available in SpacetimeDB 2.6.1, while `insert_or_update` and `try_insert_or_update` are gated by `unstable`. Do not invent `unsafe` or `experimental` Cargo features and do not add a separate UUID crate.

### Identifier rules

- Use `Identity` only for the signed-in player, an owning player, or another authenticated actor. Continue deriving the acting user from `ctx.sender()`.
- Never use `Identity` as a general-purpose entity identifier.
- Use `spacetimedb::Uuid` for entity primary keys and their corresponding foreign keys, including levels, level layers, successor edges, characters, pieces, lootboxes, lootbox drops, granted player lootboxes, and player progression/inventory relationship rows.
- Keep the player record keyed by `Identity`; it represents the authenticated principal rather than a separate entity ID.
- Do not use `Uuid` for counts, weights, timestamps, grid coordinates, ordering values, or other fields that are not identifiers.
- Keep readable content names or slugs in separate validated string fields where operators, source files, logs, sorting, or UI need them. A display slug is not a foreign key.
- Use the same UUID type for both sides of every relationship. Remove stringly typed and `u64` entity foreign keys.

### UUID creation and imported content

- Generate runtime entity IDs inside reducers with `ctx.new_uuid_v7()` and propagate generation failures through the repository's typed result.
- Do not accept a client-generated UUID for a new server-owned row when the server can generate it.
- Store stable, valid UUID strings in committed seed and Tiled content for authored entities. Parse them in the admin process and pass typed UUID reducer arguments; do not generate new IDs on every import.
- Update successor, reward-lootbox, character-piece, and lootbox-drop references in committed content to use those stable UUIDs.
- Validate malformed UUIDs and duplicate IDs/references before invoking import reducers.

### Upsert operations

- Replace manual `find` followed by `insert` or `update`, and manual delete-then-insert upserts, with the generated primary-key accessor's `insert_or_update` where replacement is the intended behavior.
- Use `try_insert_or_update` when a constraint violation is an expected input failure that must become a typed service error.
- Do not use upsert where creation must fail on duplicates, where an update must prove the row already exists, or where replacement would discard state that must be preserved.
- Perform all validation before an upsert so a failed replacement cannot leave ambiguous behavior.

### Client and generated data

- Regenerate the TypeScript bindings and use the SDK's generated UUID type throughout reducer arguments, scene state, view rows, and foreign-key comparisons.
- Compare and key UUID values using the SDK UUID's value semantics or canonical string form rather than JavaScript object identity.
- Do not display raw UUIDs as ordinary UI labels; use the entity's name or slug.
- Update level caching, sorting, texture keys, subscriptions, and scene payload types so UUID migration does not rely on the previous string IDs.

### Acceptance criteria

- Every persisted `Identity` field denotes an authenticated actor or owner.
- Entity primary keys and foreign keys consistently use `spacetimedb::Uuid`; unrelated numeric and value fields remain their appropriate types.
- No entity relationship relies on a human-readable string or auto-incrementing integer ID.
- Runtime UUIDs are generated by reducers, and authored UUIDs remain stable across repeated imports.
- Importing the same authored content updates the intended rows rather than creating duplicates.
- Appropriate upserts use `insert_or_update` or `try_insert_or_update`; operations with create-only or update-only semantics retain those guarantees.
- Client behavior, caches, comparisons, reducer calls, and subscriptions work with generated UUID bindings.
- Tests cover UUID parsing, stable re-import, relationship integrity, UUID value comparison/keying, and representative upsert semantics.

## 4. Use the copied Kenney platformer assets in Tiled and the game client

The Kenney New Platformer Pack has been copied into `levels/tiled/Sprites/`, but the editable maps still reference the placeholder `levels/tiled/tiles.png`, and the Phaser client still generates procedural terrain textures in `client/src/textures.ts`. Make the copied pack the actual visual source for authored levels and the running game.

This asset pack is an explicit exception to the original prompt's prohibition on downloaded art. The pack source is <https://kenney.nl/assets/new-platformer-pack> and is licensed CC0. Use the files already committed under `levels/tiled/Sprites/`; do not download the pack again or add another committed copy.

### One tile catalog for authoring and rendering

- Define a clear Tiled tileset/catalog that maps RocknRolla's stable semantic tile IDs to selected images from `Sprites/Tiles/Default/`.
- Use the pack's native ground and ramp images for solid terrain, slope-up, and slope-down tiles. Map the remaining current semantics to appropriate pack images where available, including a flag/exit, spikes, water, fire/lava, a weight, and decoration.
- Preserve semantic tile IDs in stored level data. Physics and gameplay behavior must depend on the semantic ID, not a PNG filename or Tiled global ID.
- Update every editable map under `levels/tiled/` to use the new catalog so authors see the intended art in Tiled.
- Regenerate the committed exports under `levels/generated/` and keep them valid for the admin importer.
- Remove the old placeholder `tiles.png` after it is no longer referenced.
- Keep exactly one canonical committed copy of each selected asset. Tiled and Vite must consume the same files rather than maintaining hand-copied authoring and runtime directories.
- Add a short asset-source/license note beside the level authoring documentation and remove copied OS metadata such as `.DS_Store`.

### Enemy sprite availability

- Copy the complete `/Users/rguerreiro/Downloads/kenney_new-platformer-pack-1.1/Sprites/Enemies/` tree into `levels/tiled/Sprites/Enemies/`, preserving both `Default/` and `Double/` filenames and relative paths.
- Copy all 60 Default and 60 Double PNGs, but exclude `.DS_Store` and other OS metadata.
- Never reference `/Users/rguerreiro/Downloads` at runtime or from committed source/configuration; it is only the source for this one-time repository import.
- Add clear Tiled collection-of-images tilesets for the enemy sprites so they can be browsed and selected while authoring. Keep Default and Double variants distinguishable, and use Default for the current 64-pixel logical cell scale.
- Do not attach unused enemy tilesets to exported maps in a way that changes semantic gameplay tile IDs or breaks the importer's single gameplay-catalog assumptions.
- Expose the canonical enemy tree to the Vite client through a small typed asset manifest or the shared asset-copy mechanism. The production and Capacitor bundles must contain the enemy sprites with predictable keys derived from their relative paths.
- Preload only sprites required by the active scene. Making the full enemy catalog available does not require decoding all 120 images at boot.
- Do not add enemy tables, AI, spawning, animation state machines, collisions, or gameplay semantics in this task. The frog player uses one supplied enemy sprite; the rest are available assets for later work.

### Client asset loading and rendering

- Make the selected files under `levels/tiled/Sprites/` part of the Vite asset graph or copy them through one minimal reproducible build step. The final `client/dist` and Capacitor web bundle must include them without relying on filesystem paths outside the built app.
- Preload required gameplay textures before entering scenes that use them. Show an actionable load failure rather than silently replacing missing files with procedural terrain.
- Replace procedural generation for catalog tiles with the Kenney PNG textures. Procedural character-ball and small particle textures may remain where the pack does not supply the intended RocknRolla visual.
- The tile displayed by Phaser must match the tile shown in Tiled for the same semantic ID. A Tiled-only visual update is insufficient.
- Use the pack's native `Default` 64-pixel tiles as 64-pixel logical gameplay cells. Update shared cell-size constants, authored maps, physics coordinates, sensors, camera bounds, and validation together; do not independently stretch art and collision geometry until they disagree.
- Make slope Matter vertices follow the visible ramp direction and silhouette closely enough that the ball rolls on the painted surface rather than floating above or cutting through it.
- Keep the existing layer Z and parallax behavior. Do not convert the level into a Phaser Tilemap or add a new rendering framework solely to display these images.

### Scope and acceptance criteria

- Ground and both slope directions visibly use the copied Kenney assets in Tiled and in the running `GameScene`.
- The remaining current tile semantics use coherent pack art where an appropriate image exists.
- All committed Tiled maps open with valid asset references and retain the stable semantic catalog mapping.
- Imported RLE data remains semantic tile IDs and is not coupled to asset paths.
- Rendered terrain, sensors, dynamic obstacles, and slope collision geometry align at runtime.
- Assets load in Vite development, the production web build, and the Capacitor-synced bundle.
- Both enemy sprite variants are available in Tiled and the client build from the same canonical repository files.
- No procedural placeholder ground or slope is rendered, no duplicate runtime asset tree is committed, and no stale `tiles.png` or `.DS_Store` references remain.
- The Kenney source URL and CC0 license are documented.

## 5. Split the shared level SDK into focused files

`server/sdks/rocknrolla-level/src/lib.rs` currently contains the tile catalog, gameplay constants, RLE codec and errors, content hashing, layer data, validation, test helpers, and all tests in one file. Preserve one cohesive crate interface while moving these responsibilities into smaller files with explicit names.

Use this structure unless the implementation reveals a materially clearer grouping:

```text
server/sdks/rocknrolla-level/src/
  lib.rs
  catalog.rs
  rle.rs
  hash.rs
  layer.rs
```

### Module responsibilities

- `catalog.rs`: semantic tile IDs and shared gameplay-layer constants such as cell size and gameplay Z.
- `rle.rs`: the `rle-v1` identifier, codec error type, encoder, decoder, and their focused tests.
- `hash.rs`: deterministic layer content hashing and its focused tests.
- `layer.rs`: `LayerFacts`, whole-layer-set validation, validation errors, and focused tests.
- `lib.rs`: crate-level documentation, module declarations, and only the intentional public interface; it must contain no codec or validation implementation.

Keep code that changes together in the same file. Do not create one file per constant or function, empty wrapper modules, generic utilities, traits with one implementation, or compatibility modules. Prefer qualified imports that make the owning module clear; use explicit re-exports only when they materially improve the crate's small external interface.

### Acceptance criteria

- `lib.rs` is a short map of the crate rather than the implementation of every responsibility.
- File names make the tile catalog, encoding, hashing, and layer validation easy to locate.
- Existing codec, hash, and validation behavior is preserved, including malformed-input errors and the updated gameplay cell size required by the visual catalog.
- Tests live beside the behavior they verify and retain the current edge-case coverage.
- The database module and renamed `admin` package use the reorganized crate through its intended public interface.
- `cargo test -p rocknrolla-level` passes without adding dependencies.

## 6. Lock the camera to a rolling Kenney character

The current game uses a generated 28-pixel ball with a hard-coded circular Matter body and a camera that follows with `0.08` interpolation and an arbitrary offset. Replace this with one round Kenney character whose collision polygon comes from its image alpha channel, and keep that character at a fixed screen-space anchor while it moves through the level.

### One playable character for now

- From the imported enemy tree, use `levels/tiled/Sprites/Enemies/Default/frog_idle.png` as the exact 64×64 single playable character asset for this prototype pass.
- Load it through the same canonical Vite/Capacitor asset path established for the other Kenney files.
- Replace the generated ball texture in gameplay; do not tint or redraw the Kenney character procedurally.
- Seed and expose one playable character for now. Keep the server model capable of multiple character definitions, but do not build a generalized character-sprite catalog, animation system, or fallback asset path until another playable character is required.
- Use a stable backend style/asset key that resolves to this sprite rather than treating `style` only as a CSS color.
- Update selection and collection UI so they do not present unavailable placeholder characters. If only one character is eligible, proceed with it directly instead of forcing a meaningless choice between one item.

### Alpha-derived Matter body

- After the texture is loaded, read its alpha channel through Phaser's texture APIs.
- Ignore transparent padding. Trace the outer opaque boundary using one named alpha threshold and simplify it to a stable, reasonably small polygon without erasing meaningful parts of the frog silhouette.
- Pass the contour to Phaser Matter's `fromVertices` support and use Phaser's already-bundled concave decomposition. Do not reduce the frog to a bounding box or unconditional convex hull when that would fill transparent concavities.
- Build the player Matter body from those vertices with Phaser's bundled Matter integration. Do not use `shape: "circle"`, a hand-tuned radius, an external contour library, standalone `matter-js`, or manually authored PhysicsEditor JSON.
- Center the derived vertices and the rendered image on the same origin so the collision hull remains aligned while rotating.
- Derive and cache the hull once per texture load, not once per frame or respawn.
- Fail with an actionable error if the texture is missing or has no usable opaque pixels; do not silently substitute a circle.
- Keep character density, jump, flight, buoyancy, and resistance data-driven. Alpha controls the collision silhouette, not those mechanics.

The frog has one approximately round outer silhouette, so a single alpha contour plus Phaser's bundled decomposition is sufficient for this pass. Do not add support for holes, disconnected opaque islands, multiple animation frames, or a general asset-processing pipeline until another character requires it.

### Camera anchor and rolling

- During normal gameplay, keep the player's body center at one-third of the viewport width from the left and at the viewport's vertical center.
- Follow the body center exactly with no lerp lag, deadzone, look-ahead, or velocity-based drift. The camera must move as the player rolls rather than allowing the player to travel across the viewport.
- Compute the follow offset from the current camera dimensions and refresh it on resize/orientation changes; do not hard-code a pixel offset for the 960×540 design size.
- Account for level-edge clamping so the player remains at the requested anchor from spawn through finish. Add only the camera padding required for this behavior; do not change the Matter world bounds merely to move the camera.
- HUD elements remain fixed to the viewport. Parallax layers continue using their authored scroll factors.
- Allow the Matter body to rotate naturally from terrain friction and collisions, and keep the sprite angle synchronized with the body. Do not fake rolling by rotating according to horizontal velocity or by running a tween.
- Do not lock rotation or overwrite the body's angle during jumps. Rotation and angular momentum must continue in the air and resume naturally on landing.
- Existing short camera-shake effects may temporarily offset the view, but normal follow must return immediately to the fixed anchor afterward.

### Acceptance criteria

- The Kenney `frog_idle.png` character is the only playable character shown in the current flow and no generated ball is visible.
- The player's Matter body is a polygon derived from the sprite's alpha data and excludes transparent image padding.
- The collision hull remains centered on the sprite at multiple angles and does not visibly float above or sink into flat terrain and slopes.
- Ground contact causes visible character rotation; reversing direction reverses angular motion, and airborne rotation is not reset.
- At rest, while rolling, while jumping, and after viewport resizing, the body center remains at `(viewport width / 3, viewport height / 2)` apart from explicit camera shake.
- Camera tracking has no visible easing lag or deadzone and does not break authored parallax or fixed HUD behavior.
- Refreshing, retrying, and restarting reuse the cached hull and do not leak duplicate textures or bodies.

## 7. End every failed run with defeat and level selection

The current gameplay death screen says `Wrecked! Tap to retry`, while completion-report failure offers a `Retry saving` button. Remove both retry workflows. Every failed run ends as a defeat and returns to level selection; only a server-confirmed completion reaches the success result.

- Treat lethal hazards, insufficient fire resistance, falling below the level, reducer rejection, and completion-confirmation timeout as defeat outcomes.
- Route every defeat through one small scene-owned outcome path so input, physics, timers, and subscription listeners are cleaned up consistently.
- Show a clear `Defeat` result with one direct tap/button action back to level selection.
- Do not restart the level automatically or on tap, and do not offer `Retry`, `Retry saving`, or an equivalent replay action from defeat.
- Keep completion reporting single-shot for the run.
- On reducer rejection or confirmation timeout, stop waiting, remove the completion-row listener, and cancel the timeout before showing defeat.
- Do not grant progress, successors, lootboxes, pieces, or a success result locally when confirmation is missing.
- Keep the normal confirmed-completion path unchanged.
- Ensure late callbacks from the failed attempt cannot transition the abandoned scene to the success result.

### Acceptance criteria

- A confirmed completion opens the normal success result.
- Lethal hazards, failed resistance checks, falling, rejected completion reports, and timed-out completion reports all show `Defeat` and return to level selection.
- Tapping after defeat cannot restart gameplay directly.
- No retry button, duplicate completion call, stale listener, or orphaned timeout remains after either outcome.

## 8. Respect mobile safe areas

The page opts into edge-to-edge rendering with `viewport-fit=cover`, and Capacitor iOS uses `contentInset: "never"`, but the client never applies `safe-area-inset-*`. Keep the game background edge-to-edge while placing the Phaser canvas and all interactive UI inside the device's usable safe rectangle.

- Use CSS `env(safe-area-inset-top)`, `env(safe-area-inset-right)`, `env(safe-area-inset-bottom)`, and `env(safe-area-inset-left)` with zero fallbacks. Do not hard-code dimensions for specific devices.
- Size and center the Phaser host/canvas within the safe content box while allowing the page background to fill cutout regions.
- Keep Phaser scaling, pointer coordinates, camera viewport calculations, and the one-third player anchor correct when the safe rectangle changes.
- Recalculate layout after resize and orientation/safe-area changes.
- Ensure fixed HUD controls, titles, back buttons, reward controls, and defeat/success actions remain inside the safe rectangle with comfortable touch margins.
- Preserve ordinary browser behavior where all inset values are zero.
- Do not add a safe-area plugin when CSS environment insets and the existing Capacitor WebView are sufficient.

### Acceptance criteria

- No text or interactive control overlaps a notch, rounded corner, status area, home indicator, or system navigation inset.
- The full 16:9 game remains visible and centered within the remaining safe rectangle without distorted scaling.
- Pointer/touch hit targets remain aligned with rendered buttons after safe-area padding and resizing.
- Desktop and non-notched devices render as before with zero insets.
- Safe-area behavior is verified in responsive browser emulation and at least one available iOS or Android Capacitor simulator/device.

## 9. Lock native gameplay to sensor landscape

The native projects currently allow portrait orientation even though RocknRolla is a landscape game. Configure Capacitor's generated native applications for sensor-driven landscape while allowing the device to rotate between both landscape directions.

- Set the Android activity orientation to `sensorLandscape` in the native manifest.
- On iOS and iPadOS, support `UIInterfaceOrientationLandscapeLeft` and `UIInterfaceOrientationLandscapeRight` only; remove portrait and portrait-upside-down from the supported application orientations.
- Allow 180-degree rotation between the two landscape orientations when the physical device rotates.
- Keep ordinary browser builds responsive; do not call the browser Screen Orientation API or block browser portrait rendering.
- Recalculate Phaser scaling, safe-area insets, camera dimensions, and the one-third player anchor after rotating between landscape directions.
- Use native manifest/plist configuration. Do not add a Capacitor orientation plugin for static application orientation support.

### Acceptance criteria

- Android launches and remains in sensor landscape and rotates between landscape-left and landscape-right.
- iPhone and iPad builds expose only the two landscape orientations and rotate between them.
- Rotation does not misplace the canvas, HUD, pointer targets, safe-area padding, camera anchor, or parallax layers.
- Web development and desktop browser resizing remain unrestricted.

## 10. Split `GameScene` into focused gameplay files

`client/src/scenes/GameScene.ts` is already more than 500 lines and currently owns level rendering, Matter terrain construction, player creation, movement/input state, collisions, camera behavior, run outcomes, HUD, and effects. The camera, Kenney assets, alpha-contour collision, and defeat changes would make it harder to navigate. Keep the Phaser scene as the lifecycle coordinator and move cohesive gameplay implementations behind small interfaces.

Use this structure unless implementation evidence supports a clearer naming/grouping:

```text
client/src/
  gameplay/
    levelBuilder.ts
    playerBody.ts
    playerController.ts
    cameraFollow.ts
    runOutcome.ts
  scenes/
    GameScene.ts
```

### Module responsibilities

- `levelBuilder.ts`: render decoded layers and build terrain, slopes, sensors, water regions, heavy bodies, and spawn facts from semantic tiles.
- `playerBody.ts`: load/use the selected character texture, derive/cache its alpha contour, construct the Matter body, and keep sprite/body origins aligned.
- `playerController.ts`: own jump buffering, coyote time, double jump, held lift, grounded transitions, buoyancy, and per-run controller state.
- `cameraFollow.ts`: own the one-third/vertical-center anchor, camera padding, resize/orientation updates, and follow cleanup.
- `runOutcome.ts`: own mutually exclusive success/defeat transitions plus completion listener/timeout registration and cleanup.
- `GameScene.ts`: load the selected level/character, compose these modules, connect collision/input events, build the small HUD, and forward Phaser lifecycle calls.

Keep tiny effects beside their owner; create an `effects.ts` only if jump, landing, shake, and outcome effects form enough cohesive code to justify it.

### Design rules

- Use direct functions or small stateful objects where state is real. Do not add managers, factories, an event bus, dependency injection, a plugin, or interfaces with one implementation.
- Give each module a small behavior-rich interface. Do not merely move every existing method into a one-to-one wrapper file.
- Keep Phaser scene/world dependencies explicit in constructors or function parameters; do not use global scene state.
- Keep tuning constants with the module that owns the behavior.
- Avoid circular imports. Shared gameplay types should live with their owner or in one small `types.ts` only when multiple modules genuinely need them.
- Register and clean up Phaser input, resize, Matter collision, timer, and SpacetimeDB listeners within the module that owns them.
- Preserve client authority for physics and the existing server waypoint/completion interface.

### Acceptance criteria

- `GameScene.ts` reads as run orchestration rather than low-level rendering, contour, movement, camera, and async-outcome implementation.
- Each gameplay responsibility has one obvious file and no new file becomes a miscellaneous dumping ground.
- Scene restart/shutdown disposes every listener and timer owned by the composed gameplay modules.
- Gameplay behavior, tuning, camera anchor, alpha collision, parallax, success, and defeat remain observable end to end.
- `task client:build` passes without adding a framework or dependency.

## Verification

Run the narrowest checks while working, then complete the repository-prescribed verification:

1. format the Rust workspace;
2. confirm the Rust SDK, TypeScript SDK, and selected `spacetime` CLI are all 2.6.1;
3. run `task server:check`;
4. run `task server:test`;
5. launch `task server:admin` and exercise help, status, validation, cancellation, error recovery, and clean exit;
6. validate every committed Tiled export, confirm the editable maps resolve the Kenney terrain tileset, and confirm both enemy collections expose all 120 PNGs without changing imported semantic IDs;
7. regenerate TypeScript bindings with the 2.6.1 CLI;
8. run `task client:build` and confirm the Kenney terrain and enemy assets are addressable in the output through their canonical keys;
9. run root `task build`;
10. exercise solid ground, both slope directions, hazards, water, the heavy obstacle, and the finish in a browser;
11. enable Matter debug rendering while developing and verify the alpha-derived hull stays aligned through a full rotation, then disable debug rendering for the delivered build;
12. verify the player stays at the one-third/vertical-center camera anchor while stationary, rolling, jumping, landing, near both level edges, and after resizing the browser;
13. exercise every defeat source plus confirmed, rejected, and timed-out completion reporting; verify only the confirmed path reaches success and every failure returns to level selection;
14. restart and leave gameplay repeatedly while checking that input, collision, resize, database, and timer callbacks fire only once;
15. emulate nonzero safe-area insets and verify layout, camera anchoring, and pointer alignment at multiple landscape aspect ratios;
16. run `task client:sync`, verify native orientation declarations, and rotate an available simulator/device between both landscape directions;
17. confirm the assets and safe-area behavior are included in the Capacitor bundle when the native toolchain is available;
18. publish/reset only the local development database when needed and exercise the affected browser flow against it when available.

Report any unavailable service or tool precisely. Do not claim an integration step passed when it could not be run.
