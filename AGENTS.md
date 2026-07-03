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
- `task server:levels-validate`: dry-run validate committed levels and seed content.
- `task server:levels-import`: import seed content and levels (vars `LEVELS_DB`, `LEVELS_SERVER`).

## Verification

- Run the narrowest relevant check available in the component.
- For client changes, run `task client:build` and exercise the affected behavior in a browser.
- For Rust changes, run `task server:check`; use `task server:test` when behavior changes.
- For build changes, run the affected build/export command rather than inventing a parallel validation path.
- If a required tool or command is not present, report that clearly; do not claim verification.
