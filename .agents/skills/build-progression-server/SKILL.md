---
name: build-progression-server
description: Implement or diagnose RocknRolla rewards and progression in the Rust SpacetimeDB server. Use for tables, reducers, subscriptions, identities, waypoint claims, reward collection, progress persistence, generated bindings, or SpacetimeDB commands under server/. Do not use for client physics or packaging.
---

# Build Progression Server

Build the smallest transactional progression change. SpacetimeDB is the server and database; do not introduce a separate API layer.

## Workflow

1. Read `AGENTS.md`, the Rust workspace manifests, and the affected module, tables, reducers, and generated bindings.
2. Model only state required by the current reward or progress behavior.
3. Add or change a reducer for each client mutation and a public table or view only when the client must subscribe to it.
4. Use `ctx.sender()` as ownership identity. Never trust a player identity supplied as a reducer argument.
5. Accept client-reported waypoint or completion claims as valid. Validate shape, ownership, duplicates, and impossible database transitions, but do not emulate physics or add anti-cheat.
6. Update the Phaser client call or subscription when the feature spans both sides, then regenerate bindings instead of editing generated files.
7. Run `task server:check`, the narrowest relevant Rust tests, and `task server:build` when bindings or the module interface change.

## SpacetimeDB rules

- Keep reducers deterministic and transactional: no filesystem, network, external timers, or nondeterministic randomness.
- Read results through subscriptions; reducers mutate state and do not return query data.
- Use current Rust APIs: `&ReducerContext`, `ctx.sender()`, `ctx.db.table()`, and the `Table` trait.
- Make tables private unless clients need to subscribe.
- Add indexes only for current query patterns.
- Return `Result<(), String>` for expected failures; do not panic for user input.
- Use one straightforward table and reducer before decomposing schemas or introducing helpers.
- Do not add REST services, caches, queues, telemetry pipelines, migrations, or server-side physics.

When verification cannot run, state which manifest, module, binding target, or executable is missing.
