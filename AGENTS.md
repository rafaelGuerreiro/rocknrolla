# RocknRolla agent guide

## Product

RocknRolla is a prototype physics game. The player controls a circular body rolling downhill and presses the screen to avoid obstacles.

Optimize for a playable prototype, not a future platform. Apply YAGNI: prefer Phaser, browser, Capacitor, Rust, and SpacetimeDB built-ins; avoid speculative abstractions, services, configuration, and dependencies.

## Monorepo

- `client/`: Phaser TypeScript client using Phaser's bundled Matter physics and Capacitor for native packaging.
- `server/`: Rust workspace containing SpacetimeDB modules in `bins/*` and shared crates in `sdks/*`.

## Architecture boundaries

- The client is authoritative for movement and physics.
- Send milestone or waypoint events to the server; do not stream transforms.
- The server records rewards and progression. It does not simulate, replay, or validate physics.
- Trust client-reported gameplay outcomes for this prototype. Do not add anti-cheat until it is explicitly required.
- Still use `ctx.sender()` as the player identity. Never accept an identity argument as proof of ownership.
- Keep secrets, signing credentials, and generated build artifacts out of git.

## Working rules

- Read the affected component before changing it and keep changes inside that component unless an interface genuinely crosses boundaries.
- Implement the smallest end-to-end behavior that can be played or observed.
- Reuse existing scenes, scripts, tables, reducers, and build commands before adding new structure.
- Do not add a dependency, framework, manager, service layer, generic repository, event bus, or plugin for one use case.
- Avoid compatibility layers and migrations until persistent prototype data matters.
- Keep generated SpacetimeDB bindings generated; do not hand-edit them.
- Update this file when real build or test commands are added.

## Commands

- `task check`: check the server and build the client.
- `task test`: run workspace tests.
- `task build`: generate SpacetimeDB bindings and build server plus client.
- `task client:dev`: run the browser client.
- `task client:sync`: build and sync Capacitor platforms.
- `task server:publish`: publish the development database.
- `task server:admin`: interactive admin shell to validate and import authored content (components, characters, faces, backdrops, levels, seed) from `content/`.

## Verification

- Run the narrowest relevant check available in the component.
- For client changes, run `task client:fmt`, `task client:lint`, `task client:build`, and `task client:test` (Node's built-in test runner over `client/tests/`), and exercise the affected behavior in a browser.
- For Rust changes, run `task server:lint` (fmt + check + Clippy denying warnings, workspace-wide); use `task server:test` when behavior changes. A warning anywhere in the workspace fails this, even in code your change didn't touch — fix it rather than leaving it.
- For build changes, run the affected build/export command rather than inventing a parallel validation path.
- If a required tool or command is not present, report that clearly; do not claim verification.
